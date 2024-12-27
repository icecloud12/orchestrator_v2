use std::{fmt::Display, sync::OnceLock};
use tokio_postgres::{Client, NoTls};

pub static POSTGRES_CLIENT: OnceLock<Client> = OnceLock::new();
pub async fn connect() {
    let (client, connection) =
        tokio_postgres::connect("host=localhost user=postgres password=root dbname=orchestrator", NoTls)
            .await
            .unwrap();
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("POSTGRES connection error: {}",e);
        }
    });
    //it's ok for it to crash since it is still on initialization phase and is a requirement
    POSTGRES_CLIENT.get_or_init(|| client);
}
//table enums
pub enum TABLES {
    SERVICE_ROUTE,
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
