use tokio_postgres::types::Type;

use crate::utils::postgres_utils::POSTGRES_CLIENT;

pub async fn create(load_balancer_id: &i32, container_id: &i32) -> Result<(), String> {
    let insert_result = POSTGRES_CLIENT.get().unwrap()
        .query_typed(
            "INSERT INTO load_balancer_container_junction (load_balancer_fk, container_fk) VALUES ($1, $2)",
            &[
                (load_balancer_id, Type::INT4),
                (container_id, Type::INT4)
            ]
         ).await;
    match insert_result {
        Ok(_) => Ok(()),
        Err(err) => Err(err.to_string()),
    }
}
