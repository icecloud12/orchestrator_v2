use std::error::Error;

use custom_tcp_listener::models::{router::response_to_bytes, types::Request};
use http::StatusCode;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::{controllers::{load_balancer_controller::{get_or_init_load_balancer, LOADBALANCERS}, route_controller::route_resolver}, models::{service_container_models::DockerImageId, service_load_balancer::ServiceLoadBalancer, service_route_model::ServiceRoute}, utils::mongodb_utils::DATABASE};


pub async fn route_to_service_handler (request:Request, mut tcp_stream: TcpStream) -> Result<(), Box<dyn Error>> {
	
	let resolved_service = route_resolver(request.path).await;
	match resolved_service {
		Ok(t1) => {
			match t1 {
				Some(t2) => {
					let ServiceRoute {mongo_image, address, exposed_port, ..} = t2;
					let lb_key = get_or_init_load_balancer(mongo_image,address,exposed_port).await.unwrap();

					let mut lb_hm = LOADBALANCERS.get().unwrap().lock().await;
					let lb= lb_hm.get_mut(&lb_key).unwrap();
					let exposed_port = lb.next_container().await;
					//let next_container_result = next_container(lb, ).await;
					// match next_container_result {
					// 	Ok(next_container_port) => {
					// 		println!("next_container_port: {}", next_container_port);
					// 	},
					// 	Err(err) => {
					// 		println!("Return internal server error: {:#?}", err);
					// 	},
					// }
				},
				None => {
						let body: &[u8] = &Vec::new();
						let response_builder = http::Response::builder().status(StatusCode::NOT_FOUND).body(body).unwrap();
						let response_bytes = response_to_bytes(response_builder);
						let _write_result = tcp_stream.write_all(&response_bytes).await;
						let _flush_result = tcp_stream.flush().await;
				},
			}
		},
		Err(err) => {
			//throw internal server error
		},
	};
	// if resolved_service.is_some() {
	// 	//code here for forwarding the request
		
	// }else{
	// 	//return a 404
	
	// }
	
	Ok(())	
}