use std::sync::Arc;

use bollard::{container::{RemoveContainerOptions, StartContainerOptions}, errors::Error, Docker};
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
    pub async fn start_container(&self, docker: Arc<Docker>) -> Result<(), String> {
        let start_result = docker
            .start_container(&self.container_id, None::<StartContainerOptions<String>>)
            .await;
        match start_result {
            Ok(_) => Ok(()),
            Err(err) => Err(err.to_string()),
        }
    }
    pub async fn delete_container(&self, docker: Arc<Docker>) ->Result<(), Error> {
        let remove_container_options : RemoveContainerOptions = RemoveContainerOptions {force: true , ..Default::default() };
        let delete_result = docker
            .remove_container(&self.container_id, Some(remove_container_options)).await;
        delete_result
    }
}
