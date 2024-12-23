use std::collections::HashMap;

use custom_tcp_listener::models::types::Request;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;

use crate::controllers::{
    container_controller, load_balancer_container_junction,
    load_balancer_controller::{ELoadBalancerBehavior, ELoadBalancerMode, AWAITED_CONTAINERS},
};

use super::service_container_models::ServiceContainer;

#[derive(Debug)]
//this struct represents the load balancer of the current program
pub struct ServiceLoadBalancer {
    pub _id: ObjectId, //mongo_db_load_balancer_instance,
    pub docker_image_id: String,
    pub exposed_port: String,
    pub address: String,
    pub head: usize, //current head pointer of the load_balancer
    pub behavior: ELoadBalancerBehavior,
    pub mode: ELoadBalancerMode,
    pub containers: Vec<ServiceContainer>, //docker_container_id_instances
    pub awaited_containers: HashMap<String, ServiceContainer>, //docker_containers not pushed to the active container vector
    pub validated: bool, //initially false to let the program know if the containers are checke,
    pub request_queue: Vec<(Request, TcpStream)>,
}

impl ServiceLoadBalancer {
    ///the container array size differ depending on:
    /// 1. if it's newly created
    /// 2. the recorded load_balancer in the database have/have no entries
    pub async fn next_container(&mut self) -> Result<usize, ()> {
        let containers = &self.containers;

        let container_len = containers.len();
        let ret: Result<usize, ()> = if container_len == 0 {
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
            let public_port = containers.get(*head).unwrap().public_port.clone();
            Ok(public_port)
        };
        ret
    }

    pub async fn create_container(&mut self) -> Result<ServiceContainer, String> {
        let col =
            container_controller::create_container(&self.docker_image_id, &self.exposed_port).await;

        match col {
            Ok(service_container) => {
                let _ = load_balancer_container_junction::create(&self._id, &service_container._id)
                    .await;
                Ok(service_container)
            }
            Err(err) => Err(err),
        }
    }
    pub fn queue_request(&mut self, request: Request, tcp_stream: TcpStream) {
        self.request_queue.push((request, tcp_stream));
    }
    pub fn add_container(&mut self, container: ServiceContainer) {
        let containers = &mut self.containers;
        containers.push(container);
    }
    pub async fn queue_container(&mut self, container: ServiceContainer) {
        let mut await_container_lock = AWAITED_CONTAINERS.get().unwrap().lock().await;
        await_container_lock.insert(container.uuid.clone(), self.docker_image_id.clone());
        let awaited_containers = &mut self.awaited_containers;
        awaited_containers.insert(container.uuid.clone(), container);
    }
    pub fn empty_queue(&mut self) -> Vec<(Request, TcpStream)> {
        std::mem::take(&mut self.request_queue)
    }
}
//this struct represents the data from mongodb
//will be used to quickly restore the load-balancers from the old state just incase
//if the orchestrator is closed due to user intervention but docker isn't
//load from cache thingy-majig if that makes sense
#[derive(Deserialize, Serialize, Debug)]
pub struct LoadBalancerEntryAggregate {
    pub _id: ObjectId,
    pub mongo_image_reference: ObjectId, //mongo_image_reference
    pub head: usize,
    pub behavior: String,
    pub containers: Vec<ServiceContainer>,
}

#[derive(Deserialize, Serialize)]
pub struct LoadBalancerEntry {
    pub _id: ObjectId,
    pub mongo_image_reference: ObjectId, //mongo_image_reference
    pub head: usize,
    pub behavior: String,
}

///struct for insert purposes only
#[derive(Serialize)]
pub struct LoadBalancerInsert {
    pub mongo_image_reference: ObjectId, //mongo_image_reference
    pub head: usize,
    pub behavior: String,
}
