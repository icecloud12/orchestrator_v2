use std::{collections::HashMap, str::FromStr, sync::Arc};

use bollard::{
    container::{Config, CreateContainerOptions},
    secret::{HostConfig, PortBinding},
    Docker,
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
    utils::orchestrator_utils::{return_503, return_response},
};

///load_balancer_key is the docker_image_id
pub async fn create_container(
    docker_image_id: &String,
    exposed_port: &String,
    docker: Arc<Docker>,
    postgres_client: Arc<tokio_postgres::Client>,
    orchestrator_uri: &String,
    orchestrator_public_uuid: &Uuid,
) -> Result<ServiceContainer, String> {
    let prepare_allocation_port_result =
        prepare_port_allocation(postgres_client.clone(), orchestrator_public_uuid).await;
    match prepare_allocation_port_result {
        Ok((cippj_id, port, container_uuid)) => {
            //initialize docker container here

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
                    format!("orchestrator_uri={}", orchestrator_uri),
                ]),
                ..Default::default()
            };
            let create_container_result = docker.create_container(options, config).await;
            match create_container_result {
                Ok(res) => {
                    //insert into db the container
                    let docker_container_id = &res.id;
                    let container_insert_result = container_insert_query(
                        &docker_container_id,
                        &cippj_id,
                        postgres_client.clone(),
                    )
                    .await;
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
                            deallocate_port(vec![cippj_id], postgres_client);
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
    https: &bool,
    postgres_client: Arc<tokio_postgres::Client>,
    reqwest_client: Arc<reqwest::Client>,
) -> bool {
    tracing::info!("inside forward_request");
    insert_forward_trace(
        request_uuid.clone(),
        container_id.clone(),
        postgres_client.clone(),
    );
    tracing::info!("after forward trace | creating client_builder");
    tracing::info!("create url format");
    let url = format!(
        "{https}://localhost:{container_port}{request_path}",
        https = if *https { "https" } else { "http" },
        container_port = &container_port,
        request_path = request.path
    );

    tracing::info!("forwarding");
    let send_request_result = reqwest_client
        .request(
            reqwest::Method::from_str(request.method.as_str()).unwrap(),
            url,
        )
        .headers(request.headers)
        .body(request.body)
        .send()
        .await;
    tracing::info!("forwarded");

    match send_request_result {
        Ok(response) => {
            let response_status = response.status().as_u16() as i32;
            return_response(response, tcp_stream).await;
            insert_returned_trace(request_uuid.clone(), postgres_client.clone());
            insert_finalized_trace(request_uuid, response_status, postgres_client).await;
            return true;
        }
        Err(_) => {
            insert_failed_trace(request_uuid.clone(), postgres_client.clone());
            return_503(tcp_stream).await;
            insert_finalized_trace(
                request_uuid,
                StatusCode::SERVICE_UNAVAILABLE.as_u16() as i32,
                postgres_client,
            )
            .await;
            //remove container from the lb
            return false;
        }
    };
}
