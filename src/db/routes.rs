use std::{fmt::Display, sync::Arc};

use tokio_postgres::{types::Type, Error, Row};

use crate::models::service_image_models::ServiceImage;

use super::{images::ServiceImageColumns, tables::ETables};

pub enum ServiceRouteColumns {
    ID,
    IMAGE_FK,
    PREFIX,
    EXPOSED_PORT,
    SEGMENTS,
    HTTPS,
    SERVICE_FK,
}

impl Display for ServiceRouteColumns {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::ID => write!(f, "id"),
            Self::IMAGE_FK => write!(f, "image_fk"),
            Self::PREFIX => write!(f, "prefix"),
            Self::EXPOSED_PORT => write!(f, "exposed_port"),
            Self::SEGMENTS => write!(f, "segments"),
            Self::HTTPS => write!(f, "https"),
            Self::SERVICE_FK => write!(f, "service_fk"),
        }
    }
}

pub async fn route_resolution_query(
    uri_string: &String,
    postgres_client: Arc<tokio_postgres::Client>,
) -> Result<Vec<Row>, Error> {
    let route_result = postgres_client
        .query_typed(
            format!(
                "SELECT
            r.{r_id} as r_id,
            {r_image_fk},
            {r_prefix},
            {r_exposed_port},
            {r_segments},
            {i_docker_image_id},
            {r_https}
            FROM
            {routes_table} r
            LEFT JOIN {images_table} img on r.{r_image_fk} = img.{i_id}
            WHERE $1 LIKE {r_prefix} || '%' ORDER BY segments DESC LIMIT 1",
                r_id = ServiceRouteColumns::ID,
                r_image_fk = ServiceRouteColumns::IMAGE_FK,
                r_prefix = ServiceRouteColumns::PREFIX,
                r_exposed_port = ServiceRouteColumns::EXPOSED_PORT,
                r_segments = ServiceRouteColumns::SEGMENTS,
                i_docker_image_id = ServiceImageColumns::DOCKER_IMAGE_ID,
                r_https = ServiceRouteColumns::HTTPS,
                routes_table = ETables::SERVICE_ROUTE,
                images_table = ETables::SERVICE_IMAGE,
                i_id = ServiceImageColumns::ID,
            )
            .as_str(),
            &[(uri_string, Type::TEXT)],
        )
        .await;
    return route_result;
}
