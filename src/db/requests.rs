use std::fmt::Display;

pub enum ServiceRequestColumns {
    ID,
    PATH,
    UUID,
    REQUEST_TIME,
    RESPONSE_TIME,
    CONTAINER_FK,
    METHOD,
    STATUS_CODE,
    ORCHESTRATOR_INSTANCE_FK,
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
            Self::ORCHESTRATOR_INSTANCE_FK => write!(f, "orchestrator_instance_fk"),
            Self::IMAGE_FK => write!(f, "image_fk"),
        }
    }
}
