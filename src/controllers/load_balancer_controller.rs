use std::{collections::HashMap, sync::{Arc, OnceLock}};

use mongodb::bson::{doc, oid::ObjectId};
use tokio::sync::Mutex;

use crate::{models::{service_image_models::ServiceImage, service_load_balancer::{LoadBalancerEntry, LoadBalancerInsert, ServiceLoadBalancer}}, utils::mongodb_utils::MongoCollections};
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

pub async fn get_or_init_load_balancer(mongo_image: ObjectId, address: String) -> Result<bool, Box<dyn std::error::Error>>{
	//get the image first
	let find_one_result = MongoCollections::Images.as_collection::<ServiceImage>().find_one(doc!{
		"_id" : mongo_image
	}).await?;
	let result = match find_one_result {
		Some(service_image_entry) => {
			let hm = LOADBALANCERS.get().unwrap().lock().await;
			match hm.get(&service_image_entry.docker_image_id){
				Some(_service_load_balancer) => true,
				None => {
					drop(hm);
					let lb_find_result = MongoCollections::LoadBalancers.as_collection::<LoadBalancerEntry>().find_one(doc!{
						"mongo_image_reference" : &service_image_entry.docker_image_id
					}).await?;

					let lb_ref = match lb_find_result	 {
						Some(lb_entry) => lb_entry,
						None => {
							let lb_create_result  = MongoCollections::LoadBalancers.as_collection::<LoadBalancerInsert>().insert_one(LoadBalancerInsert {
								mongo_image_reference: service_image_entry._id.clone(),
								head: 0,
								behavior: ELoadBalancerBehavior::RoundRobin.to_string(),
								containers: Vec::new(),
							}).await?;

							MongoCollections::LoadBalancers.as_collection::<LoadBalancerEntry>().find_one(doc!{ "_id": lb_create_result.inserted_id.as_object_id().unwrap()}).await?.unwrap()
							
						},
					};
					let mut hm = LOADBALANCERS.get().unwrap().lock().await;
					hm.insert(service_image_entry.docker_image_id.clone(), ServiceLoadBalancer {
						_id: lb_ref._id,
						address: address,
						head: Arc::new(Mutex::new(lb_ref.head)),
						behavior: ELoadBalancerBehavior::RoundRobin,
						containers: Arc::new(Mutex::new(lb_ref.containers)),
						validated: Arc::new(Mutex::new(false)),
					});
					true
				},
			}
		},
		None => false,
	};
	Ok(result)

}
