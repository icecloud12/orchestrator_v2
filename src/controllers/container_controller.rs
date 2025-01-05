use std::collections::HashMap;

use bollard::{
    container::{Config, CreateContainerOptions},
    secret::{HostConfig, PortBinding},
};
use rand::Rng;
use tokio_postgres::types::Type;
use uuid::Uuid;

use crate::{
    models::service_container_models::ServiceContainer,
    utils::{
        docker_utils::DOCKER, orchestrator_utils::ORCHESTRATOR_URI, postgres_utils::POSTGRES_CLIENT,
    },
};

///load_balancer_key is the docker_image_id
pub async fn create_container(
    docker_image_id: &String,
    exposed_port: &String,
) -> Result<ServiceContainer, String> {
    //create docker container

    let docker = DOCKER.get().unwrap();

    let starting_port = std::env::var("STARTING_PORT_RANGE")
        .unwrap()
        .parse::<i32>()
        .unwrap();

    let ending_port = std::env::var("ENDING_PORT_RANGE")
        .unwrap()
        .parse::<i32>()
        .unwrap();
    let local_port = rand::thread_rng().gen_range(starting_port..ending_port);

    let mut port_binding = HashMap::new();
    port_binding.insert(
        format!("{}/tcp", exposed_port),
        Some(vec![PortBinding {
            host_port: Some(local_port.clone().to_string()),
            host_ip: Some("0.0.0.0".to_string()),
        }]),
    );

    let options = Some(CreateContainerOptions::<String> {
        ..Default::default()
    });
    let host_config: HostConfig = HostConfig {
        port_bindings: Some(port_binding),
        ..Default::default()
    };
    let uuid = Uuid::new_v4().simple().to_string();
    let config = Config {
        image: Some(docker_image_id.to_owned()),
        host_config: Some(host_config),
        env: Some(vec![
            format!("uuid={}", uuid.clone()),
            format!(
                "orchestrator_uri={}",
                ORCHESTRATOR_URI.get().unwrap().as_str()
            ),
        ]),
        ..Default::default()
    };
    let create_container_result = docker.create_container(options, config).await;
    match create_container_result {
        Ok(res) => {
            //insert into db the container
            let container_insert_result= POSTGRES_CLIENT.get().unwrap().query_typed
				("INSERT INTO containers (docker_container_id, public_port, uuid) VALUES ($1, $2, $3) RETURNING id;",
				&[
					(&res.id, Type::TEXT),
					(&local_port, Type::INT4),
					(&uuid, Type::TEXT)
				]).await;
            match container_insert_result {
                Ok(rows) => {
                    //we are only expect 1 result
                    let row = &rows[0];
                    Ok(ServiceContainer {
                        id: row.get::<&str, i32>("id"),
                        container_id: res.id,
                        public_port: local_port,
                        uuid,
                    })
                }
                Err(err) => Err(err.to_string()),
            }
        }
        Err(err) => Err(format!("Docker Create Container Error: {:#?}", err)),
    }
}
