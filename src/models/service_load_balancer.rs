use std::sync::Arc;

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::controllers::load_balancer_controller::ELoadBalancerBehavior;


//this struct represents the load balancer of the current program
pub struct ServiceLoadBalancer {
    pub _id: ObjectId, //mongo_db_load_balancer_instance
    pub address: String,
    pub head: Arc<Mutex<usize>>,//current head pointer of the load_balancer
    pub behavior: ELoadBalancerBehavior,
    pub containers: Arc<Mutex<Vec<String>>>, //docker_container_id_instances
    pub validated: Arc<Mutex<bool>>, //initially false to let the program know if the containers are checke
}


//this struct represents the data from mongodb
//will be used to quickly restore the load-balancers from the old state just incase 
//if the orchestrator is closed due to user intervention but docker isn't
//load from cache thingy-majig if that makes sense
#[derive(Deserialize, Serialize)]
pub struct LoadBalancerEntry {
    pub _id: ObjectId, 
    pub mongo_image_reference:ObjectId, //mongo_image_reference
    pub head: usize,
    pub behavior: String,
    pub containers: Vec<String>
}

///struct for insert purposes only
#[derive(Serialize)]
pub struct LoadBalancerInsert {
    pub mongo_image_reference:ObjectId, //mongo_image_reference
    pub head: usize,
    pub behavior: String,
    pub containers: Vec<String>
}