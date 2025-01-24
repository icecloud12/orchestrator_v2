use std::fmt::Display;

pub enum LoadBalancerContainerJunctionColumns {
    ID,
    LOAD_BALANCER_FK,
    CONTAINER_FK,
}

impl Display for LoadBalancerContainerJunctionColumns {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::ID => write!(f, "id"),
            Self::LOAD_BALANCER_FK => write!(f, "load_balancer_fk"),
            Self::CONTAINER_FK => write!(f, "container_fk"),
        }
    }
}

impl LoadBalancerContainerJunctionColumns {
    pub fn as_str(&self) -> &str {
        match *self {
            Self::ID => "id",
            Self::LOAD_BALANCER_FK => "load_balancer_fk",
            Self::CONTAINER_FK => "container_fk",
        }
    }
}
