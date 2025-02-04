///The int32 values represent the actual ID numbers in the database
// #[derive(Copy, Clone)]
pub enum ERequestTraceTypes {
    INTERCEPTED = 1,
    FORWARDED = 2,
    FAILED = 3,
    RETURNED = 4,
    FINALIZED = 5
}
