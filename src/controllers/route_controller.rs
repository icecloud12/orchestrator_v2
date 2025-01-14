use std::sync::Arc;

use tokio_postgres::types::Type;

use crate::{
    models::{service_image_models::ServiceImage, service_route_model::ServiceRoute},
    utils::postgres_utils::{ServiceRouteColumns, POSTGRES_CLIENT},
};

pub async fn route_resolver(
    uri_string: String,
) -> Result<(Option<ServiceRoute>, Option<ServiceImage>), String> {
    let route_result = POSTGRES_CLIENT.get().unwrap().query_typed(
        "SELECT r.id as r_id, image_fk, prefix, exposed_port, exposed_port, segments, img.docker_image_id  FROM routes r LEFT JOIN images img on r.image_fk = img.id where $1 LIKE prefix || '%' ORDER BY segments DESC LIMIT 1",
        &[(&uri_string, Type::TEXT)]
    ).await; //quite honestly i am expanding * to the columns so we can immediately throw when query is malformed
    match route_result {
        Ok(rows) => {
            let service_route: (Option<ServiceRoute>, Option<ServiceImage>) = if !rows.is_empty() {
                let row = &rows[0];
                let image_id =
                    row.get::<&str, i32>(ServiceRouteColumns::IMAGE_FK.to_string().as_str());
                let service_route = Some(ServiceRoute {
                    id: row.get::<&str, i32>("r_id"),
                    image_fk: Arc::new(image_id.clone()),
                    prefix: row
                        .get::<&str, String>(ServiceRouteColumns::PREFIX.to_string().as_str()),
                    exposed_port: row.get::<&str, String>(
                        ServiceRouteColumns::EXPOSED_PORT.to_string().as_str(),
                    ),
                    segments: row
                        .get::<&str, i32>(ServiceRouteColumns::SEGMENTS.to_string().as_str()),
                });
                let service_image: Option<ServiceImage> =
                    if let Ok(docker_image_id) = row.try_get::<&str, String>("docker_image_id") {
                        Some(ServiceImage {
                            id: image_id,
                            docker_image_id,
                        })
                    } else {
                        None
                    };
                (service_route, service_image)
            } else {
                (None, None)
            };
            Ok(service_route)
        }
        Err(err) => Err(err.to_string()),
    }
}
