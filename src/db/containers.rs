use std::fmt::Display;
use tokio_postgres::{types::Type, Error, Row};
use uuid::Uuid;

use crate::utils::postgres_utils::POSTGRES_CLIENT;

use super::tables::TABLES;

pub enum ServiceContainerColumns {
    ID,
    DOCKER_CONTAINER_ID,
}
impl Display for ServiceContainerColumns {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::ID => write!(f, "id"),
            Self::DOCKER_CONTAINER_ID => write!(f, "docker_container_id"),
        }
    }
}
// pub async fn container_insert_query(
//     docker_container_id: &String,
//     container_public_port: &i32,
//     container_uuid: &Uuid,
// ) -> Result<Vec<Row>, Error> {
//     let client = POSTGRES_CLIENT.get().unwrap();

//     client.query_typed
// 				(format!(
// 				    "INSERT INTO {container_table} ({docker_container_id_column}, {public_port_column}, {uuid_column}) VALUES ($1, $2, $3) RETURNING id;",
// 				    container_table = TABLES::SERVICE_CONTAINER.to_string(),
// 				    docker_container_id_column = ServiceContainerColumns::DOCKER_CONTAINER_ID.to_string(),
// 				    public_port_column = ServiceContainerColumns::PUBLIC_PORT.to_string(),
// 				    uuid_column = ServiceContainerColumns::UUID.to_string()
// 				).as_str(),
// 				&[
// 					(&docker_container_id, Type::TEXT),
// 					(&container_public_port, Type::INT4),
// 					(&container_uuid, Type::UUID)
// 				]).await
// }
