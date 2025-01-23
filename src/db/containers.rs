use std::{fmt::Display, sync::Arc};
use tokio_postgres::{types::Type, Error, Row};
use uuid::Uuid;

use crate::utils::postgres_utils::POSTGRES_CLIENT;

use super::{
    container_instance_port_pool_junction::{
        allocate_port, ContainerInstancePortPoolJunctionColumns,
    },
    port_pool::{get_port_pool, PortPoolColumns},
    tables::TABLES,
};

pub enum ServiceContainerColumns {
    ID,
    DOCKER_CONTAINER_ID,
    CONTAINER_INSTANCE_PORT_POOL_JUNCTION_FK,
}
impl Display for ServiceContainerColumns {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::ID => write!(f, "id"),
            Self::DOCKER_CONTAINER_ID => write!(f, "docker_container_id"),
            Self::CONTAINER_INSTANCE_PORT_POOL_JUNCTION_FK => {
                write!(f, "container_instance_port_pool_junction_fk")
            }
        }
    }
}

impl ServiceContainerColumns {
    pub fn as_str(&self) -> &str {
        match *self {
            Self::ID => "id",
            Self::DOCKER_CONTAINER_ID => "docker_container_id",
            Self::CONTAINER_INSTANCE_PORT_POOL_JUNCTION_FK => {
                "container_instance_port_pool_junction_fk"
            }
        }
    }
}
/// A function call to prepare a port and Uuid for container creation
pub async fn prepare_port_allocation() -> Result<(i32, Arc<Uuid>), Error> {
    //allocate a port if available
    let allocate_result = allocate_port().await;
    match allocate_result {
        Ok(rows) => {
            let row = &rows[0];
            let port_pool_fk = row
                .get::<&str, i32>(ContainerInstancePortPoolJunctionColumns::PORT_POOL_FK.as_str());
            let generated_uuid =
                row.get::<&str, Uuid>(ContainerInstancePortPoolJunctionColumns::UUID.as_str());
            //give then port_pool_fk we can query it back.
            let port_pool_query_result = get_port_pool(&port_pool_fk).await;
            match port_pool_query_result {
                Ok(rows) => {
                    //expecting 1  row only
                    let row = &rows[0];
                    let allocated_port: i32 = row.get::<&str, i32>(PortPoolColumns::PORT.as_str());
                    return Ok((allocated_port, Arc::new(generated_uuid)));
                }
                Err(err) => return Err(err),
            }
        }
        Err(err) => return Err(err),
    };
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
