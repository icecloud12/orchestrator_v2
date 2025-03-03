use std::{collections::HashMap, sync::Arc};

use bollard::{container::ListContainersOptions, service::ContainerStateStatusEnum};
use custom_tcp_listener::models::types::Request;
use tokio::net::TcpStream;
use tokio_postgres::types::Type;
use tokio_rustls::server::TlsStream;
use uuid::Uuid;

use crate::{
    controllers::{
        container_controller, load_balancer_container_junction_controller, load_balancer_controller::{ELoadBalancerBehavior, ELoadBalancerMode, AWAITED_CONTAINERS}
    }, db::container_instance_port_pool_junction::deallocate_port, utils::orchestrator_utils::RouterDecoration
};

use super::service_container_models::ServiceContainer;

#[derive(Debug)]
//this struct represents the load balancer of the current program
pub struct ServiceLoadBalancer {
    pub id: i32, //mongo_db_load_balancer_instance,
    pub docker_image_id: String,
    pub exposed_port: String,
    pub address: String,
    pub head: usize, //current head pointer of the load_balancer
    pub behavior: ELoadBalancerBehavior,
    pub mode: ELoadBalancerMode,
    pub containers: Vec<ServiceContainer>, //docker_container_id_instances
    pub awaited_containers: HashMap<String, ServiceContainer>, //docker_containers not pushed to the active container vector
    pub request_queue: Vec<(Request, TlsStream<TcpStream>, Arc<Uuid>)>,
    pub https: bool,
    pub decoration: Arc<RouterDecoration>
}

impl ServiceLoadBalancer {

    pub async fn new(image_fk: Arc<i32>, docker_image_id: String, exposed_port: String, address:String, https: bool, decoration: Arc<RouterDecoration>) -> Result<ServiceLoadBalancer, String>{

        let head: i32 = 0;
        let behavior: String = ELoadBalancerBehavior::RoundRobin.to_string();
        let insert_result = decoration.postgres_client.query_typed("INSERT INTO load_balancers (image_fk, head, behavior) VALUES ($1, $2, $3) RETURNING id", &[(image_fk.as_ref(), Type::INT4), 
            (&head, Type::INT4), 
            (&behavior, Type::TEXT)]).await;
        match insert_result {
            Ok(rows) => {
                //returns exactly 1 row
                let id: i32 = rows[0].get::<&str,i32>("id");
                let load_balancer = ServiceLoadBalancer{
                    id,
                    docker_image_id,
                    exposed_port,
                    address,
                    head: 0,
                    behavior: ELoadBalancerBehavior::RoundRobin,
                    mode: ELoadBalancerMode::QUEUE,
                    containers: vec![], 
                    awaited_containers: HashMap::new(),
                    request_queue: vec![],
                    https,
                    decoration
                };
                Ok(load_balancer)
            }
            Err(err) => {Err(err.to_string())}
        }
    }
    ///the container array size differ depending on:
    /// 1. if it's newly created
    /// 2. the recorded load_balancer in the database have/have no entries
    //returns Ok((Option<container_port>, Option<AwaitedContainerInstance>),()>
    pub async fn next_container(&mut self) -> Option<(i32,i32)> {
        let containers = &self.containers;

        let container_len = containers.len();
        let ret: Option<(i32, i32)> = if container_len == 0 {
            None
        } else {
            //has container entries
            //move the head
            let head = &mut self.head;
            *head = (*head + 1) % container_len;
            let container_ref = containers.get(*head).unwrap();
            let public_port = container_ref.public_port.clone();
            let container_fk = container_ref.id.clone();
            Some((public_port, container_fk))
        };
        ret
    }

    pub async fn create_container(&mut self) -> Result<ServiceContainer, String> {
        let col =
            container_controller::create_container(&self.docker_image_id, &self.exposed_port, self.decoration.docker_connection.clone(), self.decoration.postgres_client.clone(), &self.decoration.orchestrator_uri.as_ref(), &self.decoration.orchestrator_public_uuid.as_ref()).await;

        match col {
            Ok(service_container) => {
                // it is ok that it fails but if that was the case then the container is non-trackable
                load_balancer_container_junction_controller::create(Arc::new(self.id), Arc::new(service_container.id), self.decoration.postgres_client.clone());
                Ok(service_container)
            }
            Err(err) => Err(err),
        }
    }
    pub fn queue_request(&mut self, request: Request, tcp_stream: TlsStream<TcpStream>, request_uuid: Arc<Uuid>) {
        self.request_queue.push((request, tcp_stream, request_uuid));
    }
    pub fn add_container(&mut self, container: ServiceContainer) -> &ServiceContainer {
        let containers = &mut self.containers;
        containers.push(container);
        let container_ref = containers.last().unwrap();
        container_ref
    }
    pub async fn remove_container(&mut self, container_id: i32) -> Option<ServiceContainer> {
        let containers = &mut self.containers;
        let index = containers.iter().position(|c| c.id == container_id);
        if index.is_some() {
            tracing::info!("[INFO] Dropping Container");
            let container = containers.remove(index.unwrap());
            deallocate_port(vec![container.cippj_fk], self.decoration.postgres_client.clone());// free port usage
            let _delete_container_result = container.delete_container(self.decoration.docker_connection.clone()).await;
            Some(container)
        }else{
            None
        }
    }
    pub async fn queue_container(&mut self, container: ServiceContainer) {
        let mut await_container_lock = AWAITED_CONTAINERS.get().unwrap().lock().await;
        await_container_lock.insert(container.uuid.to_string(), self.docker_image_id.clone());
        let awaited_containers = &mut self.awaited_containers;
        awaited_containers.insert(container.uuid.to_string(), container);
    }
    pub fn empty_queue(&mut self) -> Vec<(Request, TlsStream<TcpStream>, Arc<Uuid>)> {
        std::mem::take(&mut self.request_queue)
    }
    ///This function returns an Option of Vec<ServiceContainer> where
    ///The Vec<i32> defines the invalid container id(s) to prune out of the db rows
    ///for both load_balancer_container_junction and container tables
    pub async fn validate_containers(&mut self) -> Result<(),String> {
        let lb_containers: Vec<ServiceContainer> = std::mem::take(&mut self.containers);
        let mut container_map: HashMap<String, ServiceContainer> = HashMap::new();
        lb_containers.into_iter().for_each(|container| {
            container_map.insert(container.container_id.clone(), container);
        });
        //docker filters
        let mut filters = HashMap::new();

        filters.insert("id", container_map.keys().map(|k| k.as_str()).collect());
        let options = ListContainersOptions {
            all: true,
            filters,
            ..Default::default()
        };
        let docker = &self.decoration.docker_connection;
        let container_list_result = match docker.list_containers(Some(options)).await {
            Ok(container_list) => {
                // let mut await_container_lock = AWAITED_CONTAINERS.get().unwrap().lock().await;
                
                for container_summary in container_list.into_iter() {
                    let container_status = container_summary.state.unwrap();
                    if container_status == ContainerStateStatusEnum::RUNNING.as_ref() {
                        let container: ServiceContainer = container_map
                            .remove(container_summary.id.unwrap().as_str())
                            .unwrap();

                        self.containers.push(container);
                    }
                }
                if container_map.is_empty() {
                    return Ok(());
                }else{
                    // if container_map still has remaining entries then these containers are no longer valid
                    let deallocate_ids_buffer: Vec<i32> = container_map.into_iter().map(|(_key, value)| {
                         return value.cippj_fk;
                    }).collect();
                    deallocate_port(deallocate_ids_buffer,self.decoration.postgres_client.clone());
                    return Ok(());
                }
                
                //lock should be dropped here
            }
            Err(docker_container_list_error) => Err(docker_container_list_error.to_string()),
        };
        container_list_result
    }
}

