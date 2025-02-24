use tokio_postgres::types::Type;

use crate::utils::postgres_utils::POSTGRES_CLIENT;
use std::{fmt::Display, sync::Arc};

use super::tables::ETables;
pub enum ELoadBalancerContainerJunctionColumns {
    ID,
    LOAD_BALANCER_FK,
    CONTAINER_FK,
}

impl Display for ELoadBalancerContainerJunctionColumns {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::ID => write!(f, "id"),
            Self::LOAD_BALANCER_FK => write!(f, "load_balancer_fk"),
            Self::CONTAINER_FK => write!(f, "container_fk"),
        }
    }
}


pub fn insert_lbcj(load_balancer_id: Arc<i32>, container_id: Arc<i32>) {
    tokio::spawn(async move {
        let client = POSTGRES_CLIENT.get().unwrap();
        let _ = client
            .query_typed(
                format!(
                    "INSERT INTO {lbcj_table} ({lbcj_lbfk}, {lbcj_cfk})
        VALUES
        ($1, $2)",
                    lbcj_table = ETables::LOAD_BALANCER_CONTAINER_JUNCTION,
                    lbcj_lbfk = ELoadBalancerContainerJunctionColumns::LOAD_BALANCER_FK,
                    lbcj_cfk = ELoadBalancerContainerJunctionColumns::CONTAINER_FK
                )
                .as_str(),
                &[
                    (load_balancer_id.as_ref(), Type::INT4),
                    (container_id.as_ref(), Type::INT4),
                ],
            )
            .await;
    });
}
