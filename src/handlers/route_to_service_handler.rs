use std::error::Error;

use custom_tcp_listener::models::{router::response_to_bytes, types::Request};
use http::StatusCode;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::controllers::route_controller::route_resolver;


pub async fn route_to_service_handler (request:Request, mut tcp_stream: TcpStream) -> Result<(), Box<dyn Error>> {
	
	let resolved_service = route_resolver(request.path).await;
	//check if db returns a miss
	if resolved_service.is_ok_and(|x| x.is_some()) {
		//code here for forwarding the request
		
	}else{
		//return a 404
		let body: &[u8] = &Vec::new();
		let response_builder = http::Response::builder().status(StatusCode::NOT_FOUND).body(body).unwrap();
		let response_bytes = response_to_bytes(response_builder);
		let _write_result = tcp_stream.write_all(&response_bytes).await;
		let _flush_result = tcp_stream.flush().await;
	}
	
	Ok(())	
}