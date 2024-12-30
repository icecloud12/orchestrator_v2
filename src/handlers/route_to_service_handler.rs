use std::{error::Error, str::FromStr};

use custom_tcp_listener::models::{router::response_to_bytes, types::Request};
use http::StatusCode;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::{
    controllers::{
        load_balancer_controller::{
            get_or_init_load_balancer, ELoadBalancerMode, AWAITED_CONTAINERS,
            AWAITED_LOADBALANCERS, LOADBALANCERS,
        },
        route_controller::route_resolver,
    },
    models::{service_image_models::ServiceImage, service_route_model::ServiceRoute},
    utils::orchestrator_utils::{return_404, return_500, return_503, return_response},
};

pub async fn route_to_service_handler(
    request: Request,
    tcp_stream: TcpStream,
) -> Result<(), Box<dyn Error>> {
    let resolved_service: Result<(Option<ServiceRoute>, Option<ServiceImage>), String> =
        route_resolver(request.path.clone()).await;
    match resolved_service {
        Ok((service_route_option, service_image_option)) => {
            match (service_route_option, service_image_option) {
                (None, _) => {
                    return_404(tcp_stream).await;
                }
                (Some(_), None) => {
                    return_500(
                        tcp_stream,
                        "Orchestrator Error: System cannot find route-image pair".to_string(),
                    )
                    .await
                }
                (Some(service_route), Some(service_image)) => {
                    let ServiceRoute {
                        image_fk,
                        prefix,
                        exposed_port,
                        ..
                    } = service_route;
                    //queue_ing logic for await_load_balancers
                    let awaited_lb_hm = AWAITED_LOADBALANCERS.get().unwrap();
                    let mut awaited_lb_lock = awaited_lb_hm.lock().await;
                    //checking if the lb is being awaited here
                    if let Some(awaited_lb) =
                        awaited_lb_lock.get_mut(&service_image.docker_image_id)
                    {
                        awaited_lb.push((request, tcp_stream));
                        drop(awaited_lb_lock);
                    } else {
                        drop(awaited_lb_lock);
                        //dropping the lock here because it is used on the next function
                        let (lb_key, new_lb) = get_or_init_load_balancer(
                            image_fk,
                            prefix,
                            exposed_port,
                            service_image,
                        )
                        .await
                        .unwrap();
                        let lb_mutex = LOADBALANCERS.get().unwrap();
                        let mut lb_hm = lb_mutex.lock().await;
                        let lb = lb_hm.get_mut(&lb_key).unwrap();
                        println!("{:#?}", &lb);
                        match lb.mode {
                            ELoadBalancerMode::FORWARD => {
                                let next_container_result = lb.next_container().await;
                                match next_container_result {
                                    Ok(container_public_port) => {
                                        println!("next-container-succes");
                                        let client_builder = reqwest::ClientBuilder::new();
                                        let client = client_builder
                                            .danger_accept_invalid_certs(true)
                                            .build()
                                            .unwrap();
                                        let url = format!(
                                            "http://localhost:{}{}",
                                            container_public_port, request.path
                                        );

                                        let send_request_result: Result<
                                            reqwest::Response,
                                            reqwest::Error,
                                        > = client
                                            .request(
                                                reqwest::Method::from_str(request.method.as_str())
                                                    .unwrap(),
                                                url,
                                            )
                                            .headers(request.headers)
                                            .body(request.body)
                                            .send()
                                            .await;
                                        match send_request_result {
                                            Ok(response) => {
                                                return_response(response, tcp_stream).await;
                                            }
                                            Err(_) => {
                                                //errors concerning the connection towards the address
                                                return_503(tcp_stream).await;
                                            }
                                        }
                                    }
                                    Err(_) => {
                                        //cannot next as it is empty
                                        //create the container here
                                        println!("next-container-fail");
                                        match lb.create_container().await {
                                            Ok(new_container) => {
                                                println!("create-container-success");
                                                //created  successfully
                                                match new_container.start_container().await {
                                                    Ok(_) => {
                                                        lb.mode = ELoadBalancerMode::QUEUE;
                                                        lb.queue_request(request, tcp_stream);
                                                        lb.queue_container(new_container).await;
                                                    }
                                                    Err(docker_start_error) => {
                                                        return_500(tcp_stream, docker_start_error)
                                                            .await;
                                                    }
                                                }
                                            }
                                            Err(docker_create_error) => {
                                                println!("create-container-fail");
                                                return_500(tcp_stream, docker_create_error).await;
                                            }
                                        }
                                    }
                                };
                            }
                            ELoadBalancerMode::QUEUE => {
                                if new_lb {
                                    match lb.create_container().await {
                                        Ok(new_container) => {
                                            match new_container.start_container().await {
                                                Ok(_container_start_success) => {
                                                    println!("queueing container");
                                                    lb.queue_request(request, tcp_stream);
                                                    lb.queue_container(new_container).await;
                                                }
                                                Err(container_start_error) => {
                                                    return_500(tcp_stream, container_start_error)
                                                        .await;
                                                }
                                            }
                                        }
                                        Err(container_create_error) => {
                                            return_500(tcp_stream, container_create_error).await;
                                        }
                                    }
                                } else {
                                    lb.queue_request(request, tcp_stream);
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(err) => {
            return_500(tcp_stream, err).await;
        }
    }

    Ok(())
}

pub async fn container_ready(
    request: Request,
    mut tcp_stream: TcpStream,
) -> Result<(), Box<dyn Error>> {
    let params = request.parameters;
    let uuid = params.get("uuid").unwrap();
    let awaited_container_mutex = AWAITED_CONTAINERS.get().unwrap();
    let awaited_containers = awaited_container_mutex.lock().await;
    println!("awaited containers{:#?}", &awaited_containers);
    println!("uuid: {}", &uuid);
    let load_balancer_key = awaited_containers.get(uuid).unwrap().clone();
    drop(awaited_containers);
    let load_balacer_mutex = LOADBALANCERS.get().unwrap();
    let mut load_balancers = load_balacer_mutex.lock().await;
    let service_load_balancer = load_balancers.get_mut(&load_balancer_key).unwrap();
    let awaited_container_key_val_pair = service_load_balancer
        .awaited_containers
        .remove_entry(uuid)
        .unwrap();
    let readied_container = awaited_container_key_val_pair.1;
    let container_port = readied_container.public_port.clone();
    service_load_balancer.add_container(readied_container);
    //acknowledge container readyness
    let empty_body: Vec<u8> = Vec::new();
    let response_builder = http::Response::builder()
        .status(StatusCode::OK)
        .body(empty_body)
        .unwrap();
    let response_bytes = response_to_bytes(response_builder);
    let _write_result = tcp_stream.write_all(&response_bytes).await; //we dont care if it fails
    let _flush_result = tcp_stream.flush().await;
    //if the load balancer is queue then change it to forwarding
    match service_load_balancer.mode {
        ELoadBalancerMode::QUEUE => {
            print!("here");
            service_load_balancer.mode = ELoadBalancerMode::FORWARD;
            let queue = service_load_balancer.empty_queue();
            println!("request_queue : {:#?}", &queue);
            for (request, mut tcp_stream) in queue.into_iter() {
                let port = container_port.clone();

                println!("here");
                //foward all queued stream to the newly made container
                //todo NEED TO STORE THE REQUEST INFO AS WELL
                let client_builder = reqwest::ClientBuilder::new();
                let client = client_builder
                    .danger_accept_invalid_certs(true)
                    .build()
                    .unwrap();
                let url = format!("http://localhost:{}{}", port, request.path);
                println!("url:{}", &url);
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
                        return_response(response, tcp_stream).await;
                    }
                    Err(_) => {
                        return_503(tcp_stream).await;
                    }
                };
            }
            service_load_balancer.request_queue = Vec::new();
        }
        ELoadBalancerMode::FORWARD => {
            //do nothing

            print!("here");
        }
    };
    service_load_balancer.request_queue = Vec::new();
    Ok(())
}
