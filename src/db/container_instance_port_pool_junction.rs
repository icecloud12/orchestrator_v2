use std::fmt::Display;

use tokio_postgres::{Error, Row};

use crate::utils::postgres_utils::POSTGRES_CLIENT;

use super::tables::TABLES;

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

pub async fn allocate_port -> Result<Vec<Row>, Error> {
    let client = POSTGRES_CLIENT.get().unwrap();
    let insert_result = client.query_typed(format!(
        "INSERT INTO {cippj} ({cippj_ppfk}, {cippj_in_use}) values (
            SELECT pp.{pp_id} from {pp} as pp,
            
        )",
         cippj = TABLES::CONTAINER_INSTANCE_PORT_POOL_JUNCTION.as_str(),
         cippj_ppfk =  ContainerInstancePortPoolJunctionColumns::PORT_POOL_FK.as_str(),
         cippj_in_use = ContainerInstancePortPoolJunctionColumns::IN_USE.as_str()
     ).as_str(), &[]);

} 
