use std::fmt::Display;

pub enum ContainerInstances {
    ID,
    CONTAINER_FK,
    UUID,
}

impl Display for ContainerInstances {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::ID => write!(f, "id"),
            Self::CONTAINER_FK => write!(f, "container_fk"),
            Self::UUID => write!(f, "uuid"),
        }
    }
}
