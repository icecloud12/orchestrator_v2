use std::fmt::Display;

pub enum PortPool {
    ID,
    PORT,
    ORCHESTRATOR_FK,
}

impl Display for PortPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::ID => write!(f, "id"),
            Self::PORT => write!(f, "port"),
            Self::ORCHESTRATOR_FK => write!(f, "orchestrator_fk"),
        }
    }
}
