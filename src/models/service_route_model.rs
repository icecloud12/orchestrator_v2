use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use serde_json::Number;

#[derive(Deserialize, Serialize, Debug)]
pub struct ServiceRoute {
	pub _id: ObjectId,
	pub mongo_image: ObjectId, // a reference to the image,
	pub address: String, //prefix of the service
	pub exposed_port: String,// port where the service within the container would listen,
	pub segments: usize
}