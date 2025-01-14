use std::{fmt::Display, sync::OnceLock};
use tokio_postgres::{Client, NoTls};

pub static POSTGRES_CLIENT: OnceLock<Client> = OnceLock::new();
pub async fn connect(db_host: String, db_user: String, db_pass: String, db_name: String) {
    let (client, connection) = tokio_postgres::connect(
        format!(
            "host={} user={} password={} dbname={}",
            db_host, db_user, db_pass, db_name,
        )
        .as_str(),
        NoTls,
    )
    .await
    .unwrap();
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("POSTGRES connection error: {}", e);
        }
    });
    //it's ok for it to crash since it is still on initialization phase and is a requirement
    POSTGRES_CLIENT.get_or_init(|| client);
}
//table enums
pub enum TABLES {
    SERVICE_ROUTE,
    SERVICE_REQUEST,
}

impl Display for TABLES {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::SERVICE_ROUTE => write!(f, "routes"),
            Self::SERVICE_REQUEST => write!(f, "requests"),
        }
    }
}

pub enum ServiceRouteColumns {
    ID,
    IMAGE_FK,
    PREFIX,
    EXPOSED_PORT,
    SEGMENTS,
}

impl Display for ServiceRouteColumns {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::ID => write!(f, "id"),
            Self::IMAGE_FK => write!(f, "image_fk"),
            Self::PREFIX => write!(f, "prefix"),
            Self::EXPOSED_PORT => write!(f, "exposed_port"),
            Self::SEGMENTS => write!(f, "segments"),
        }
    }
}

pub enum ServiceImageColumns {
    ID,
    DOCKER_IMAGE_ID,
}

impl Display for ServiceImageColumns {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::ID => write!(f, "id"),
            Self::DOCKER_IMAGE_ID => write!(f, "docker_image_id"),
        }
    }
}
impl ServiceImageColumns {
    pub fn as_str(&self) -> &str {
        match *self {
            Self::ID => "id",
            Self::DOCKER_IMAGE_ID => "docker_image_id",
        }
    }
}

pub enum ServiceRequestColumns {
    ID,
    PATH,
    UUID,
    REQUEST_TIME,
    RESPONSE_TIME,
    CONTAINER_FK,
    METHOD,
    STATUS_CODE,
    ORCHESTRATOR_INSTANCE,
    IMAGE_FK,
}

impl Display for ServiceRequestColumns {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::ID => write!(f, "id"),
            Self::PATH => write!(f, "path"),
            Self::UUID => write!(f, "uuid"),
            Self::REQUEST_TIME => write!(f, "request_time"),
            Self::RESPONSE_TIME => write!(f, "response_time"),
            Self::CONTAINER_FK => write!(f, "container_fk"),
            Self::METHOD => write!(f, "method"),
            Self::STATUS_CODE => write!(f, "status_code"),
            Self::ORCHESTRATOR_INSTANCE => write!(f, "orchestrator_instance"),
            Self::IMAGE_FK => write!(f, "image_fk"),
        }
    }
}
