use super::{
    containers::ServiceContainerColumns,
    orchestrators::OrchestratorColumns,
    port_pool::{get_port_pool, PortPoolColumns},
    tables::ETables,
};
use std::{fmt::Display, sync::Arc};
use tokio_postgres::{types::Type, Error, Row};
use uuid::Uuid;

pub enum ContainerInstancePortPoolJunctionColumns {
    ID,
    PORT_POOL_FK,
    IN_USE,
    UUID,
}

impl Display for ContainerInstancePortPoolJunctionColumns {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::ID => write!(f, "id"),
            Self::PORT_POOL_FK => write!(f, "port_pool_fk"),
            Self::IN_USE => write!(f, "in_use"),
            Self::UUID => write!(f, "uuid"),
        }
    }
}
impl ContainerInstancePortPoolJunctionColumns {
    pub fn as_str(&self) -> &str {
        match *self {
            Self::ID => "id",
            Self::PORT_POOL_FK => "port_pool_fk",
            Self::IN_USE => "in_use",
            Self::UUID => "uuid",
        }
    }
}

pub async fn allocate_port(
    postgres_client: &tokio_postgres::Client,
    orchestrator_public_uuid: &Uuid,
) -> Result<Vec<Row>, Error> {
    let insert_result = postgres_client
        .query_typed(
            format!(
                "
        INSERT INTO {cippj} ({cippj_ppfk}, {cippj_in_use}) values (
            (SELECT pp.{pp_id}
            FROM {pp} pp
            LEFT JOIN {o_table} on pp.{pp_o_fk} = {o_table}.{o_table_id}
            WHERE
                {o_table}.{o_table_pub_uuid} = $1
                AND
                NOT EXISTS(
                    SELECT 1 FROM {cippj} cippj
                    WHERE pp.{pp_id} = cippj.{cippj_ppfk}
                    AND
                    cippj.{cippj_in_use}
                    LIMIT 1 FOR UPDATE
                )
            LIMIT 1 FOR UPDATE),
            TRUE)
        RETURNING *
    ",
                cippj = ETables::CONTAINER_INSTANCE_PORT_POOL_JUNCTION,
                cippj_ppfk = ContainerInstancePortPoolJunctionColumns::PORT_POOL_FK,
                cippj_in_use = ContainerInstancePortPoolJunctionColumns::IN_USE,
                pp = ETables::PORT_POOL,
                pp_id = PortPoolColumns::ID,
                pp_o_fk = PortPoolColumns::ORCHESTRATOR_FK,
                o_table = ETables::ORCHESTRATORS,
                o_table_id = OrchestratorColumns::ID,
                o_table_pub_uuid = OrchestratorColumns::PUBLIC_UUID,
            )
            .as_str(),
            &[(orchestrator_public_uuid, Type::UUID)],
        )
        .await;
    insert_result
}

/// A function call to prepare a port and Uuid for container creation
pub async fn prepare_port_allocation(
    postgres_client: Arc<tokio_postgres::Client>,
    orchestrator_public_uuid: &Uuid,
) -> Result<(i32, Arc<i32>, Uuid), Error> {
    //allocate a port if available
    let allocate_result = allocate_port(postgres_client.as_ref(), &orchestrator_public_uuid).await;
    match allocate_result {
        Ok(rows) => {
            let row = &rows[0];
            let port_pool_fk = row
                .get::<&str, i32>(ContainerInstancePortPoolJunctionColumns::PORT_POOL_FK.as_str());
            let generated_uuid =
                row.get::<&str, Uuid>(ContainerInstancePortPoolJunctionColumns::UUID.as_str());
            let cippj_id =
                row.get::<&str, i32>(ContainerInstancePortPoolJunctionColumns::ID.as_str());
            //give then port_pool_fk we can query it back.
            let port_pool_query_result = get_port_pool(&port_pool_fk, &postgres_client).await;
            match port_pool_query_result {
                Ok(rows) => {
                    //expecting 1  row only
                    let row = &rows[0];
                    let allocated_port: i32 = row.get::<&str, i32>("pp_port");
                    return Ok((cippj_id, Arc::new(allocated_port), generated_uuid));
                }
                Err(err) => {
                    tracing::error!("[ERROR] Port Pool Allocation Error {}", err);
                    return Err(err);
                }
            }
        }
        Err(err) => {
            tracing::error!("[ERROR] Port Pool Allocation Error {}", err);
            return Err(err);
        }
    };
}

pub fn deallocate_port(cippj_ids: Vec<i32>, postgres_client: Arc<tokio_postgres::Client>) {
    tokio::spawn(async move {
        //the problem is that when this fails we might not be able to correct it anymore since it is a new task
        // maybe use a background process to clean it up instead
        let _update_result = postgres_client
            .query_typed(
                format!(
                    "
                UPDATE {cippj_table} SET {cippj_in_use} = false
                WHERE {cippj_ids} = ANY($1)
            ",
                    cippj_table = ETables::CONTAINER_INSTANCE_PORT_POOL_JUNCTION,
                    cippj_in_use = ContainerInstancePortPoolJunctionColumns::IN_USE,
                    cippj_ids = ContainerInstancePortPoolJunctionColumns::ID,
                )
                .as_str(),
                &[(&cippj_ids, Type::INT4_ARRAY)],
            )
            .await;
    });
}

pub async fn deallocate_port_by_container_id(
    container_ids: Vec<String>,
    postgres_client: Arc<tokio_postgres::Client>,
) {
    let _update_result = postgres_client.query_typed(
        format!("
                UPDATE {cippj_table} set {cippj_in_use} = false
                WHERE {cippj_id} IN (SELECT {c_cippjfk} FROM {containers_table} WHERE {c_dci} = ANY($1)) 
            ",
            cippj_table = ETables::CONTAINER_INSTANCE_PORT_POOL_JUNCTION,
            cippj_in_use = ContainerInstancePortPoolJunctionColumns::IN_USE,
            cippj_id = ContainerInstancePortPoolJunctionColumns::ID,
            c_cippjfk = ServiceContainerColumns::CONTAINER_INSTANCE_PORT_POOL_JUNCTION_FK,
            containers_table = ETables::SERVICE_CONTAINER,
            c_dci = ServiceContainerColumns::DOCKER_CONTAINER_ID
        ).as_str()
        , &[(&container_ids, Type::VARCHAR_ARRAY)]).await;
}
