use std::fmt::Display;

pub enum ContainerInstancePortPoolJunction {
    ID,
    CONTAINER_INSTANCE_FK,
    PORT_POOL,
    IN_USE,
}

impl Display for ContainerInstancePortPoolJunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::ID => write!(f, "id"),
            Self::CONTAINER_INSTANCE_FK => write!(f, "container_instance_fk"),
            Self::PORT_POOL => write!(f, "port_pool"),
            Self::IN_USE => write!(f, "in_use"),
        }
    }
}
