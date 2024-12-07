use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use serde_json::Number;

#[derive(Deserialize, Serialize, Debug)]
pub struct ServiceRoute {
	_id: ObjectId,
	mongo_image: ObjectId, // a reference to the image,
	address: String, //prefix of the service
	exposed_port: String,// port where the service within the container would listen,
	segments: usize
}