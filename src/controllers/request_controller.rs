use std::sync::Arc;

use tokio_postgres::types::Type;
use uuid::Uuid;

use crate::{db::{requests::ServiceRequestColumns, tables::TABLES}, utils::{orchestrator_utils::ORCHESTRATOR_INSTANCE_ID, postgres_utils::POSTGRES_CLIENT}};

pub fn record_service_request_acceptance(path: Arc<String>, method: Arc<String>, image_fk: Arc<i32>) -> Arc<Uuid>{
    let request_uuid = Arc::new(Uuid::new_v4());
    let uuid_clone = request_uuid.clone();    //spawn a task here so the query insert does not "block" the task as it is considered a different task
    tokio::spawn(async move {
        let client = POSTGRES_CLIENT.get().unwrap();
        let uuid = request_uuid.clone();
        let _query_result = client
            .query_typed(
                format!(
                    "INSERT INTO requests ({path}, {uuid}, {request_intercepted_timestamp}, {method}, {orchestrator_instance}, {image_fk}) values ($1, $2, NOW(), $3, $4, $5);",
                    path = ServiceRequestColumns::PATH.to_string(),
                    uuid = ServiceRequestColumns::UUID.to_string(),
                    request_intercepted_timestamp= ServiceRequestColumns::REQUEST_TIME.to_string(), 
                    method = ServiceRequestColumns::METHOD.to_string(),
                    orchestrator_instance = ServiceRequestColumns::ORCHESTRATOR_INSTANCE_FK.to_string(),
                    image_fk = ServiceRequestColumns::IMAGE_FK.to_string())
                .as_str(),
                &[
                    (path.as_ref(), Type::TEXT), 
                    (uuid.as_ref(), Type::UUID), 
                    (method.as_ref(), Type::TEXT),
                    (ORCHESTRATOR_INSTANCE_ID.get().unwrap(), Type::INT4),
                    (image_fk.as_ref(), Type::INT4)
                    
                
            ]).await;
        if let Err(err) = _query_result{
            tracing::warn!("Error in inserting request interception: {}", err.to_string());
        }
    });
    //create the query to send the request ehre to db
    uuid_clone
}

pub async fn record_service_request_responded(uuid: Arc<Uuid>, container_id: &i32, status_code: i32){
    let client = POSTGRES_CLIENT.get().unwrap();
    let update_result = client.query_typed(
        format!("UPDATE {request_table} 
            SET {response_time_col} = NOW(), {container_ref} = $1, {status_code} = $2   
            WHERE {uuid_col} = $3", 
            request_table = TABLES::SERVICE_REQUEST.to_string(),
            response_time_col = ServiceRequestColumns::RESPONSE_TIME.to_string(),
            container_ref = ServiceRequestColumns::CONTAINER_FK.to_string(),
            status_code = ServiceRequestColumns::STATUS_CODE.to_string(),
            uuid_col = ServiceRequestColumns::UUID.to_string()
        ).as_str()
        , &[
            (container_id, Type::INT4),
            (&status_code, Type::INT4),
            (uuid.as_ref(), Type::UUID)
    ]).await;
    if let Err(err) = update_result {
        tracing::error!("Error in updating request response: {}", err.to_string());
    }
     
}
