use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
#[derive(Deserialize, Serialize, Debug)]
pub struct ServiceLoadBalancerContainerJunction {
	pub _id: ObjectId,
	pub load_balancer_id: ObjectId,
	pub container_id: ObjectId
}
#[derive(Deserialize, Serialize, Debug)]
pub struct ServiceLoadBalancerContainerJunctionInsertBody {
	pub load_balancer_id: ObjectId,
	pub container_id: ObjectId
}