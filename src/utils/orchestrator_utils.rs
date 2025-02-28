use bollard::Docker;
use custom_tcp_listener::models::router::response_to_bytes;
use http::StatusCode;
use reqwest::{Client, Response};
use std::sync::OnceLock;
use tokio::{io::AsyncWriteExt, net::TcpStream};
use tokio_rustls::server::TlsStream;
use uuid::Uuid;

use crate::db::orchestrator_instances::create_orchestrator_instance_query;
use crate::db::orchestrators::OrchestratorColumns;

pub static ORCHESTRATOR_URI: OnceLock<String> = OnceLock::new();
pub static ORCHESTRATOR_PUBLIC_UUID: OnceLock<Uuid> = OnceLock::new();
pub static ORCHESTRATOR_INSTANCE_ID: OnceLock<i32> = OnceLock::new();
pub static REQWEST_CLIENT: OnceLock<Client> = OnceLock::new();
pub struct RouterDecoration {
    pub orchestrator_uri: String,
    pub orchesttrator_uuid: Uuid,
    pub orchestrator_instance_id: i32,
    pub request_client: Client,
    pub docker_connection: Docker,
}
pub async fn return_404(mut tcp_stream: TlsStream<TcpStream>) {
    let body: Vec<u8> = Vec::new();
    let response_builder = http::Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(body)
        .unwrap();
    let response_bytes = response_to_bytes(response_builder);
    let _write_result = tcp_stream.write_all(&response_bytes).await;
    let _flush_result = tcp_stream.flush().await;
}

pub async fn return_500(mut tcp_stream: TlsStream<TcpStream>, message: String) {
    let body: Vec<u8> = message.as_bytes().to_vec();
    let response_builder = http::Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(body)
        .unwrap();
    let response_bytes = response_to_bytes(response_builder);
    let _write_result = tcp_stream.write_all(&response_bytes).await; //we dont care if it fails
    let _flush_result = tcp_stream.flush().await;
}

pub async fn return_503(mut tcp_stream: TlsStream<TcpStream>) {
    let body: Vec<u8> = Vec::new();
    let response_builder = http::Response::builder()
        .status(StatusCode::SERVICE_UNAVAILABLE)
        .body(body)
        .unwrap();
    let response_bytes = response_to_bytes(response_builder);
    let _write_result = tcp_stream.write_all(&response_bytes).await; //we dont care if it fails
    let _flush_result = tcp_stream.flush().await;
}

pub async fn return_response(response: Response, mut tcp_stream: TlsStream<TcpStream>) {
    tracing::info!("returning");
    let mut response_builder = http::Response::builder()
        .status(response.status().as_u16())
        .version(response.version());
    tracing::info!("creating response builder");
    for (k, v) in response.headers().into_iter() {
        response_builder = response_builder.header(k, v)
    }
    let content_length: bool = response.content_length().is_some();
    let body_bytes: Option<Vec<u8>>;
    let byte_response: Option<Vec<u8>>;
    if content_length {
        body_bytes = Some(response.bytes().await.unwrap().to_vec());
        byte_response = Some(response_to_bytes(
            response_builder.body(body_bytes.unwrap()).unwrap(),
        ));
    } else {
        byte_response = Some(response_to_bytes(
            response_builder.body(Vec::new()).unwrap(),
        ));
    };

    tracing::info!("writing");
    let _write_all_result = tcp_stream.write_all(byte_response.unwrap().as_ref()).await;
    tracing::info!("finished writing | flushing");
    let _flush_result = tcp_stream.flush().await;
    tracing::info!("flushed");
}

pub async fn create_instance() -> bool {
    let insert_result = create_orchestrator_instance_query().await;
    match insert_result {
        Ok(rows) => {
            //we assert there is only 1 result
            let row = &rows[0];
            let orchestrator_instance_id =
                row.get::<&str, i32>(OrchestratorColumns::ID.to_string().as_str());
            ORCHESTRATOR_INSTANCE_ID.get_or_init(|| orchestrator_instance_id);
            let client_builder = reqwest::ClientBuilder::new();
            let client = client_builder
                .danger_accept_invalid_certs(true)
                .build()
                .unwrap();
            REQWEST_CLIENT.get_or_init(|| client);
            true
        }
        Err(err) => {
            let error = err.to_string();
            tracing::warn!(error);
            false
        }
    }
}
