use std::fmt::Display;
pub enum EServiceRequestColumns {
    ID,
    UUID,
    PATH,
    METHOD,
    IMAGE_FK,
    ORCHESTRATOR_INSTANCE_FK,
    STATUS_CODE
}

impl Display for EServiceRequestColumns {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::ID => write!(f, "id"),
            Self::UUID => write!(f, "uuid"),
            Self::PATH => write!(f, "path"),
            Self::METHOD => write!(f, "method"),
            Self::IMAGE_FK => write!(f, "image_fk"),
            Self::ORCHESTRATOR_INSTANCE_FK => write!(f, "orchestrator_instance_fk"),
            Self::STATUS_CODE => write!(f, "status_code"),
        }
    }
}
impl EServiceRequestColumns {
    pub fn as_str(&self) -> &str {
        match *self {
            Self::ID => "id",
            Self::UUID => "uuid",
            Self::PATH => "path",
            Self::METHOD => "method",
            Self::IMAGE_FK => "image_fk",
            Self::ORCHESTRATOR_INSTANCE_FK => "orchestrator_instance_fk",
            Self::STATUS_CODE => "status_code",
        }
    }
}
//

// pub async fn update_request_responded(
//     uuid: Arc<Uuid>,
//     container_id: &i32,
//     status_code: i32
// ){
    
//     let client = POSTGRES_CLIENT.get().unwrap();
//     let update_result = client
//         .query_typed(
//             format!(
//                 "UPDATE {sr_table} 
//             SET {sr_replied_time} = NOW(), {sr_container_fk} = $1, {sr_status_code} = $2   
//             WHERE {sr_uuid} = $3",
//                 sr_table = TABLES::SERVICE_REQUEST.to_string(),
//                 sr_replied_time = ServiceRequestColumns::REPLIED_TIME.to_string(),
//                 sr_container_fk = ServiceRequestColumns::CONTAINER_FK.to_string(),
//                 sr_status_code = ServiceRequestColumns::STATUS_CODE.to_string(),
//                 sr_uuid = ServiceRequestColumns::UUID.to_string()
//             )
//             .as_str(),
//             &[
//                 (container_id, Type::INT4),
//                 (&status_code, Type::INT4),
//                 (uuid.as_ref(), Type::UUID),
//             ],
//         )
//         .await;
//     if let Err(err) = update_result {
//         tracing::error!("Error in updating request response: {}", err.to_string());
//     }
// }
