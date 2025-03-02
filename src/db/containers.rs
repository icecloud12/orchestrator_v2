use std::{fmt::Display, sync::Arc};
use tokio_postgres::{types::Type, Error, Row};


use super::tables::ETables;

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

pub async fn container_insert_query(
    docker_container_id: &String,
    cippj_id: &i32,
    postgres_client: Arc<tokio_postgres::Client>
) -> Result<Vec<Row>, Error> {
    let client = postgres_client;
    let insert_result = client
        .query_typed(
            format!(
                "INSERT INTO {containers_table} ({dcid}, {cippj_fk})
                VALUES ($1, $2)
                RETURNING *
            ",
                containers_table = ETables::SERVICE_CONTAINER,
                dcid = ServiceContainerColumns::DOCKER_CONTAINER_ID,
                cippj_fk = ServiceContainerColumns::CONTAINER_INSTANCE_PORT_POOL_JUNCTION_FK
            )
            .as_str(),
            &[(docker_container_id, Type::TEXT), (cippj_id, Type::INT4)],
        )
        .await;
    insert_result
}
