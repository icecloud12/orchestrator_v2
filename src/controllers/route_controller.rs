use tokio_postgres::types::Type;

use crate::{
    models::service_route_model::ServiceRoute,
    utils::postgres_utils::{ServiceRouteColumns, POSTGRES_CLIENT},
};

pub async fn route_resolver(uri_string: String) -> Result<Option<ServiceRoute>, String> {
    let route_result = POSTGRES_CLIENT.get().unwrap().query_typed(
        "SELECT id, image_fk, prefix, exposed_port, exposed_port, segments FROM routes where $1 LIKE prefix || '%' ORDER BY segments DESC LIMIT 1 ORDER BY segments DESC LIMIT 1",
        &[(&uri_string, Type::TEXT)]
    ).await; //quite honestly i am expanding * to the columns so we can immediately throw when query is malformed
    match route_result {
        Ok(rows) => {
            let service_route = if !rows.is_empty() {
                let row = &rows[0];
                Some(ServiceRoute {
                    id: row.get::<&str, i32>(ServiceRouteColumns::ID.to_string().as_str()),
                    image_fk: row
                        .get::<&str, i32>(ServiceRouteColumns::IMAGE_FK.to_string().as_str()),
                    prefix: row
                        .get::<&str, String>(ServiceRouteColumns::PREFIX.to_string().as_str()),
                    exposed_port: row.get::<&str, String>(
                        ServiceRouteColumns::EXPOSED_PORT.to_string().as_str(),
                    ),
                    segments: row
                        .get::<&str, i32>(ServiceRouteColumns::SEGMENTS.to_string().as_str()),
                })
            } else {
                None
            };
            Ok(service_route)
        }
        Err(err) => Err(err.to_string()),
    }
}
