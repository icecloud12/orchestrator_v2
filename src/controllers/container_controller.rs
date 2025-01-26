use std::collections::HashMap;

use bollard::{
    container::{Config, CreateContainerOptions},
    secret::{HostConfig, PortBinding},
};
use rand::Rng;
use uuid::Uuid;

use crate::{
    db::{
        container_instance_port_pool_junction::prepare_port_allocation,
        containers::{container_insert_query, ServiceContainerColumns},
    },
    models::service_container_models::ServiceContainer,
    utils::{docker_utils::DOCKER, orchestrator_utils::ORCHESTRATOR_URI},
};

///load_balancer_key is the docker_image_id
pub async fn create_container(
    docker_image_id: &String,
    exposed_port: &String,
) -> Result<ServiceContainer, String> {
    let prepare_allocation_port_result = prepare_port_allocation().await;
    match prepare_allocation_port_result {
        Ok((cippj_id, port, container_uuid)) => {
            //initialize docker container here
            let docker = DOCKER.get().unwrap();
            let mut port_binding = HashMap::new();
            port_binding.insert(
                format!("{}/tcp", exposed_port),
                Some(vec![PortBinding {
                    host_port: Some(port.clone().to_string()),
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
            let config = Config {
                image: Some(docker_image_id.to_owned()),
                host_config: Some(host_config),
                env: Some(vec![
                    format!("uuid={}", &container_uuid.to_string()),
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
                    let docker_container_id = &res.id;
                    let container_insert_result =
                        container_insert_query(&docker_container_id, &cippj_id).await;
                    match container_insert_result {
                        Ok(rows) => {
                            //we are only expect 1 result
                            let row = &rows[0];
                            Ok(ServiceContainer {
                                id: row.get::<&str, i32>(ServiceContainerColumns::ID.as_str()),
                                container_id: res.id,
                                public_port: *port,
                                uuid: container_uuid,
                            })
                        }
                        Err(err) => {
                            tracing::error!("Postgress Container Creation Error: {:#?}", err);
                            Err(err.to_string())
                        }
                    }
                }
                Err(err) => Err(format!("Docker Create Container Error: {:#?}", err)),
            }
        }
        Err(err) => Err(format!("Postgres Port Allocation Error: {:#?}", err)),
    }
}
