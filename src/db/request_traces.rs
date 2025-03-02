use crate::data::request_traces_types::ERequestTraceTypes;
use chrono::{DateTime, Utc};
use std::{fmt::Display, sync::Arc};
use tokio_postgres::types::Type;
use uuid::Uuid;

use super::{requests::EServiceRequestColumns, tables::ETables};

pub enum ERequestTracesColumns {
    ID,
    REQUEST_UUID,
    REQUEST_TRACE_TYPES_FK,
    CONTAINER_ID_FK,
    TIMESTAMP,
}

impl Display for ERequestTracesColumns {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::ID => write!(f, "id"),
            Self::REQUEST_UUID => write!(f, "request_uuid"),
            Self::REQUEST_TRACE_TYPES_FK => write!(f, "request_trace_types_fk"),
            Self::CONTAINER_ID_FK => write!(f, "container_id_fk"),
            Self::TIMESTAMP => write!(f, "timestamp"),
        }
    }
}
// This function must be called and NOT spawned as a different task from the tcp_stream it should be matched with logically
// the problem with this function is that this should be allowed to fail;
pub async fn insert_request_acceptance_query(
    path: Arc<String>,
    method: Arc<String>,
    uuid: Arc<Uuid>,
    postgres_client: Arc<tokio_postgres::Client>,
    orchestrator_instance_id: Arc<i32>,
) {
    let uuid_cloned = uuid.clone();
    let dt = Utc::now();
    let postgres_client_clone = postgres_client.clone();
    // create the service request entry
    tokio::spawn(async move {
        let future1 = tokio::task::spawn(async move {
            let ret = postgres_client
                .query_typed(
                    format!(
                        "
                INSERT INTO {sr_table}
                ({sr_path}, {sr_uuid}, {sr_method}, {sr_orchestrator_instance})
                VALUES ($1, $2, $3, $4, $5);
            ",
                        sr_table = ETables::SERVICE_REQUEST,
                        sr_path = EServiceRequestColumns::PATH,
                        sr_uuid = EServiceRequestColumns::UUID,
                        sr_method = EServiceRequestColumns::METHOD,
                        sr_orchestrator_instance = EServiceRequestColumns::ORCHESTRATOR_INSTANCE_FK,
                    )
                    .as_str(),
                    &[
                        (path.as_ref(), Type::TEXT),
                        (uuid_cloned.as_ref(), Type::UUID),
                        (method.as_ref(), Type::TEXT),
                        (orchestrator_instance_id.as_ref(), Type::INT4),
                    ],
                )
                .await;
            ret
        });
        let future2 = tokio::task::spawn(async move {
            insert_request_trace(
                uuid,
                ERequestTraceTypes::INTERCEPTED as i32,
                None,
                dt,
                postgres_client_clone,
            )
            .await;
        });
        let (_ret1, _ret2) = tokio::join!(future1, future2);
    });
}
pub async fn insert_request_trace(
    request_uuid: Arc<Uuid>,
    request_trace_type_fk: i32,
    container_id_fk: Option<&i32>,
    timestamp: DateTime<Utc>,
    postgres_client: Arc<tokio_postgres::Client>,
) {
    let _insert_request_trace_result = postgres_client
        .query_typed(
            format!(
                "
                INSERT INTO {rt_table} (
                    {rt_uuid},
                    {rt_ttfk},
                    {rt_timestamp},
                    {rt_container_id_fk}
                    ) VALUES ($1, $2, $3, $4)
            ",
                rt_table = ETables::REQUEST_TRACES,
                rt_uuid = ERequestTracesColumns::REQUEST_UUID,
                rt_ttfk = ERequestTracesColumns::REQUEST_TRACE_TYPES_FK,
                rt_timestamp = ERequestTracesColumns::TIMESTAMP,
                rt_container_id_fk = ERequestTracesColumns::CONTAINER_ID_FK
            )
            .as_str(),
            &[
                (request_uuid.as_ref(), Type::UUID),
                (&request_trace_type_fk, Type::INT4),
                (&timestamp, Type::TIMESTAMPTZ),
                (&container_id_fk, Type::INT4),
            ],
        )
        .await;
}
///
/// This function is called at the end of the request trace so no need to create a task for it to query independently
pub async fn insert_finalized_trace(
    request_uuid: Arc<Uuid>,
    status_code: i32,
    postgres_client: Arc<tokio_postgres::Client>,
) {
    let dt = Utc::now();
    //update service request table
    //there is nothing we can do if it fails
    let _update_query = postgres_client
        .query_typed(
            format!(
                "
            UPDATE {sr_table}
            SET {sr_status_code} = $1
            WHERE {sr_uuid} = $2
        ",
                sr_table = ETables::SERVICE_REQUEST,
                sr_status_code = EServiceRequestColumns::STATUS_CODE,
                sr_uuid = EServiceRequestColumns::UUID
            )
            .as_str(),
            &[
                (&status_code, Type::INT4),
                (request_uuid.as_ref(), Type::UUID),
            ],
        )
        .await;
    insert_request_trace(
        request_uuid,
        ERequestTraceTypes::FINALIZED as i32,
        None,
        dt,
        postgres_client,
    )
    .await;
}

pub fn insert_forward_trace(request_uuid: Arc<Uuid>, container_id: Arc<i32>, postgres_client: Arc<tokio_postgres::Client>) {
    let dt = Utc::now();
    tokio::spawn(async move {
        insert_request_trace(
            request_uuid,
            ERequestTraceTypes::FORWARDED as i32,
            Some(container_id.as_ref()),
            dt,
            postgres_client
        )
        .await;
    });
}

pub fn insert_failed_trace(request_uuid: Arc<Uuid>, postgres_client: Arc<tokio_postgres::Client>) {
    //create dt outside tokio spawn so it uses the time spawn is created and not when the spawn is executed
    let dt = Utc::now();
    tokio::spawn(async move {
        insert_request_trace(request_uuid, ERequestTraceTypes::FAILED as i32, None, dt, postgres_client).await;
    });
}

pub fn insert_returned_trace(request_uuid: Arc<Uuid>, postgres_client: Arc<tokio_postgres::Client>) {
    let dt = Utc::now();
    tokio::spawn(async move {
        insert_request_trace(request_uuid, ERequestTraceTypes::RETURNED as i32, None, dt, postgres_client).await;
    });
}
