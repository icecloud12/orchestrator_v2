use std::fmt::Display;
pub enum EServiceRequestColumns {
    ID,
    UUID,
    PATH,
    METHOD,
    IMAGE_FK,
    ORCHESTRATOR_INSTANCE_FK,
    STATUS_CODE
}

impl Display for EServiceRequestColumns {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::ID => write!(f, "id"),
            Self::UUID => write!(f, "uuid"),
            Self::PATH => write!(f, "path"),
            Self::METHOD => write!(f, "method"),
            Self::IMAGE_FK => write!(f, "image_fk"),
            Self::ORCHESTRATOR_INSTANCE_FK => write!(f, "orchestrator_instance_fk"),
            Self::STATUS_CODE => write!(f, "status_code"),
        }
    }
}
