use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use mongodb::{
    bson::{doc, oid::ObjectId},
    error,
};
use tokio::sync::Mutex;

use crate::{
    models::{
        service_container_models::DockerImageId,
        service_image_models::ServiceImage,
        service_load_balancer::{
            LoadBalancerEntry, LoadBalancerEntryAggregate, LoadBalancerInsert, ServiceLoadBalancer,
        },
    },
    utils::mongodb_utils::MongoCollections,
};

pub static LOADBALANCERS: OnceLock<Arc<Mutex<HashMap<DockerImageId, ServiceLoadBalancer>>>> =
    OnceLock::new();
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
}
///This function returns the load_balancer key and if it is a fresh load balancer does 3 things:
/// 1. if locally cached then returns the key directly
/// 2. if not, rebuild it from the database record if it exists there
/// 3. last case. create a new one. Save it locally and in db
pub async fn get_or_init_load_balancer(
    mongo_image: ObjectId,
    address: String,
    exposed_port: String,
) -> Result<(DockerImageId, bool), String> {
    //get the image first
    let find_one_result = MongoCollections::Images
        .as_collection::<ServiceImage>()
        .find_one(doc! {
            "_id" : mongo_image
        })
        .await;
    match find_one_result {
        Ok(find_one_result_option) => {
            let result = match find_one_result_option {
                Some(service_image_entry) => {
                    let hm = LOADBALANCERS.get().unwrap().lock().await;
                    match hm.get(&service_image_entry.docker_image_id) {
                        Some(_service_load_balancer) => {
                            Ok((service_image_entry.docker_image_id, false))
                        }
                        None => {
                            drop(hm);
                            let pipeline = vec![
                                doc! {
                                    "$lookup":{
                                        "from":"load_balancer_container_junction",
                                        "localField": "_id",
                                        "foreignField": "load_balancer_id",
                                        "as" : "_junc"
                                    }
                                },
                                doc! {
                                    "$lookup" : {
                                        "from": "containers",
                                        "localField": "_junc.container_id",
                                        "foreignField": "_id",
                                        "as" : "containers"
                                    }
                                },
                                doc! {
                                    "$match": {
                                        "mongo_image_reference" : { "$eq" : &service_image_entry._id}
                                    }
                                },
                                doc! {
                                    "$project":{
                                        "_junc": 0,
                                    }
                                },
                                doc! {
                                    "$limit" : 1
                                },
                            ];

                            let result_cursor = MongoCollections::LoadBalancers
                                .as_collection::<LoadBalancerEntryAggregate>()
                                .aggregate(pipeline)
                                .with_type::<LoadBalancerEntryAggregate>()
                                .await;

                            match result_cursor {
                                Ok(mut cursor) => {
                                    match cursor.advance().await {
                                        Ok(advanced) => {
                                            let lb_ref_result = if advanced {
                                                let cursor_deserialized =
                                                    cursor.deserialize_current();
                                                println!("{:#?}", &cursor_deserialized);
                                                match cursor_deserialized {
                                                    Ok(t) => Ok(t),
                                                    Err(err) => Err(err.to_string()),
                                                }
                                                // Ok(cursor.deserialize_current().unwrap())
                                            } else {
                                                // empty result
                                                let lb_create_result =
                                                    MongoCollections::LoadBalancers
                                                        .as_collection::<LoadBalancerInsert>()
                                                        .insert_one(LoadBalancerInsert {
                                                            mongo_image_reference:
                                                                service_image_entry._id.clone(),
                                                            head: 0,
                                                            behavior:
                                                                ELoadBalancerBehavior::RoundRobin
                                                                    .to_string(),
                                                        })
                                                        .await;
                                                match lb_create_result {
                                                    Ok(insert_result) => {
                                                        Ok(LoadBalancerEntryAggregate {
                                                            _id: insert_result
                                                                .inserted_id
                                                                .as_object_id()
                                                                .unwrap(),
                                                            mongo_image_reference:
                                                                service_image_entry._id,
                                                            head: 0,
                                                            behavior:
                                                                ELoadBalancerBehavior::RoundRobin
                                                                    .to_string(),
                                                            containers: vec![],
                                                        })
                                                    }
                                                    Err(err) => Err(err.to_string()),
                                                }
                                            };
                                            match lb_ref_result {
                                                Ok(lb_ref) => {
                                                    let mut lb = ServiceLoadBalancer {
                                                        _id: lb_ref._id,
                                                        address: address,
                                                        head: lb_ref.head,
                                                        behavior: ELoadBalancerBehavior::RoundRobin,
                                                        containers: lb_ref.containers,
                                                        validated: false,
                                                        docker_image_id: service_image_entry
                                                            .docker_image_id
                                                            .clone(),
                                                        exposed_port,
                                                        mode: ELoadBalancerMode::QUEUE,
                                                        request_queue: Vec::new(),
                                                        awaited_containers: HashMap::new(),
                                                    };
                                                    let new_lb: bool = lb.containers.len() == 0;
                                                    //check container instances here if they are valid
                                                    println!("validating containers");
                                                    lb.validate_containers().await;
                                                    let mut hm =
                                                        LOADBALANCERS.get().unwrap().lock().await;
                                                    hm.insert(
                                                        service_image_entry.docker_image_id.clone(),
                                                        lb,
                                                    );
                                                    Ok((
                                                        service_image_entry.docker_image_id,
                                                        new_lb,
                                                    ))
                                                }
                                                Err(err) => Err(err),
                                            }
                                        }
                                        Err(advance_error) => {
                                            // mongodb error
                                            //internal server error
                                            Err(advance_error.to_string())
                                        }
                                    }
                                }
                                Err(err) => {
                                    // mongodb error
                                    Err(err.to_string())
                                }
                            }
                        }
                    }
                }
                None => Err("Cannot find image pair for route".to_string()),
            };
            result
        }
        Err(err) => Err(err.to_string()),
    }

    //Ok(result)
}
