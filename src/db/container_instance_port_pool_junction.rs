use super::{
    orchestrators::OrchestratorColumns,
    port_pool::{get_port_pool, PortPoolColumns},
    tables::TABLES,
};
use crate::utils::{orchestrator_utils::ORCHESTRATOR_PUBLIC_UUID, postgres_utils::POSTGRES_CLIENT};
use std::{fmt::Display, sync::Arc};
use tokio_postgres::{types::Type, Error, GenericClient, Row};
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

pub async fn allocate_port() -> Result<Vec<Row>, Error> {
    let client = POSTGRES_CLIENT.get().unwrap();
    let insert_result = client
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
                cippj = TABLES::CONTAINER_INSTANCE_PORT_POOL_JUNCTION.as_str(),
                cippj_ppfk = ContainerInstancePortPoolJunctionColumns::PORT_POOL_FK.as_str(),
                cippj_in_use = ContainerInstancePortPoolJunctionColumns::IN_USE.as_str(),
                pp = TABLES::PORT_POOL.as_str(),
                pp_id = PortPoolColumns::ID.as_str(),
                pp_o_fk = PortPoolColumns::ORCHESTRATOR_FK.as_str(),
                o_table = TABLES::ORCHESTRATORS.as_str(),
                o_table_id = OrchestratorColumns::ID.as_str(),
                o_table_pub_uuid = OrchestratorColumns::PUBLIC_UUID.as_str(),
            )
            .as_str(),
            &[(ORCHESTRATOR_PUBLIC_UUID.get().unwrap(), Type::UUID)],
        )
        .await;
    insert_result
}

/// A function call to prepare a port and Uuid for container creation
pub async fn prepare_port_allocation() -> Result<(i32, Arc<i32>, Uuid), Error> {
    //allocate a port if available
    let allocate_result = allocate_port().await;
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
            let port_pool_query_result = get_port_pool(&port_pool_fk).await;
            match port_pool_query_result {
                Ok(rows) => {
                    //expecting 1  row only
                    let row = &rows[0];
                    let allocated_port: i32 = row.get::<&str, i32>("pp_port");
                    return Ok((cippj_id, Arc::new(allocated_port), generated_uuid));
                }
                Err(err) => {
                    tracing::error!("get_port_pool error {}", err);
                    return Err(err);
                }
            }
        }
        Err(err) => {
            tracing::error!("Failed to allocate port:  {}", err);
            return Err(err);
        }
    };
}

pub fn deallocate_port(cippj_ids: Vec<i32>) {
    tokio::spawn(async move {
        let client = POSTGRES_CLIENT.get().unwrap();
        //the problem is that when this fails we might now be able to correct it anymore
        // maybe use a background process to clean it up instead
        let _update_result = client
            .query_typed(
                format!(
                    "
                UPDATE {cippj_table} SET {cippj_in_use} = false
                WHERE {cippj_ids} in $1
            ",
                    cippj_table = TABLES::CONTAINER_INSTANCE_PORT_POOL_JUNCTION,
                    cippj_in_use = ContainerInstancePortPoolJunctionColumns::IN_USE,
                    cippj_ids = ContainerInstancePortPoolJunctionColumns::ID,
                )
                .as_str(),
                &[(&cippj_ids, Type::INT4_ARRAY)],
            )
            .await;
    });
}
