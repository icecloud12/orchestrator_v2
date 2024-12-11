use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceContainer {
	pub _id: ObjectId, 
	pub container_id: String, // docker internal id of the docker container
	pub public_port: usize, //
}
#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceContainerBody{
	pub container_id: String, // docker internal id of the docker container
	pub public_port: usize, //
}

pub type DockerImageId = String;