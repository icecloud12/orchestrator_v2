use custom_tcp_listener::models::router::response_to_bytes;
use http::{HeaderMap, StatusCode, Version};
use reqwest::Response;
use tokio::{io::AsyncWriteExt, net::TcpStream};

pub async fn return_404(mut tcp_stream: TcpStream) {
    let body: &[u8] = &Vec::new();
    let response_builder = http::Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(body)
        .unwrap();
    let response_bytes = response_to_bytes(response_builder);
    let _write_result = tcp_stream.write_all(&response_bytes).await;
    let _flush_result = tcp_stream.flush().await;
}

pub async fn return_500(mut tcp_stream: TcpStream, message: String) {
    let body: &[u8] = message.as_bytes();
    let response_builder = http::Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(body)
        .unwrap();
    let response_bytes = response_to_bytes(response_builder);
    let _write_result = tcp_stream.write_all(&response_bytes).await; //we dont care if it fails
    let _flush_result = tcp_stream.flush().await;
}

pub async fn return_503(mut tcp_stream: TcpStream) {
    let body: &[u8] = &Vec::new();
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

    let content_length = response.content_length();
    if content_length.is_some() {
        let body_bytes: &[u8] = &response.bytes().await.unwrap();
        let byte_response = response_to_bytes(response_builder.body(body_bytes).unwrap());
        let _write_all_result = tcp_stream.write_all(&byte_response).await;
        let _flush_result = tcp_stream.flush().await;
    };
}
