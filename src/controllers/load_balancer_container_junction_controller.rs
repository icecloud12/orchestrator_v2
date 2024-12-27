use mongodb::bson::{doc, oid::ObjectId};

use crate::{models::service_load_balancer_container_junction_model::ServiceLoadBalancerContainerJunctionInsertBody, utils::mongodb_utils::MongoCollections};

pub async fn create(load_balancer_id: &ObjectId, container_id: &ObjectId) -> Result<(), String> {
	let _insert_result = MongoCollections::LoadBalancerContainerJunction.as_collection::<ServiceLoadBalancerContainerJunctionInsertBody>().insert_one(ServiceLoadBalancerContainerJunctionInsertBody {
		load_balancer_id: *load_balancer_id,
		container_id: *container_id,
	}).await;

	Ok(())
}