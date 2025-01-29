use std::{fmt::Display, sync::Arc};

use tokio_postgres::{types::Type, Error, Row};
use uuid::Uuid;

use crate::utils::{orchestrator_utils::ORCHESTRATOR_INSTANCE_ID, postgres_utils::POSTGRES_CLIENT};

use super::tables::TABLES;

pub enum ServiceRequestColumns {
    ID,
    UUID,
    PATH,
    METHOD,
    IMAGE_FK,
    ORCHESTRATOR_INSTANCE_FK,
    ACCEPTED_TIME,
    FORWARDED_TIME,
    RETURNED_TIME,
    REPLIED_TIME,
    STATUS_CODE,
    CONTAINER_FK,
}

impl Display for ServiceRequestColumns {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::ID => write!(f, "id"),
            Self::UUID => write!(f, "uuid"),
            Self::PATH => write!(f, "path"),
            Self::METHOD => write!(f, "method"),
            Self::IMAGE_FK => write!(f, "image_fk"),
            Self::ORCHESTRATOR_INSTANCE_FK => write!(f, "orchestrator_instance_fk"),
            Self::ACCEPTED_TIME => write!(f, "accepted_time"),
            Self::FORWARDED_TIME => write!(f, "forwarded_time"),
            Self::RETURNED_TIME => write!(f, "returned_time"),
            Self::REPLIED_TIME => write!(f, "replied_time"),
            Self::STATUS_CODE => write!(f, "status_code"),
            Self::CONTAINER_FK => write!(f, "container_fk"),
        }
    }
}
impl ServiceRequestColumns {
    pub fn as_str(&self) -> &str {
        match *self {
            Self::ID => "id",
            Self::UUID => "uuid",
            Self::PATH => "path",
            Self::METHOD => "method",
            Self::IMAGE_FK => "image_fk",
            Self::ORCHESTRATOR_INSTANCE_FK => "orchestrator_instance_fk",
            Self::ACCEPTED_TIME => "accepted_time",
            Self::FORWARDED_TIME => "forwarded_time",
            Self::RETURNED_TIME => "returned_time",
            Self::REPLIED_TIME => "replied_time",
            Self::STATUS_CODE => "status_code",
            Self::CONTAINER_FK => "container_fk",
        }
    }
}
///
/// This function must be called and NOT spawned as a different task from the tcp_stream it should be matched with logically
/// the problem with this function is that this should be allowed to fail;
pub async fn insert_request_acceptance_query(
    path: Arc<String>,
    method: Arc<String>,
    image_fk: Arc<i32>,
    uuid: Arc<Uuid>
    ) -> Result<Vec<Row>, Error> {
    let client = POSTGRES_CLIENT.get().unwrap();
    let insert_result = client
        .query_typed(
            format!(
                "
            INSERT INTO {sr_table}
            ({sr_path}, {sr_uuid}, {sr_accepted_time}, {sr_method}, {sr_orchestrator_instance}, {sr_image_fk} )
            VALUES ($1, $2, NOW(), $3, $4, $5);
        ",
            sr_table= TABLES::SERVICE_REQUEST.as_str(),
            sr_path= ServiceRequestColumns::PATH,
            sr_uuid= ServiceRequestColumns::UUID,
            sr_accepted_time= ServiceRequestColumns::ACCEPTED_TIME,
            sr_method = ServiceRequestColumns::METHOD,
            sr_orchestrator_instance = ServiceRequestColumns::ORCHESTRATOR_INSTANCE_FK,
            sr_image_fk = ServiceRequestColumns::IMAGE_FK                
            )
            .as_str(),
            &[
                (path.as_ref(), Type::TEXT),
                (uuid.as_ref(), Type::UUID),
                (method.as_ref(), Type::TEXT),
                (ORCHESTRATOR_INSTANCE_ID.get().unwrap(), Type::INT4),
                (image_fk.as_ref(), Type::INT4)
            ],
        )
        .await;
    insert_result
}

pub async fn update_request_responded(
    uuid: Arc<Uuid>,
    container_id: &i32,
    status_code: i32
){
    
    let client = POSTGRES_CLIENT.get().unwrap();
    let update_result = client
        .query_typed(
            format!(
                "UPDATE {sr_table} 
            SET {sr_replied_time} = NOW(), {sr_container_fk} = $1, {sr_status_code} = $2   
            WHERE {sr_uuid} = $3",
                sr_table = TABLES::SERVICE_REQUEST.to_string(),
                sr_replied_time = ServiceRequestColumns::REPLIED_TIME.to_string(),
                sr_container_fk = ServiceRequestColumns::CONTAINER_FK.to_string(),
                sr_status_code = ServiceRequestColumns::STATUS_CODE.to_string(),
                sr_uuid = ServiceRequestColumns::UUID.to_string()
            )
            .as_str(),
            &[
                (container_id, Type::INT4),
                (&status_code, Type::INT4),
                (uuid.as_ref(), Type::UUID),
            ],
        )
        .await;
    if let Err(err) = update_result {
        tracing::error!("Error in updating request response: {}", err.to_string());
    }
}
