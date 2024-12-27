use tokio_postgres::types::Type;

use crate::{
    models::service_image_models::ServiceImage,
    utils::postgres_utils::{ServiceImageColumns, POSTGRES_CLIENT},
};

pub async fn get_image(image_fk: &i32) -> Result<Option<ServiceImage>, String> {
    let image_result = POSTGRES_CLIENT
        .get()
        .unwrap()
        .query_typed(
            "SELECT id, docker_image_id FROM images where id = $1",
            &[(image_fk, Type::INT2)],
        )
        .await;
    match image_result {
        //we are expecting only 1 or no rows
        Ok(rows) => {
            let service_image_option = if rows.is_empty() {
                None
            } else {
                let row = &rows[0];
                Some(ServiceImage {
                    id: row.get::<&str, i32>(ServiceImageColumns::ID.as_str()),
                    docker_image_id: row
                        .get::<&str, String>(ServiceImageColumns::DOCKER_IMAGE_ID.as_str()),
                })
            };
            Ok(service_image_option)
        }
        Err(err) => Err(err.to_string()),
    }
}
