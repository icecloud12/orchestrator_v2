use std::fmt::Display;

pub enum OrchestratorColumns {
    ID,
    NAME,
    PUBLIC_UUID,
}
impl Display for OrchestratorColumns {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::ID => write!(f, "id"),
            Self::NAME => write!(f, "name"),
            Self::PUBLIC_UUID => write!(f, "public_uuid"),
        }
    }
}
