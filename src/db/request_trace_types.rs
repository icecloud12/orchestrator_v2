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

impl ERequestTraceTypesColumns {
    pub fn as_str(&self) -> &str {
        match *self {
            Self::ID => "id",
            Self::DESCRIPTION => "description"
        }
    }
}
