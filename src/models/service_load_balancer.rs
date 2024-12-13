use std::{ops::DerefMut, sync::Arc};

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::controllers::load_balancer_controller::{ELoadBalancerBehavior, LOADBALANCERS};

use super::service_container_models::ServiceContainer;

#[derive(Debug)]
//this struct represents the load balancer of the current program
pub struct ServiceLoadBalancer {
    pub _id: ObjectId, //mongo_db_load_balancer_instance,
    pub docker_image_id: String,
    pub address: String,
    pub head: Arc<Mutex<usize>>,//current head pointer of the load_balancer
    pub behavior: ELoadBalancerBehavior,
    pub containers: Arc<Mutex<Vec<ServiceContainer>>>, //docker_container_id_instances
    pub validated: Arc<Mutex<bool>>, //initially false to let the program know if the containers are checke
}

impl ServiceLoadBalancer {
    ///the container array size differ depending on:
    /// 1. if it's newly created
    /// 2. the recorded load_balancer in the database have/have no entries
    pub async fn next_container(&mut self, load_balancer_key:String) -> Option<usize>
    {
        let containers = &self.containers;
        let containers_lock = containers.lock().await;
        let container_len = containers_lock.len();
        
        let ret  = if container_len == 0 {
            None
        }else{//has container entries
            //move the head
            let head = &self.head;
            let mut num = head.lock().await;
            *num = (*num + 1) % container_len;
            
            Some(containers_lock.get(*num).unwrap().public_port.clone())
        };
        ret
    }
}

//this struct represents the data from mongodb
//will be used to quickly restore the load-balancers from the old state just incase 
//if the orchestrator is closed due to user intervention but docker isn't
//load from cache thingy-majig if that makes sense
#[derive(Deserialize, Serialize, Debug)]
pub struct LoadBalancerEntryAggregate {
    pub _id: ObjectId, 
    pub mongo_image_reference:ObjectId, //mongo_image_reference
    pub head: usize,
    pub behavior: String,
	pub containers: Vec<ServiceContainer>
}

#[derive(Deserialize, Serialize)]
pub struct LoadBalancerEntry {
    pub _id: ObjectId, 
    pub mongo_image_reference:ObjectId, //mongo_image_reference
    pub head: usize,
    pub behavior: String,
}

///struct for insert purposes only
#[derive(Serialize)]
pub struct LoadBalancerInsert {
    pub mongo_image_reference:ObjectId, //mongo_image_reference
    pub head: usize,
    pub behavior: String,
}