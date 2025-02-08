use crate::{
    data::request_traces_types::ERequestTraceTypes,
    utils::{orchestrator_utils::ORCHESTRATOR_INSTANCE_ID, postgres_utils::POSTGRES_CLIENT},
};
use chrono::{DateTime, Utc};
use std::{arch::x86_64::_mm_insert_epi16, fmt::Display, sync::Arc};
use tokio_postgres::{types::Type, GenericClient};
use uuid::Uuid;

use super::{requests::EServiceRequestColumns, tables::TABLES};

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
impl ERequestTracesColumns {
    pub fn as_str(&self) -> &str {
        match *self {
            Self::ID => "id",
            Self::REQUEST_UUID => "request_uuid",
            Self::REQUEST_TRACE_TYPES_FK => "request_trace_types_fk",
            Self::CONTAINER_ID_FK => "container_id_fk",
            Self::TIMESTAMP => "timestamp",
        }
    }
}
// This function must be called and NOT spawned as a different task from the tcp_stream it should be matched with logically
// the problem with this function is that this should be allowed to fail;
pub async fn insert_request_acceptance_query(
    path: Arc<String>,
    method: Arc<String>,
    image_fk: Arc<i32>,
    uuid: Arc<Uuid>,
) {
    let uuid_cloned = uuid.clone();
    let dt = Utc::now();
    // create the service request entry
    tokio::spawn(async move {
        let client = POSTGRES_CLIENT.get().unwrap();
        let _insert_service_request_result = client
            .query_typed(
                format!(
                    "
                INSERT INTO {sr_table}
                ({sr_path}, {sr_uuid}, {sr_method}, {sr_orchestrator_instance}, {sr_image_fk} )
                VALUES ($1, $2, $3, $4, $5);
            ",
                    sr_table = TABLES::SERVICE_REQUEST.as_str(),
                    sr_path = EServiceRequestColumns::PATH,
                    sr_uuid = EServiceRequestColumns::UUID,
                    sr_method = EServiceRequestColumns::METHOD,
                    sr_orchestrator_instance = EServiceRequestColumns::ORCHESTRATOR_INSTANCE_FK,
                    sr_image_fk = EServiceRequestColumns::IMAGE_FK
                )
                .as_str(),
                &[
                    (path.as_ref(), Type::TEXT),
                    (uuid_cloned.as_ref(), Type::UUID),
                    (method.as_ref(), Type::TEXT),
                    (ORCHESTRATOR_INSTANCE_ID.get().unwrap(), Type::INT4),
                    (image_fk.as_ref(), Type::INT4),
                ],
            )
            .await;
        insert_request_trace(
            uuid_cloned,
            ERequestTraceTypes::INTERCEPTED as i32,
            None,
            dt,
        )
        .await;
    });
}
pub async fn insert_request_trace(
    request_uuid: Arc<Uuid>,
    request_trace_type_fk: i32,
    container_id_fk: Option<&i32>,
    timestamp: DateTime<Utc>,
) {
    let client = POSTGRES_CLIENT.get().unwrap();
    let _insert_request_trace_result = client
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
                rt_table = TABLES::REQUEST_TRACES,
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
pub async fn insert_finalized_trace(request_uuid: Arc<Uuid>, status_code: i32) {
    let client = POSTGRES_CLIENT.get().unwrap();
    let dt = Utc::now();
    //update service request table
    //there is nothing we can do if it fails
    let _update_query = client
        .query_typed(
            format!(
                "
            UPDATE {sr_table}
            SET {sr_status_code} = $1
            WHERE {sr_uuid} = $2
        ",
                sr_table = TABLES::SERVICE_REQUEST,
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
    insert_request_trace(request_uuid, ERequestTraceTypes::FINALIZED as i32, None, dt).await;
}

pub fn insert_forward_trace(request_uuid: Arc<Uuid>, container_id: Arc<i32>) {
    tokio::spawn(async move {
        insert_request_trace(
            request_uuid,
            ERequestTraceTypes::FORWARDED as i32,
            Some(container_id.as_ref()),
            Utc::now(),
        )
        .await;
    });
}
