use std::{fmt::Display, sync::Arc};

use tokio_postgres::{types::Type, Error, Row};


use crate::utils::postgres_utils;

use super::{
    container_instance_port_pool_junction::ContainerInstancePortPoolJunctionColumns, tables::ETables,
};

pub enum PortPoolColumns {
    ID,
    PORT,
    ORCHESTRATOR_FK,
}

impl Display for PortPoolColumns {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::ID => write!(f, "id"),
            Self::PORT => write!(f, "port"),
            Self::ORCHESTRATOR_FK => write!(f, "orchestrator_fk"),
        }
    }
}

impl PortPoolColumns {
    pub fn as_str(&self) -> &str {
        match *self {
            Self::ID => "id",
            Self::PORT => "port",
            Self::ORCHESTRATOR_FK => "orchestrator_fk",
        }
    }
}
pub async fn get_port_pool(ppp_id: &i32, postgres_client: &tokio_postgres::Client) -> Result<Vec<Row>, Error> {
        let select_result = postgres_client.query_typed(
            format!(
                "
        SELECT pp.{pp_port} as pp_port, cippj.{cippj_id} as cippj_id
        FROM {pp} pp
        LEFT JOIN {cippj} cippj on pp.{pp_id} = cippj.{cippj_ppfk}
        WHERE
            pp.{id} = $1    
        ",
                pp_port = PortPoolColumns::PORT,
                cippj_id = ContainerInstancePortPoolJunctionColumns::ID,
                pp = ETables::PORT_POOL,
                cippj = ETables::CONTAINER_INSTANCE_PORT_POOL_JUNCTION,
                pp_id = PortPoolColumns::ID,
                cippj_ppfk = ContainerInstancePortPoolJunctionColumns::PORT_POOL_FK,
                id = PortPoolColumns::ID,
            )
            .as_str(),
            &[(ppp_id, Type::INT4)],
        )
        .await;
    select_result
}
