use std::{collections::HashMap, sync::Arc};

use bollard::{container::ListContainersOptions, service::ContainerStateStatusEnum};
use custom_tcp_listener::models::types::Request;
use tokio::net::TcpStream;
use tokio_postgres::types::Type;
use uuid::Uuid;

use crate::{
    controllers::{
        container_controller, 
        load_balancer_controller::{ELoadBalancerBehavior, ELoadBalancerMode, AWAITED_CONTAINERS}, load_balancer_container_junction_controller,
    },
    utils::{docker_utils::DOCKER, postgres_utils::POSTGRES_CLIENT},
};

use super::service_container_models::ServiceContainer;

#[derive(Debug)]
//this struct represents the load balancer of the current program
pub struct ServiceLoadBalancer {
    pub id: i32, //mongo_db_load_balancer_instance,
    pub docker_image_id: String,
    pub exposed_port: String,
    pub address: String,
    pub head: i32, //current head pointer of the load_balancer
    pub behavior: ELoadBalancerBehavior,
    pub mode: ELoadBalancerMode,
    pub containers: Vec<ServiceContainer>, //docker_container_id_instances
    pub awaited_containers: HashMap<String, ServiceContainer>, //docker_containers not pushed to the active container vector
    pub validated: bool, //initially false to let the program know if the containers are checke,
    pub request_queue: Vec<(Request, TcpStream, Arc<Uuid>)>,
}

impl ServiceLoadBalancer {

    pub async fn new(image_fk: Arc<i32>, docker_image_id: String, exposed_port: String, address:String) -> Result<ServiceLoadBalancer, String>{

        let head: i32 = 0;
        let behavior: String = ELoadBalancerBehavior::RoundRobin.to_string();
        let insert_result = POSTGRES_CLIENT.get().unwrap().query_typed("INSERT INTO load_balancers (image_fk, head, behavior) VALUES ($1, $2, $3) RETURNING id", &[(image_fk.as_ref(), Type::INT4), 
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
                    validated: false,
                    request_queue: vec![]
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
    pub async fn next_container(&mut self) -> Result<(i32,i32), ()> {
        let containers = &self.containers;

        let container_len:i32 = containers.len() as i32;
        let ret: Result<(i32, i32), ()> = if container_len == 0 {
            // let create_container_result = self.create_container().await;
            // //maybe create some logic before creating the container here for scalability logic
            // match create_container_result {
            //     Ok(container_port) => Ok(container_port),
            //     Err(err) => Err(err),
            // }
            Err(())
        } else {
            //has container entries
            //move the head
            let head = &mut self.head;
            *head = (*head + 1) % container_len;
            let container_ref = containers.get(*head as usize).unwrap();
            let public_port = container_ref.public_port.clone();
            let container_fk = container_ref.id.clone();
            Ok((public_port, container_fk))
        };
        ret
    }

    pub async fn create_container(&mut self) -> Result<ServiceContainer, String> {
        let col =
            container_controller::create_container(&self.docker_image_id, &self.exposed_port).await;

        match col {
            Ok(service_container) => {
                let _ = load_balancer_container_junction_controller::create(&self.id, &service_container.id)
                    .await;
                Ok(service_container)
            }
            Err(err) => Err(err),
        }
    }
    pub fn queue_request(&mut self, request: Request, tcp_stream: TcpStream, request_uuid: Arc<Uuid>) {
        self.request_queue.push((request, tcp_stream, request_uuid));
    }
    pub fn add_container(&mut self, container: ServiceContainer) -> &ServiceContainer {
        let containers = &mut self.containers;
        containers.push(container);
        let container_ref = containers.last().unwrap();
        container_ref
    }
    pub async fn queue_container(&mut self, container: ServiceContainer) {
        let mut await_container_lock = AWAITED_CONTAINERS.get().unwrap().lock().await;
        await_container_lock.insert(container.uuid.clone(), self.docker_image_id.clone());
        let awaited_containers = &mut self.awaited_containers;
        awaited_containers.insert(container.uuid.clone(), container);
    }
    pub fn empty_queue(&mut self) -> Vec<(Request, TcpStream, Arc<Uuid>)> {
        std::mem::take(&mut self.request_queue)
    }
    ///This function returns an Option of Vec<ServiceContainer> where
    ///The Vec<i32> defines the invalid container id(s) to prune out of the db rows
    ///for both load_balancer_container_junction and container tables
    pub async fn validate_containers(&mut self) -> Result<Option<Vec<i32>>,String> {
        let containers: Vec<ServiceContainer> = std::mem::take(&mut self.containers);
        let mut container_map: HashMap<String, ServiceContainer> = HashMap::new();
        containers.into_iter().for_each(|container| {
            container_map.insert(container.container_id.clone(), container);
        });
        let mut filters = HashMap::new();

        filters.insert("id", container_map.keys().map(|k| k.as_str()).collect());
        let options = ListContainersOptions {
            all: true,
            filters,
            ..Default::default()
        };
        let docker = DOCKER.get().unwrap();
        let container_list_result = match docker.list_containers(Some(options)).await {
            Ok(container_list) => {
                let mut await_container_lock = AWAITED_CONTAINERS.get().unwrap().lock().await;
                
                for container_summary in container_list.into_iter() {
                    let container_status = container_summary.state.unwrap();
                    if container_status == ContainerStateStatusEnum::RUNNING.as_ref() {
                        let container: ServiceContainer = container_map
                            .remove(container_summary.id.unwrap().as_str())
                            .unwrap();

                        self.containers.push(container);
                    } else {
                        let container = container_map
                            .remove(container_summary.id.unwrap().as_str())
                            .unwrap();
                        //if we cannot start the container we must remove the container from db and docker
                        //await the containers before trying to start
                            let uuid = container.uuid.clone();
                            await_container_lock
                                .insert(uuid.clone(), self.docker_image_id.clone());
                            let awaited_containers = &mut self.awaited_containers;
                            awaited_containers.insert(uuid, container);
                            
                    };
                }
                if container_map.is_empty() {
                    Ok(None)
                }else{
                    Ok(Some(
                        container_map.into_iter().map( |(_key, container)| {
                            container.id
                        }).collect()
                    ))
                }
                
                //lock should be dropped here
            }
            Err(docker_container_list_error) => Err(docker_container_list_error.to_string()),
        };
        container_list_result
    }
    pub async fn remove_containers(container_ids: Vec<i32>) {
        let client = POSTGRES_CLIENT.get().unwrap();
        let delete_result = client.query_typed( "DELETE from load_balancer_container_junction where container_fk = any($1)",&[(&container_ids, Type::INT4_ARRAY)]).await;
        if delete_result.is_ok() {
            
            let _delete_container_result = client.query_typed("DELETE FROM containers where id = ANY($1)", &[(&container_ids, Type::INT4_ARRAY)]).await;
        }
    }
}

