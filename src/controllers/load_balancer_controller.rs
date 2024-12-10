use std::{collections::HashMap, sync::{Arc, OnceLock}};

use mongodb::{bson::{doc, oid::ObjectId}, error};
use tokio::sync::Mutex;

use crate::{models::{docker_container_models::DockerImageId, service_image_models::ServiceImage, service_load_balancer::{LoadBalancerEntry, LoadBalancerEntryAggregate, LoadBalancerInsert, ServiceLoadBalancer}}, utils::mongodb_utils::MongoCollections};


pub static LOADBALANCERS: OnceLock<Arc<Mutex<HashMap<DockerImageId, ServiceLoadBalancer>>>> = OnceLock::new();
#[derive(Debug)]
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

pub async fn get_or_init_load_balancer(mongo_image: ObjectId, address: String) -> Result<DockerImageId, String>{
	//get the image first
	let find_one_result = MongoCollections::Images.as_collection::<ServiceImage>().find_one(doc!{
		"_id" : mongo_image
	}).await;
	match find_one_result {
		Ok(find_one_result_option) => {
			let result = match find_one_result_option {
				Some(service_image_entry) => {
					let hm = LOADBALANCERS.get().unwrap().lock().await;
					match hm.get(&service_image_entry.docker_image_id){
						Some(_service_load_balancer) => Ok(service_image_entry.docker_image_id),
						None => {
							drop(hm);
							let pipeline = vec![
								doc!{
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
										"localField": "junc.container_id",
										"foreignField": "_id",
										"as" : "containers"
									}
								},
								doc!{
									"$match": {
										"mongo_image_reference" : { "$eq" : &service_image_entry._id}
									}
								},
								doc!{
									"$project":{
										"_junc": 0,
										"containers.mongo_image_reference": 0,
										"containers.container_id": 0,
										"containers.public_port": 0,
									}
								},
								doc!{
									"$limit" : 1
								}
							];
							
							let mut result_cursor = MongoCollections::LoadBalancers.as_collection::<LoadBalancerEntryAggregate>().
							aggregate(pipeline).with_type::<LoadBalancerEntryAggregate>().await;
							
							match  result_cursor {
								Ok(mut cursor) => {
									match cursor.advance().await {
										Ok(advanced) => {
											let lb_ref_result = if advanced{
												Ok(cursor.deserialize_current().unwrap())
											}else{ // empty result
												let lb_create_result  = MongoCollections::LoadBalancers.as_collection::<LoadBalancerInsert>().insert_one(LoadBalancerInsert {
													mongo_image_reference: service_image_entry._id.clone(),
													head: 0,
													behavior: ELoadBalancerBehavior::RoundRobin.to_string()
												}).await;
												match lb_create_result {
													Ok(insert_result) => {
														let new_entry_cursor_result = MongoCollections::LoadBalancers.as_collection::<LoadBalancerEntry>().find_one(doc!{
															"_id": insert_result.inserted_id.as_object_id().unwrap()
														}).await;
														match new_entry_cursor_result {
															Ok(new_entry_cursor) => {
																let l_b_entry = new_entry_cursor.unwrap();
																Ok(LoadBalancerEntryAggregate {
																	_id: l_b_entry._id,
																	mongo_image_reference: l_b_entry.mongo_image_reference,
																	head: 0,
																	behavior: l_b_entry.behavior, // need to be based from string to Enum
																	containers: vec![],
																})
															},
															Err(err) => Err(err.to_string()),
														}
													},
													Err(err) => Err(err.to_string()),
												}								
											};
											match lb_ref_result {
												Ok(lb_ref) => {
													let mut hm = LOADBALANCERS.get().unwrap().lock().await;
													hm.insert(
														service_image_entry.docker_image_id.clone(), ServiceLoadBalancer {
														_id: lb_ref._id,
														address: address,
														head: Arc::new(Mutex::new(lb_ref.head)),
														behavior: ELoadBalancerBehavior::RoundRobin,
														containers: Arc::new(Mutex::new(lb_ref.containers)),
														validated: Arc::new(Mutex::new(false)),
													});
												},
												Err(_) => todo!(),
											}
											
											Ok(service_image_entry.docker_image_id)
										},
										Err(advance_error) => { // mongodb error
											//internal server error
											Err(advance_error.to_string())
										},
									}
								}
								Err(err) => { // mongodb error
									println!("{:#?}", err); //internal server here
									Err(err.to_string())
								},
							}
							
						},
					}
				},
				None => Err("Cannot find image pair for route".to_string()),
			};
			result
		},
		Err(err) => Err(err.to_string()),
	}
	
	//Ok(result)

}
