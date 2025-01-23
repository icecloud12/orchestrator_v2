use std::fmt::Display;

use tokio_postgres::{types::Type, Error, GenericClient, Row};

use crate::utils::postgres_utils::POSTGRES_CLIENT;

use super::tables::TABLES;

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
pub async fn get_port_pool(pp_id: &i32) -> Result<Vec<Row>, Error> {
    let client = POSTGRES_CLIENT.get().unwrap();
    let select_result = client
        .query_typed(
            format!(
                "
        SELECT {p} FROM {pp} pp
        WHERE
            pp.{id} = $1    
        ",
                p = PortPoolColumns::PORT.as_str(),
                pp = TABLES::PORT_POOL.as_str(),
                id = PortPoolColumns::ID.as_str()
            )
            .as_str(),
            &[(pp_id, Type::INT4)],
        )
        .await;
    select_result
}
