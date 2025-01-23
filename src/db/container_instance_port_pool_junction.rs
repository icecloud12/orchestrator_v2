use super::{orchestrators::OrchestratorColumns, port_pool::PortPoolColumns, tables::TABLES};
use crate::utils::{orchestrator_utils::ORCHESTRATOR_PUBLIC_UUID, postgres_utils::POSTGRES_CLIENT};
use std::fmt::Display;
use tokio_postgres::{Error, Row};

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
    let insert_result = client.query_typed(format!(
        "INSERT INTO {cippj} ({cippj_ppfk}, {cippj_in_use}) values (
            (SELECT pp.{pp_id} from {pp} as pp,
            LEFT JOIN {o_table} on pp.{pp_o_fk} = {o_table}.{o_table_id}
            WHERE
                {o_table}.{o_table_pub_uuid} = {pub_uuid} AND
                NOT EXISTS (SELECT 1 FROM {cippj} cippj where pp.{pp_id} = cippj.{cippj_ppfk} AND cippj.{cippj_in_use} LIMIT 1 FOR UPDATE)
            LIMIT 1 FOR UPDATE),
            TRUE)
        RETURNING *
        )",
         cippj = TABLES::CONTAINER_INSTANCE_PORT_POOL_JUNCTION.as_str(),
         cippj_ppfk =  ContainerInstancePortPoolJunctionColumns::PORT_POOL_FK.as_str(),
         cippj_in_use = ContainerInstancePortPoolJunctionColumns::IN_USE.as_str(),
         pp = TABLES::PORT_POOL.as_str(),
         pp_id = PortPoolColumns::ID.as_str(),
         pp_o_fk = PortPoolColumns::ORCHESTRATOR_FK.as_str(),
         o_table = TABLES::ORCHESTRATORS.as_str(),
         o_table_id = OrchestratorColumns::ID.as_str(),
         o_table_pub_uuid = OrchestratorColumns::PUBLIC_UUID.as_str(),
         pub_uuid = ORCHESTRATOR_PUBLIC_UUID.get().unwrap()
     ).as_str(), &[]).await;
    insert_result
}
