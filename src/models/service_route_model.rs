pub struct ServiceRoute {
    pub id: usize,
    pub image_fk: usize,      // a reference to the image,
    pub prefix: String,       //prefix of the service
    pub exposed_port: String, // port where the service within the container would listen,
    pub segments: usize,
}
