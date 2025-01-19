use std::fmt::Display;

pub enum TABLES {
    SERVICE_ROUTE,
    SERVICE_REQUEST,
    ORCHESTRATORS,
    ORCHESTRATOR_INSTANCE,
    SERVICE_CONTAINER,
}

impl Display for TABLES {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::SERVICE_ROUTE => write!(f, "routes"),
            Self::SERVICE_REQUEST => write!(f, "requests"),
            Self::ORCHESTRATORS => write!(f, "orchestrators"),
            Self::ORCHESTRATOR_INSTANCE => write!(f, "orchestrator_instances"),
            Self::SERVICE_CONTAINER => write!(f, "containers"),
        }
    }
}
