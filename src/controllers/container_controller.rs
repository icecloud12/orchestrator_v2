use std::{collections::HashMap, str::FromStr, sync::Arc};

use bollard::{
    container::{Config, CreateContainerOptions},
    secret::{HostConfig, PortBinding},
};
use custom_tcp_listener::models::types::Request;
use http::StatusCode;
use tokio::net::TcpStream;
use tokio_rustls::server::TlsStream;
use uuid::Uuid;

use crate::{
    db::{
        container_instance_port_pool_junction::{deallocate_port, prepare_port_allocation},
        containers::{container_insert_query, ServiceContainerColumns},
        request_traces::{
            insert_failed_trace, insert_finalized_trace, insert_forward_trace,
            insert_returned_trace,
        },
    },
    models::service_container_models::ServiceContainer,
    utils::{
        docker_utils::DOCKER,
        orchestrator_utils::{return_503, return_response, ORCHESTRATOR_URI},
    },
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
                                cippj_fk: cippj_id,
                            })
                        }
                        Err(err) => {
                            deallocate_port(vec![cippj_id]);
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
pub async fn forward_request(
    request: Request,
    tcp_stream: TlsStream<TcpStream>,
    request_uuid: Arc<Uuid>,
    container_id: Arc<i32>,
    container_port: &i32,
) -> bool {
    insert_forward_trace(request_uuid.clone(), container_id.clone());
    //foward all queued stream to the newly made container
    //todo NEED TO STORE THE REQUEST INFO AS WELL
    let client_builder = reqwest::ClientBuilder::new();
    let client = client_builder
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
    let url = format!("http://localhost:{}{}", &container_port, request.path);
    let send_request_result = client
        .request(
            reqwest::Method::from_str(request.method.as_str()).unwrap(),
            url,
        )
        .headers(request.headers)
        .body(request.body)
        .send()
        .await;

    match send_request_result {
        Ok(response) => {
            insert_returned_trace(request_uuid.clone());
            let response_status = response.status().as_u16() as i32;
            return_response(response, tcp_stream).await;
            insert_finalized_trace(request_uuid, response_status).await;
            return true;
        }
        Err(_) => {
            insert_failed_trace(request_uuid.clone());
            return_503(tcp_stream).await;
            insert_finalized_trace(
                request_uuid,
                StatusCode::SERVICE_UNAVAILABLE.as_u16() as i32,
            )
            .await;
            //remove container from the lb
            return false;
        }
    };
}
