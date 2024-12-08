use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceImage {
	pub _id: ObjectId,
	pub docker_image_id: String //docker internal id of the docker image
}

