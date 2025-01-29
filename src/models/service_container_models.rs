use crate::utils::docker_utils::DOCKER;
use bollard::container::StartContainerOptions;
use uuid::Uuid;

#[derive(Debug)]
pub struct ServiceContainer {
    ///postgres id
    pub id: i32,
    ///Docker container id
    pub container_id: String, // docker internal id of the docker container
    ///current port it is assigned with
    pub public_port: i32,
    ///auto-generated id used to identify itself when announcing it is ready
    pub uuid: Uuid,
    //container instance port pool junction fk
    pub cippj_fk: i32
}
#[derive(Debug)]
pub struct ServiceContainerBody {
    pub container_id: String, // docker internal id of the docker container
    pub public_port: usize,   //
    pub uuid: Uuid,
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
