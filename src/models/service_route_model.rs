use std::sync::Arc;

pub struct ServiceRoute {
    pub id: i32,
    pub image_fk: Arc<i32>,   // a reference id to the image,
    pub prefix: String,       // prefix of the service
    pub exposed_port: String, // port where the service within the container would listen,
    pub segments: i32,        // segment counts
    pub https: bool,          // if the route requires HTTPS
                              // pub service_ref: i32,
}
