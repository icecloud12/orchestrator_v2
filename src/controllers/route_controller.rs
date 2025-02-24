use std::sync::Arc;

use crate::{
    db::{images::ServiceImageColumns, routes::{route_resolution_query, ServiceRouteColumns}},
    models::{service_image_models::ServiceImage, service_route_model::ServiceRoute},
};

pub async fn route_resolver(
    uri_string: String,
) -> Result<(Option<ServiceRoute>, Option<ServiceImage>), String> {
    let route_result = route_resolution_query(uri_string).await;
    match route_result {
        Ok(rows) => {
            let service_route: (Option<ServiceRoute>, Option<ServiceImage>) = if !rows.is_empty() {
                let row = &rows[0];
                let image_id = row.get::<&str, i32>(ServiceRouteColumns::IMAGE_FK.to_string().as_str());
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
                    https: row.get::<&str, bool>(ServiceRouteColumns::HTTPS.to_string().as_str())
                });
                let service_image = Some(ServiceImage {
                    id: image_id,
                    docker_image_id: row.get::<&str, String>(ServiceImageColumns::DOCKER_IMAGE_ID.as_str()),
                });
                (service_route, service_image)
            } else {
                (None, None)
            };
            Ok(service_route)
        }
        Err(err) => Err(err.to_string()),
    }
}
