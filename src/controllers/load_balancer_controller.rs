use crate::{
    db::{
        container_instance_port_pool_junction::ContainerInstancePortPoolJunctionColumns,
        containers::ServiceContainerColumns,
        load_balancers::{get_existing_load_balancer_by_image, ServiceLoadBalancersColumns},
        port_pool::PortPoolColumns,
    },
    models::{
        service_container_models::{DockerImageId, ServiceContainer},
        service_image_models::ServiceImage,
        service_load_balancer::ServiceLoadBalancer,
    },
};
use custom_tcp_listener::models::types::Request;
use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};
use tokio::{net::TcpStream, sync::Mutex};
use uuid::Uuid;

pub static LOADBALANCERS: OnceLock<Arc<Mutex<HashMap<DockerImageId, ServiceLoadBalancer>>>> =
    OnceLock::new();
pub static AWAITED_LOADBALANCERS: OnceLock<
    Arc<Mutex<HashMap<DockerImageId, Vec<(Request, TcpStream, Arc<Uuid>)>>>>,
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
    image_fk: Arc<i32>,
    docker_image_id: String,
    exposed_port: String,
    address: String,
) -> Result<ServiceLoadBalancer, String> {
    let load_balancer_query_results = get_existing_load_balancer_by_image(&image_fk).await;
    println!("existing lb result {:#?}", load_balancer_query_results);
    match load_balancer_query_results {
        Ok(rows) => {
            let mut containers: Vec<ServiceContainer> = Vec::new();
            let mut id: Option<i32> = None;
            let mut head: Option<i32> = None;
            if !rows.is_empty() {
                for row in rows.into_iter() {
                    if id.is_none() {
                        //execute this only on the first row result
                        id = Some(row.get::<&str, i32>(ServiceLoadBalancersColumns::ID.as_str()));
                        head =
                            Some(row.get::<&str, i32>(ServiceLoadBalancersColumns::HEAD.as_str()));
                    }
                    if let Ok(c_id) = row.try_get::<&str, i32>(ServiceContainerColumns::ID.as_str())
                    {
                        containers.push(ServiceContainer {
                            id: c_id,
                            container_id: row.get::<&str, String>(
                                ServiceContainerColumns::DOCKER_CONTAINER_ID.as_str(),
                            ),
                            public_port: row.get::<&str, i32>(PortPoolColumns::PORT.as_str()),
                            uuid: row.get::<&str, Uuid>(
                                ContainerInstancePortPoolJunctionColumns::UUID.as_str(),
                            ),
                            cippj_fk: row.get::<&str, i32>(
                                ServiceContainerColumns::CONTAINER_INSTANCE_PORT_POOL_JUNCTION_FK
                                    .as_str(),
                            ),
                        });
                    }
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
        Err(err) => {
            tracing::warn!("error when trying to get_existing_load_balancer_by_image");
            Err(err.to_string())
        }
    }
}
///This function returns the load_balancer key and if it is a fresh load balancer does 3 things:
/// 1. if locally cached then returns the key directly
/// 2. if not, rebuild it from the database record if it exists there
/// 3. last case. create a new one. Save it locally and in db
pub async fn get_or_init_load_balancer(
    image_fk: Arc<i32>,
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
            let mut awaited_lb = AWAITED_LOADBALANCERS.get().unwrap().lock().await;
            awaited_lb.insert(service_image.docker_image_id.clone(), vec![]);
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
                    let request_queue = awaited_lb_lock.remove(&service_image.docker_image_id);
                    if request_queue.is_some() {
                        lb.request_queue = request_queue.unwrap();
                    };
                    drop(awaited_lb_lock);
                    //trim the containers

                    let mut hm = LOADBALANCERS.get().unwrap().lock().await;

                    let mut container_len = lb.containers.len();
                    let mut awaited_container_len = lb.awaited_containers.len();
                    let mut new_lb: bool = container_len + awaited_container_len == 0;
                    let new_lb_check: Option<String> = if !new_lb {
                        let validate_containers_result: Result<(), String> =
                            lb.validate_containers().await;
                        match validate_containers_result {
                            Ok(_) => {
                                container_len = lb.containers.len();
                                awaited_container_len = lb.awaited_containers.len();
                                if container_len > 0 {
                                    lb.mode = ELoadBalancerMode::FORWARD;
                                    None
                                } else if awaited_container_len > 0 {
                                    let awaited_containers = &lb.awaited_containers;
                                    let any_entry = awaited_containers.iter().next().unwrap();

                                    let mut awaited_container_lock =
                                        AWAITED_CONTAINERS.get().unwrap().lock().await;
                                    awaited_container_lock.insert(
                                        any_entry.1.uuid.to_string(),
                                        lb.docker_image_id.clone(),
                                    );
                                    let start_container_result =
                                        any_entry.1.start_container().await;
                                    let result = match start_container_result {
                                        Ok(_) => {
                                            drop(awaited_container_lock);
                                            None
                                        }
                                        Err(err) => {
                                            //remove the entry from the hashmap
                                            let _remove_option = awaited_container_lock
                                                .remove(&any_entry.1.uuid.to_string());
                                            drop(awaited_container_lock);
                                            Some(err.to_string())
                                        }
                                    };
                                    result
                                } else {
                                    //somehhow goes here
                                    lb.mode = ELoadBalancerMode::FORWARD;
                                    new_lb = true;
                                    None
                                }
                            }
                            Err(docker_error) => Some(docker_error),
                        }
                    } else {
                        None
                    };
                    match new_lb_check {
                        Some(err) => Err(err),
                        None => {
                            hm.insert(service_image.docker_image_id.clone(), lb);
                            drop(hm);
                            Ok((service_image.docker_image_id, new_lb))
                        }
                    }
                }

                Err(err) => Err(err),
            }
        }
    }
}
