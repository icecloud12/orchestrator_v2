use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct DockerContainer {
	_id: ObjectId, 
	mongo_image_reference: ObjectId, // references DockerImage._id
	container_id: String, // docker internal id of the docker container
	public_port: usize, //
}