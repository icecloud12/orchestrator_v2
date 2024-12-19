use std::{error::Error, str::FromStr};

use custom_tcp_listener::models::types::Request;
use http::StatusCode;
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::oneshot::error};

use crate::{controllers::{load_balancer_controller::{get_or_init_load_balancer, ELoadBalancerMode, LOADBALANCERS}, route_controller::route_resolver}, models::service_route_model::ServiceRoute, utils::orchestrator_utils::{return_404, return_500, return_503}};


pub async fn route_to_service_handler (request:Request, mut tcp_stream: TcpStream) -> Result<(), Box<dyn Error>> {
	
	let resolved_service = route_resolver(request.path.clone()).await;
	match resolved_service {
		Ok(t1) => {
			match t1 {
				Some(t2) => {
					let ServiceRoute {mongo_image, address, exposed_port, ..} = t2;
					let lb_key = get_or_init_load_balancer(mongo_image,address,exposed_port).await.unwrap();
					let lb_mutex = LOADBALANCERS.get().unwrap();
					let mut lb_hm = lb_mutex.lock().await;
					let lb= lb_hm.get_mut(&lb_key).unwrap();
					match lb.mode {
						ELoadBalancerMode::FORWARD => {
							let next_container_result = lb.next_container().await;
							match next_container_result {
								Ok(container_public_port) => {
									let client_builder = reqwest::ClientBuilder::new();
									let client = client_builder.danger_accept_invalid_certs(true).build().unwrap();
									let url = format!("https://localhost:{}{}", container_public_port, request.path);
									
									let send_request_result: Result<reqwest::Response, reqwest::Error> = client.request(reqwest::Method::from_str(request.method.as_str()).unwrap(), url).headers(request.headers).body(request.body).send().await;
									match send_request_result {
										Ok(response) => {
											let response_bytes = response.bytes().await.unwrap();
											let _write_result = tcp_stream.write_all(&response_bytes).await;
											let _flush_result = tcp_stream.flush().await;
										},
										Err(_) => { //errors concerning the connection towards the address
											return_503(tcp_stream).await;
										}
									}
								},
								Err(_) => { //cannot next as it is empty
									//create the container here
									
									match lb.create_container().await {
										Ok(new_container) => {
											//created  successfully	
											match new_container.start_container().await {
												Ok(_) => {
													lb.mode = ELoadBalancerMode::QUEUE;
													lb.queue_stream(tcp_stream);
													lb.add_container(new_container);
												},
												Err(docker_start_error) => {
													return_500(tcp_stream, docker_start_error).await;
												}
											}
										},
										Err(docker_create_error) => {
											return_500(tcp_stream, docker_create_error).await;
										},
									}
									
									
								},
							};
						},
						ELoadBalancerMode::QUEUE => {
							lb.queue_stream(tcp_stream);
						},
					}
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