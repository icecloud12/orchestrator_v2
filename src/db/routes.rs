use std::fmt::Display;

use tokio_postgres::{types::Type, Error, Row};

use crate::utils::postgres_utils::POSTGRES_CLIENT;

pub enum ServiceRouteColumns {
    ID,
    IMAGE_FK,
    PREFIX,
    EXPOSED_PORT,
    SEGMENTS,
    HTTPS
}

impl Display for ServiceRouteColumns {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::ID => write!(f, "id"),
            Self::IMAGE_FK => write!(f, "image_fk"),
            Self::PREFIX => write!(f, "prefix"),
            Self::EXPOSED_PORT => write!(f, "exposed_port"),
            Self::SEGMENTS => write!(f, "segments"),
            Self::HTTPS => write!(f, "https")
        }
    }
}

pub async fn route_resolution_query(uri_string: String) -> Result<Vec<Row>, Error> {
    let route_result = POSTGRES_CLIENT.get().unwrap().query_typed(
            "SELECT r.id as r_id, image_fk, prefix, exposed_port, exposed_port, segments, img.docker_image_id, r.https  FROM routes r LEFT JOIN images img on r.image_fk = img.id where $1 LIKE prefix || '%' ORDER BY segments DESC LIMIT 1",
            &[(&uri_string, Type::TEXT)]
    ).await;
    return route_result;
}
