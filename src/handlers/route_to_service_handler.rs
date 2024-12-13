use std::error::Error;

use custom_tcp_listener::models::types::Request;
use tokio::net::TcpStream;

use crate::{controllers::{load_balancer_controller::{get_or_init_load_balancer, LOADBALANCERS}, route_controller::route_resolver}, models::service_route_model::ServiceRoute, utils::orchestrator_utils::{return_404, return_500}};


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
					let next_container_result = lb.next_container().await;
					match next_container_result {
						Ok(container_public_port) => {
							//forward the request here to the port 
						},
						Err(message) => {return_500(tcp_stream, message).await;},
					};
				},
				None => {
					return_404(tcp_stream).await;
				},
			}
		},
		Err(err) => {
			return_500(tcp_stream, err).await;
		},
	};
	Ok(())	
}