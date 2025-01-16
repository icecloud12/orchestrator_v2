use std::sync::OnceLock;

use custom_tcp_listener::models::router::response_to_bytes;
use http::StatusCode;
use reqwest::Response;
use tokio::{io::AsyncWriteExt, net::TcpStream};
use tokio_postgres::{types::Type, GenericClient};
use uuid::Uuid;

use super::postgres_utils::{
    OrchestratorColumns, OrchestratorInstanceColumns, POSTGRES_CLIENT, TABLES,
};

pub static ORCHESTRATOR_URI: OnceLock<String> = OnceLock::new();
pub static ORCHESTRATOR_PUBLIC_UUID: OnceLock<Uuid> = OnceLock::new();
pub static ORCHESTRATOR_INSTANCE_ID: OnceLock<i32> = OnceLock::new();

pub async fn return_404(mut tcp_stream: TcpStream) {
    let body: Vec<u8> = Vec::new();
    let response_builder = http::Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(body)
        .unwrap();
    let response_bytes = response_to_bytes(response_builder);
    let _write_result = tcp_stream.write_all(&response_bytes).await;
    let _flush_result = tcp_stream.flush().await;
}

pub async fn return_500(mut tcp_stream: TcpStream, message: String) {
    let body: Vec<u8> = message.as_bytes().to_vec();
    let response_builder = http::Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(body)
        .unwrap();
    let response_bytes = response_to_bytes(response_builder);
    let _write_result = tcp_stream.write_all(&response_bytes).await; //we dont care if it fails
    let _flush_result = tcp_stream.flush().await;
}

pub async fn return_503(mut tcp_stream: TcpStream) {
    let body: Vec<u8> = Vec::new();
    let response_builder = http::Response::builder()
        .status(StatusCode::SERVICE_UNAVAILABLE)
        .body(body)
        .unwrap();
    let response_bytes = response_to_bytes(response_builder);
    let _write_result = tcp_stream.write_all(&response_bytes).await; //we dont care if it fails
    let _flush_result = tcp_stream.flush().await;
}

pub async fn return_response(response: Response, mut tcp_stream: TcpStream) {
    let mut response_builder = http::Response::builder()
        .status(response.status().as_u16())
        .version(response.version());

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

    let _write_all_result = tcp_stream.write_all(byte_response.unwrap().as_ref()).await;
    let _flush_result = tcp_stream.flush().await;
}

pub async fn create_instance() -> bool {
    let client = POSTGRES_CLIENT.get().unwrap();
    let insert_result = client
        .query_typed(
            format!(
                "INSERT INTO {orchestrator_instance_table_name} ({col_orchestrator_fk}) VALUES ((SELECT {orchestrator_id} from {orchestrator_table} where {public_uuid} = $1)) RETURNING {orchestrator_instance_id}",
                orchestrator_instance_table_name = TABLES::ORCHESTRATOR_INSTANCE.to_string(),
                col_orchestrator_fk = OrchestratorInstanceColumns::ORCHESTRATOR_FK.to_string(),
                orchestrator_id = OrchestratorColumns::ID.to_string(),
                orchestrator_table = TABLES::ORCHESTRATORS.to_string(),
                public_uuid = OrchestratorColumns::PUBLIC_UUID.to_string(),
                orchestrator_instance_id = OrchestratorInstanceColumns::ID.to_string()
            )
            .as_str(),
            &[
                (ORCHESTRATOR_PUBLIC_UUID.get().unwrap(), Type::UUID)
            ],
    ).await;
    match insert_result {
        Ok(rows) => {
            //we assert there is only 1 result
            let row = &rows[0];
            let orchestrator_instance_id =
                row.get::<&str, i32>(OrchestratorColumns::ID.to_string().as_str());
            ORCHESTRATOR_INSTANCE_ID.get_or_init(|| orchestrator_instance_id);
            true
        }
        Err(err) => {
            let error = err.to_string();
            tracing::warn!(error);
            false
        }
    }
}
