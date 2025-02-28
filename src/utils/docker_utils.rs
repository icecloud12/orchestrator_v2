use std::{
    collections::HashMap,
    process::exit,
    str::FromStr,
    sync::{Arc, OnceLock},
};

use bollard::{
    container::{ListContainersOptions, RemoveContainerOptions},
    secret::{ContainerStateStatusEnum, ContainerStatus},
    Docker,
};
use uuid::Uuid;

use crate::db::container_instance_port_pool_junction::deallocate_port_by_container_id;

use super::orchestrator_utils::RouterDecoration;

pub fn connect() {
    match Docker::connect_with_local_defaults() {
        Ok(docker_connection) => docker_connection,
        Err(err) => {
            println!("{}", err);
            exit(0x0100)
        }
    };
}
pub async fn deallocate_non_running(route_decoration: Arc<RouterDecoration>) {
    let decoration = route_decoration.as_ref();
    let docker = &decoration.docker_connection;
    let mut filters = HashMap::new();
    let container_status_filter = vec![
        ContainerStateStatusEnum::CREATED.to_string(),
        ContainerStateStatusEnum::EXITED.to_string(),
        ContainerStateStatusEnum::DEAD.to_string(),
    ];
    filters.insert(
        "status",
        container_status_filter
            .iter()
            .map(|status| status.as_str())
            .collect::<Vec<&str>>(),
    );
    let options = ListContainersOptions {
        filters,
        ..Default::default()
    };
    let list_containers_result = docker.list_containers(Some(options)).await;
    match list_containers_result {
        Ok(container_summaries) => {
            let force_stop_container_list = container_summaries
                .into_iter()
                .map(|cs| {
                    let container_id = cs.id.unwrap();
                    let cs_id = container_id.clone();
                    tokio::spawn(async move {
                        let c_id = cs_id;
                        let options = RemoveContainerOptions {
                            force: true,
                            ..Default::default()
                        };
                        match docker.remove_container(&c_id.as_ref(), Some(options)).await {
                            Ok(_) => {
                                tracing::info!("[INFO] Successfully removed container: {}", c_id)
                            }
                            Err(err) => {
                                tracing::error!("[ERROR] Failed to remove container: {:#?}", err)
                            }
                        }
                    });
                    container_id
                })
                .collect::<Vec<String>>();
            deallocate_port_by_container_id(force_stop_container_list).await;
        }
        Err(err) => {
            tracing::error!("[ERROR]: {:#?}", err);
        }
    }
}
