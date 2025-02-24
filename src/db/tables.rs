use std::fmt::Display;

pub enum ETables{
    CONTAINER_INSTANCE_PORT_POOL_JUNCTION,
    LOAD_BALANCER_CONTAINER_JUNCTION,
    ORCHESTRATORS,
    ORCHESTRATOR_INSTANCE,
    SERVICE_ROUTE,
    PORT_POOL,
    SERVICE_CONTAINER,
    SERVICE_LOADBALANCERS,
    SERVICE_REQUEST,
    REQUEST_TRACES
}

impl Display for ETables {
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
            Self::REQUEST_TRACES => write!(f, "request_traces"),
        }
    }
}

