use std::sync::Arc;
use uuid::Uuid;

use crate::db::request_traces::insert_request_acceptance_query;

pub async fn record_service_request_acceptance(
    path: Arc<String>,
    method: Arc<String>,
) -> Arc<Uuid> {
    let request_uuid = Arc::new(Uuid::new_v4());
    let _request_uuid = request_uuid.clone();
    tokio::spawn(async move {
        insert_request_acceptance_query(path, method, _request_uuid.clone()).await;
    });
    //create the query to send the request ehre to db
    request_uuid
}
