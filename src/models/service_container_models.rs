use crate::utils::docker_utils::DOCKER;
use bollard::container::StartContainerOptions;

#[derive(Debug)]
pub struct ServiceContainer {
    pub id: i32,
    pub container_id: String, // docker internal id of the docker container
    pub public_port: i32,     //
    pub uuid: String,
}
#[derive(Debug)]
pub struct ServiceContainerBody {
    pub container_id: String, // docker internal id of the docker container
    pub public_port: usize,   //
    pub uuid: String,
}

pub type DockerImageId = String;

impl ServiceContainer {
    pub async fn start_container(&self) -> Result<(), String> {
        let docker = DOCKER.get().unwrap();
        let start_result = docker
            .start_container(&self.container_id, None::<StartContainerOptions<String>>)
            .await;
        match start_result {
            Ok(_) => Ok(()),
            Err(err) => Err(err.to_string()),
        }
    }
}
