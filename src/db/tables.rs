use std::fmt::Display;

pub enum TABLES {
    CONTAINER_INSTANCE_PORT_POOL_JUNCTION,
    LOAD_BALANCER_CONTAINER_JUNCTION,
    ORCHESTRATORS,
    ORCHESTRATOR_INSTANCE,
    SERVICE_ROUTE,
    PORT_POOL,
    SERVICE_CONTAINER,
    SERVICE_LOADBALANCERS,
    SERVICE_REQUEST,
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
            Self::SERVICE_LOADBALANCERS => write!(f, "load_balancers"),
            Self::LOAD_BALANCER_CONTAINER_JUNCTION => write!(f, "load_balancer_container_junction"),
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
            Self::SERVICE_LOADBALANCERS => "load_balancers",
            Self::LOAD_BALANCER_CONTAINER_JUNCTION => "load_balancer_container_junction",
        }
    }
}
