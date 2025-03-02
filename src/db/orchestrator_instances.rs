use std::fmt::Display;
use tokio::task::JoinError;
use tokio_postgres::{types::Type, Error, Row};

use crate::utils::orchestrator_utils::ORCHESTRATOR_PUBLIC_UUID;

use super::{orchestrators::OrchestratorColumns, tables::ETables};

pub enum OrchestratorInstanceColumns {
    ID,
    ORCHESTRATOR_FK,
    UUID_INSTANCE,
}

impl Display for OrchestratorInstanceColumns {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::ID => write!(f, "id"),
            Self::ORCHESTRATOR_FK => write!(f, "orchestrator_fk"),
            Self::UUID_INSTANCE => write!(f, "uuid_instance"),
        }
    }
}

pub async fn create_orchestrator_instance_query(postgres_client: &tokio_postgres::Client) -> Result<Vec<Row>, Error> {

    return postgres_client.query_typed(
        format!(
            "INSERT INTO {orchestrator_instance_table_name} ({col_orchestrator_fk}) VALUES ((SELECT {orchestrator_id} from {orchestrator_table} where {public_uuid} = $1)) RETURNING {orchestrator_instance_id}",
            orchestrator_instance_table_name = ETables::ORCHESTRATOR_INSTANCE.to_string(),
            col_orchestrator_fk = OrchestratorInstanceColumns::ORCHESTRATOR_FK.to_string(),
            orchestrator_id = OrchestratorColumns::ID.to_string(),
            orchestrator_table = ETables::ORCHESTRATORS.to_string(),
            public_uuid = OrchestratorColumns::PUBLIC_UUID.to_string(),
            orchestrator_instance_id = OrchestratorInstanceColumns::ID.to_string()
        )
        .as_str(),
        &[
            (ORCHESTRATOR_PUBLIC_UUID.get().unwrap(), Type::UUID)
        ],
    ).await;
}
