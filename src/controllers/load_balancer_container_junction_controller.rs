use crate::db::load_balancer_container_junctions::insert_lbcj;
use std::sync::Arc;
pub fn create(load_balancer_id: Arc<i32>, container_id: Arc<i32>) {
    insert_lbcj(load_balancer_id, container_id);
}
