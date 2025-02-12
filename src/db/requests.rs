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
impl EServiceRequestColumns {
    pub fn as_str(&self) -> &str {
        match *self {
            Self::ID => "id",
            Self::UUID => "uuid",
            Self::PATH => "path",
            Self::METHOD => "method",
            Self::IMAGE_FK => "image_fk",
            Self::ORCHESTRATOR_INSTANCE_FK => "orchestrator_instance_fk",
            Self::STATUS_CODE => "status_code",
        }
    }
}
