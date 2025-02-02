use std::fmt::Display;

pub enum ERequestTracesColumns {
    ID,
    REQUEST_UUID,
    REQUEST_TRACE_TYPES_FK,
    CONTAINER_ID_FK
}

impl Display for ERequestTracesColumns {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::ID => write!(f, "id"),
            Self::REQUEST_UUID => write!(f, "request_uuid"),
            Self::REQUEST_TRACE_TYPES_FK => write!(f, "request_trace_types_fk"),
            Self::CONTAINER_ID_FK => write!(f, "container_id_fk"),
        }
    }
}
impl ERequestTracesColumns {
    pub fn as_str(&self) -> &str {
        match *self {
            Self::ID => "id",
            Self::REQUEST_UUID => "request_uuid",
            Self::REQUEST_TRACE_TYPES_FK => "request_trace_types_fk",
            Self::CONTAINER_ID_FK => "container_id_fk"
        }
    }
}


pub enum ERequestTraceTypes {
    INTERCEPTED = 1,
    FORWARDED = 2,
    FAILED = 3,
    RETURNED = 4,
    FINALIZED = 5
}
