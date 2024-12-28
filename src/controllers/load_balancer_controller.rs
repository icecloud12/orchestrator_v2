use custom_tcp_listener::models::types::Request;
use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};
use tokio::{net::TcpStream, sync::Mutex};
use tokio_postgres::types::Type;

use crate::{
    models::{
        service_container_models::{DockerImageId, ServiceContainer},
        service_image_models::ServiceImage,
        service_load_balancer::{
            LoadBalancerEntry, LoadBalancerEntryAggregate, LoadBalancerInsert, ServiceLoadBalancer,
        },
    },
    utils::postgres_utils::POSTGRES_CLIENT,
};

use super::image_controller::get_image;

pub static LOADBALANCERS: OnceLock<Arc<Mutex<HashMap<DockerImageId, ServiceLoadBalancer>>>> =
    OnceLock::new();
pub static AWAITED_LOADBALANCERS: OnceLock<
    Arc<Mutex<HashMap<DockerImageId, Vec<(Request, TcpStream)>>>>,
> = OnceLock::new();
pub static AWAITED_CONTAINERS: OnceLock<Arc<Mutex<HashMap<String, String>>>> = OnceLock::new();
#[derive(Debug)]
pub enum ELoadBalancerBehavior {
    RoundRobin,
}

impl ToString for ELoadBalancerBehavior {
    fn to_string(&self) -> String {
        match &self {
            ELoadBalancerBehavior::RoundRobin => "round_robin".to_string(),
        }
    }
}
#[derive(Debug)]
pub enum ELoadBalancerMode {
    FORWARD,
    QUEUE,
}

impl ToString for ELoadBalancerMode {
    fn to_string(&self) -> String {
        match &self {
            ELoadBalancerMode::FORWARD => "forward".to_string(),
            ELoadBalancerMode::QUEUE => "queue".to_string(),
        }
    }
}

pub fn init() {
    LOADBALANCERS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())));
    AWAITED_CONTAINERS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())));
    AWAITED_LOADBALANCERS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())));
}
pub async fn get_load_balancer_with_containers_by_image_id(
    image_fk: i32,
    docker_image_id: String,
    exposed_port: String,
    address: String,
) -> Result<ServiceLoadBalancer, String> {
    let load_balancer_query_results = POSTGRES_CLIENT.get().unwrap()
        .query_typed("SELECT lb.id as lb_id, lb.head as lb_head, c.id as c_id, c.docker_container_id, c.public_port, c.uuid
            from load_balancers lb
            LEFT JOIN load_balancer_container_junction as lbcj ON lb.id = lbcj.load_balancer_fk
            LEFT JOIN containers as c ON lbcj.container_fk = c.id
            WHERE lb.image_fk = $1;",&[(&image_fk, Type::INT2)] ).await;
    match load_balancer_query_results {
        Ok(rows) => {
            let mut containers: Vec<ServiceContainer> = Vec::new();
            let mut id: Option<i32> = None;
            let mut head: Option<i32> = None;
            if !rows.is_empty() {
                for row in rows.into_iter() {
                    if id.is_none() {
                        id = Some(row.get::<&str, i32>("lb_id"));
                        head = Some(row.get::<&str, i32>("lb_head"));
                    }
                    containers.push(ServiceContainer {
                        id: row.get::<&str, i32>("c_id"),
                        container_id: row.get::<&str, String>("docker_image_id"),
                        public_port: row.get::<&str, i32>("public_port"),
                        uuid: row.get::<&str, String>("uuid"),
                    });
                }
                let service_load_balancer: ServiceLoadBalancer = ServiceLoadBalancer {
                    id: id.unwrap(),
                    docker_image_id: docker_image_id.clone(),
                    exposed_port: exposed_port.clone(),
                    address: address.clone(),
                    head: head.unwrap(),
                    behavior: ELoadBalancerBehavior::RoundRobin,
                    mode: ELoadBalancerMode::QUEUE,
                    containers,
                    awaited_containers: HashMap::new(),
                    validated: false,
                    request_queue: Vec::new(),
                };
                Ok(service_load_balancer)
            } else {
                //insert load_balancer here
                let create_new_service_load_balancer =
                    ServiceLoadBalancer::new(image_fk, docker_image_id, exposed_port, address)
                        .await;
                create_new_service_load_balancer
            }
        }
        Err(err) => Err(err.to_string()),
    }
}
///This function returns the load_balancer key and if it is a fresh load balancer does 3 things:
/// 1. if locally cached then returns the key directly
/// 2. if not, rebuild it from the database record if it exists there
/// 3. last case. create a new one. Save it locally and in db
pub async fn get_or_init_load_balancer(
    image_fk: i32,
    address: String,
    exposed_port: String,
    service_image: ServiceImage,
) -> Result<(DockerImageId, bool), String> {
    //get the image first

    //matching image
    let hm = LOADBALANCERS.get().unwrap().lock().await;
    match hm.get(&service_image.docker_image_id) {
        //we can only return the key to access the lb because it is owned by the mutex
        Some(_service_load_balancer) => Ok((service_image.docker_image_id, false)), //returns an existing load-balance
        None => {
            //
            drop(hm);
            //dropping it early means that the lb hashmap can be used by other requests which is good.
            //At the same time the a problem exists where when it hits the same route|docker_image_id pair it might try to query and create a load_balancer instance while the lb creation is still being awaited,
            //this situation is bound to happen and does not need the stars to align(by pure luck)

            //awaited load balancer mutex should help us group them if ever the creation of lb might be delayed
            let awaited_lb = AWAITED_LOADBALANCERS.get().unwrap().blocking_lock();
            //blocking lock it to be prioritized
            awaited_lb.insert(service_image.docker_image_id.clone(), Some(vec![]));
            drop(awaited_lb);

            let load_balancer_query_result: Result<ServiceLoadBalancer, String> =
                get_load_balancer_with_containers_by_image_id(
                    image_fk.clone(),
                    service_image.docker_image_id.clone(),
                    exposed_port,
                    address,
                )
                .await;
            match load_balancer_query_result {
                Ok(mut lb) => {
                    //let hm = LOADBALANCERS.get().unwrap().lock().await;
                    //hm.insert(service_image.docker_image_id, lb )
                    let awaited_lb_mutex = AWAITED_LOADBALANCERS.get().unwrap();
                    let mut awaited_lb_lock = awaited_lb_mutex.lock().await;

                    //the queue might be empty;
                    let request_queue: Option<Vec<(Request, TcpStream)>> =
                        awaited_lb_lock.remove(&service_image.docker_image_id);
                    if request_queue.is_some() {
                        lb.request_queue = request_queue.unwrap();
                    };
                    drop(awaited_lb_lock);
                    let mut hm = LOADBALANCERS.get().unwrap().lock().await;
                    let new_lb: bool = lb.containers.len() + lb.awaited_containers.len() == 0;
                    hm.insert(service_image.docker_image_id.clone(), lb);
                    drop(hm);
                    Ok((service_image.docker_image_id, new_lb))
                }

                Err(err) => Err(err),
            }
        }
    }
}
