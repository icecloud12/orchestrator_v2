use std::fmt::Display;

pub enum ContainerInstancePortPoolJunction {
    ID,
    PORT_POOL_FK,
    IN_USE,
    UUID,
}

impl Display for ContainerInstancePortPoolJunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::ID => write!(f, "id"),
            Self::PORT_POOL_FK => write!(f, "port_pool_fk"),
            Self::IN_USE => write!(f, "in_use"),
            Self::UUID => write!(f, "uuid"),
        }
    }
}
impl ContainerInstancePortPoolJunction {
    pub fn as_str(&self) -> &str {
        match *self {
            Self::ID => "id",
            Self::PORT_POOL_FK => "port_pool_fk",
            Self::IN_USE => "in_use",
            Self::UUID => "uuid",
        }
    }
}
