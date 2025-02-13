use std::fmt::Display;

pub enum ERequestTraceTypesColumns {
    ID,
    DESCRIPTION    
}

impl Display for ERequestTraceTypesColumns {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::ID => write!(f, "id"),
            Self::DESCRIPTION => write!(f, "description")
        }
    }
}
