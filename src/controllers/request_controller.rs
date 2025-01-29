use std::sync::Arc;

use tokio_postgres::types::Type;
use uuid::Uuid;

use crate::{
    db::{
        requests::{insert_request_acceptance_query, ServiceRequestColumns},
        tables::TABLES,
    },
    utils::{orchestrator_utils::ORCHESTRATOR_INSTANCE_ID, postgres_utils::POSTGRES_CLIENT},
};

pub async fn record_service_request_acceptance(
    path: Arc<String>,
    method: Arc<String>,
    image_fk: Arc<i32>,
) -> Arc<Uuid> {
    let request_uuid = Arc::new(Uuid::new_v4());
    let insert_result =
        insert_request_acceptance_query(path, method, image_fk, request_uuid.clone()).await;
    if insert_result.is_err() {
        tracing::warn!("Request details insert failed");
    }
    //create the query to send the request ehre to db
    request_uuid
}
