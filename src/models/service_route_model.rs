use std::sync::Arc;

pub struct ServiceRoute {
    pub id: i32,
    pub image_fk: Arc<i32>,   // a reference to the image,
    pub prefix: String,       //prefix of the service
    pub exposed_port: String, // port where the service within the container would listen,
    pub segments: i32,
    pub https: bool,
}
