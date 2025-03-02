use std::{collections::HashMap, process::exit, sync::Arc};

use bollard::{
    container::{ListContainersOptions, RemoveContainerOptions},
    secret::ContainerStateStatusEnum,
    Docker,
};

use crate::db::container_instance_port_pool_junction::deallocate_port_by_container_id;

use super::orchestrator_utils::RouterDecoration;

pub fn connect() -> Arc<Docker>{
    match Docker::connect_with_local_defaults() {
        Ok(docker_connection) => Arc::new(docker_connection),
        Err(err) => {
            println!("{}", err);
            exit(0x0100)
        }
    }
}
pub async fn deallocate_non_running(docker: Arc<Docker>, postgres_client: Arc<tokio_postgres::Client>) {
    
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
                    let arc_docker_clone = docker.clone();
                    tokio::spawn(async move {
                        let docker = arc_docker_clone.as_ref();
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
            deallocate_port_by_container_id(force_stop_container_list, postgres_client).await;
        }
        Err(err) => {
            tracing::error!("[ERROR]: {:#?}", err);
        }
    }
}
