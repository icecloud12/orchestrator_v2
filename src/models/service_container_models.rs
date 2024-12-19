use bollard::container::StartContainerOptions;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::utils::docker_utils::DOCKER;

#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceContainer {
	pub _id: ObjectId, 
	pub container_id: String, // docker internal id of the docker container
	pub public_port: usize, //
	pub uuid: String
}
#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceContainerBody{
	pub container_id: String, // docker internal id of the docker container
	pub public_port: usize, //
	pub uuid: String
}

pub type DockerImageId = String;

impl ServiceContainer {
	pub async fn start_container(&self) -> Result<(), String> {
		let docker = DOCKER.get().unwrap();
		let start_result = docker.start_container(&self.container_id, None::<StartContainerOptions<String>>).await;
		match  start_result{
			Ok(_) => {Ok(())},
			Err(err) => {Err(err.to_string())}
		}	
	}
}