use std::fmt::Display;

pub enum TABLES {
    SERVICE_ROUTE,
    SERVICE_REQUEST,
    ORCHESTRATORS,
    ORCHESTRATOR_INSTANCE,
    SERVICE_CONTAINER,
    PORT_POOL,
    CONTAINER_INSTANCE_PORT_POOL_JUNCTION,
}

impl Display for TABLES {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::SERVICE_ROUTE => write!(f, "routes"),
            Self::SERVICE_REQUEST => write!(f, "requests"),
            Self::ORCHESTRATORS => write!(f, "orchestrators"),
            Self::ORCHESTRATOR_INSTANCE => write!(f, "orchestrator_instances"),
            Self::SERVICE_CONTAINER => write!(f, "containers"),
            Self::PORT_POOL => write!(f, "port_pool"),
            Self::CONTAINER_INSTANCE_PORT_POOL_JUNCTION => {
                write!(f, "container_instance_port_pool_junction")
            }
        }
    }
}

impl TABLES {
    pub fn as_str(&self) -> &str {
        match *self {
            Self::SERVICE_ROUTE => "routes",
            Self::SERVICE_REQUEST => "requests",
            Self::ORCHESTRATORS => "orchestrators",
            Self::ORCHESTRATOR_INSTANCE => "orchestrator_instances",
            Self::SERVICE_CONTAINER => "containers",
            Self::PORT_POOL => "port_pool",
            Self::CONTAINER_INSTANCE_PORT_POOL_JUNCTION => "container_instance_port_pool_junction",
        }
    }
}
