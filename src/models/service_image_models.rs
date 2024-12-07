use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceImage {
	_id: ObjectId,
	docker_image_id: String //docker internal id of the docker image
}

