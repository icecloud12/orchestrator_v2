use std::fmt::Display;

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
