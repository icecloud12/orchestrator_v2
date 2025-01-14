use tokio_postgres::types::Date;

#[derive(Default)]
pub struct ServiceRequest<'a> {
    // pub id: i32, //id from the DB equivalent
    pub path: Option<&'a String>,    //request path
    pub uuid: String,                //request uuid
    pub request_time: u128,          //time in unix epoch when the request was intercepted
    pub response_time: Option<u128>, //time in unix epoch when the request was responded with
    pub container_id: Option<i32>, //iamge_id reference as things can have the same path but different image during its lifetime
}
