use std::{fmt::Display, sync::Arc};
use tokio_postgres::types::Type;
use uuid::Uuid;
use chrono::Utc;
use crate::{data::request_traces_types::ERequestTraceTypes, utils::{orchestrator_utils::ORCHESTRATOR_INSTANCE_ID, postgres_utils::POSTGRES_CLIENT}};

use super::{requests::ServiceRequestColumns, tables::TABLES};

pub enum ERequestTracesColumns {
    ID,
    REQUEST_UUID,
    REQUEST_TRACE_TYPES_FK,
    CONTAINER_ID_FK,
    TIMESTAMP
}

impl Display for ERequestTracesColumns {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::ID => write!(f, "id"),
            Self::REQUEST_UUID => write!(f, "request_uuid"),
            Self::REQUEST_TRACE_TYPES_FK => write!(f, "request_trace_types_fk"),
            Self::CONTAINER_ID_FK => write!(f, "container_id_fk"),
            Self::TIMESTAMP => write!(f, "timestamp")
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
            Self::TIMESTAMP => "timestamp"
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
){
    let uuid_cloned = uuid.clone();
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
                    sr_path = ServiceRequestColumns::PATH,
                    sr_uuid = ServiceRequestColumns::UUID,
                    sr_method = ServiceRequestColumns::METHOD,
                    sr_orchestrator_instance = ServiceRequestColumns::ORCHESTRATOR_INSTANCE_FK,
                    sr_image_fk = ServiceRequestColumns::IMAGE_FK
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
        let dt = Utc::now();
        let _insert_request_trace_result = client.query_typed(
        format!("
                INSERT INTO {rt_table} ({rt_uuid}, {rt_ttfk}, {rt_timestamp}) VALUES ($1, $2, $3)
            ",
            rt_table = TABLES::REQUEST_TRACES,
            rt_uuid = ERequestTracesColumns::REQUEST_UUID,
            rt_ttfk = ERequestTracesColumns::REQUEST_TRACE_TYPES_FK,
            rt_timestamp = ERequestTracesColumns::TIMESTAMP
        ).as_str(),
        &[
            (uuid_cloned.as_ref(), Type::UUID),
            (&(ERequestTraceTypes::INTERCEPTED as i32), Type::INT4),
            (&dt, Type::TIMESTAMPTZ)
        ]).await;
    });
}
