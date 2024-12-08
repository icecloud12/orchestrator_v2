use std::{collections::HashMap, sync::{Arc, OnceLock}};

use bollard::{image, secret::Service, Docker};
use mongodb::bson::doc;
use tokio::sync::Mutex;

use crate::{models::{service_image_models::ServiceImage, service_load_balancer::{self, LoadBalancerEntry, LoadBalancerInsert, ServiceLoadBalancer}, service_route_model::ServiceRoute}, utils::mongodb_utils::{self, MongoCollections}};
type DockerImageId = String;

pub static LOADBALANCERS: OnceLock<Arc<Mutex<HashMap<DockerImageId, ServiceLoadBalancer>>>> = OnceLock::new();

pub enum ELoadBalancerBehavior {
    RoundRobin
}

impl ToString for ELoadBalancerBehavior {
    fn to_string(&self) -> String {
        match &self {
            &Self::RoundRobin => "round_robin".to_string()
        }
    }
}

pub fn init(){
	LOADBALANCERS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())));
}

async fn get_or_init_load_balancer(service_route:ServiceRoute) -> Result<DockerImageId, String>{
	//get the image first
	let result = MongoCollections::Images.as_collection::<ServiceImage>().find_one(doc!{
		"_id" : service_route.mongo_image
	}).await;
	let load_balancer_result: Result<DockerImageId, String> = match result {
		Ok(image) => {
			match &image {
				Some(image_result) => {
					//find local instanced load_balancer if exists
					let hm = LOADBALANCERS.get().unwrap().lock().await;
					match hm.get(&image_result.docker_image_id) {
						Some(_service_load_balancer) => {
							//do nothing since it is initialized
							Ok(image.unwrap().docker_image_id)
						},
						None => {
							drop(hm); //drop early so it can be used by others
							//check from db if we can recover previous LB
							let lb_find_result: Result<Option<LoadBalancerEntry>, mongodb::error::Error> = MongoCollections::LoadBalancers.as_collection::<LoadBalancerEntry>().find_one(doc!{
								"mongo_image_reference" : &image_result.docker_image_id
							}).await;
							//find or create_and_get load balancer entry from db
							let lb_ref: Result<LoadBalancerEntry, String> = match lb_find_result {
								Ok(lb_entry) => { //successful query
									if lb_entry.is_none() { //no result //create an entry
										let lb_create_result  = MongoCollections::LoadBalancers.as_collection::<LoadBalancerInsert>().insert_one(LoadBalancerInsert {
											mongo_image_reference: image_result._id.clone(),
											head: 0,
											behavior: ELoadBalancerBehavior::RoundRobin.to_string(),
											containers: Vec::new(),
										}).await;

										let load_balancer_entry_result = match lb_create_result {
											Ok(insert_result) => {
												let entry_result = MongoCollections::LoadBalancers.as_collection::<LoadBalancerEntry>().find_one(doc!{ "_id": insert_result.inserted_id.as_object_id().unwrap()}).await;
												match entry_result {
													Ok(entry) => {
														Ok(entry.unwrap())
													},
													Err(err) => Err(err.to_string()),
												}
											},
											Err(err) => Err(err.to_string()),
										};
										match load_balancer_entry_result {
											Ok(entry) => Ok(entry),
											Err(err) => Err(err),
										}
									}else{
										Ok(lb_entry.unwrap()) //return existing data
									}
								},
								Err(err) => Err(err.to_string()),
							};
							//create localized instance from LoadBalancerEntry -> Service LoadBalancer
							match lb_ref {
								Ok(entry) => {
									let mut hm = LOADBALANCERS.get().unwrap().lock().await;
									hm.insert(image_result.docker_image_id.clone(), ServiceLoadBalancer {
										_id: entry._id,
										address: service_route.address,
										head: Arc::new(Mutex::new(entry.head)),
										behavior: ELoadBalancerBehavior::RoundRobin,
										containers: Arc::new(Mutex::new(entry.containers)),
										validated: Arc::new(Mutex::new(false)),
									});
									drop(hm)
								},
								Err(_) => todo!(),
							}
							Ok(image.unwrap().docker_image_id)
						},
					}
				},
				None => Err("Dangling route! cannot find paired image".to_string()),
			}
			
		},
		//a mongodb error occured
		Err(err) => Err(err.to_string()),
	};
	load_balancer_result
}