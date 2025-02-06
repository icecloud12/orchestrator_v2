use std::{error::Error, str::FromStr, sync::Arc};

use custom_tcp_listener::models::{router::response_to_bytes, types::Request};
use http::StatusCode;
use tokio::{io::AsyncWriteExt, net::TcpStream};
use uuid::Uuid;

use crate::{
    controllers::{
        load_balancer_controller::{
            get_or_init_load_balancer, ELoadBalancerMode, AWAITED_CONTAINERS,
            AWAITED_LOADBALANCERS, LOADBALANCERS,
        },
        request_controller::record_service_request_acceptance,
        route_controller::route_resolver,
    }, db::request_traces::update_request_responded, models::{
        service_container_models::ServiceContainer, service_image_models::ServiceImage,
        service_route_model::ServiceRoute,
    }, utils::orchestrator_utils::{return_404, return_500, return_503, return_response}
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
                (Some(route), None) => {
                    let route_id = &route.id;
                    let route_path = &route.prefix;
                    tracing::error!(
                        "Invalid image reference for route-image pair for route :[{}]{}",
                        route_id,
                        route_path
                    );
                    return_500(
                        tcp_stream,
                        "Orchestrator Error: System cannot find route-image pair".to_string(),
                    )
                    .await
                }
                (Some(service_route), Some(service_image)) => {
                    //CREATE THE REQUEST STRUCT HERE
                    let ServiceRoute {
                        image_fk,
                        prefix,
                        exposed_port,
                        ..
                    } = service_route;
                    let request_path = Arc::new(request.path.clone());
                    let request_method = Arc::new(request.method.clone());

                    let service_request_uuid: Arc<Uuid> = record_service_request_acceptance(
                        request_path,
                        request_method,
                        image_fk.clone(),
                    )
                    .await;

                    //queue_ing logic for await_load_balancers
                    let awaited_lb_hm = AWAITED_LOADBALANCERS.get().unwrap();
                    let mut awaited_lb_lock = awaited_lb_hm.lock().await;
                    //checking if the lb is being awaited here
                    if let Some(awaited_lb) =
                        awaited_lb_lock.get_mut(&service_image.docker_image_id)
                    {
                        awaited_lb.push((request, tcp_stream, service_request_uuid));
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
                        match lb.mode {
                            ELoadBalancerMode::FORWARD => {
                                let next_container_result = lb.next_container().await;
                                match next_container_result {
                                    Ok((container_public_port, container_id)) => {
                                        let client_builder = reqwest::ClientBuilder::new();
                                        let client = client_builder
                                            .danger_accept_invalid_certs(true)
                                            .build()
                                            .unwrap();
                                        let url = format!(
                                            "http://localhost:{}{}",
                                            &container_public_port, request.path
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
                                                let response_status =
                                                    response.status().as_u16() as i32;
                                                return_response(response, tcp_stream).await;
                                                tracing::info!("Responded to requestor");
                                                update_request_responded(
                                                    service_request_uuid,
                                                    &container_id,
                                                    response_status,
                                                )
                                                .await;
                                            }
                                            Err(_) => {
                                                tracing::error!("Something went wrong when trying to forward request to service");
                                                //errors concerning the connection towards the address
                                                return_503(tcp_stream).await;
                                                update_request_responded(
                                                    service_request_uuid, //formatting why do you do this man
                                                    &container_id,
                                                    503,
                                                )
                                                .await;
                                            }
                                        }
                                    }
                                    Err(_) => {
                                        //cannot next as it is empty
                                        //create the container here
                                        match lb.create_container().await {
                                            Ok(new_container) => {
                                                //created  successfully
                                                match new_container.start_container().await {
                                                    Ok(_) => {
                                                        lb.mode = ELoadBalancerMode::QUEUE;
                                                        lb.queue_request(
                                                            request,
                                                            tcp_stream,
                                                            service_request_uuid,
                                                        );
                                                        lb.queue_container(new_container).await;
                                                    }
                                                    Err(docker_start_error) => {
                                                        return_500(tcp_stream, docker_start_error)
                                                            .await;
                                                        update_request_responded(
                                                            service_request_uuid,
                                                            &new_container.id,
                                                            500,
                                                        )
                                                        .await;
                                                    }
                                                }
                                            }
                                            Err(docker_create_error) => {
                                                tracing::error!("Docker error: Failure to create container [docker_image_id: {}]", &lb.docker_image_id);
                                                return_500(tcp_stream, docker_create_error).await;
                                                update_request_responded(
                                                    service_request_uuid, //formatting why do you do this man
                                                    &-1,
                                                    500,
                                                )
                                                .await;
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
                                                    tracing::info!(
                                                        "Queueing container:{}",
                                                        &new_container.id
                                                    );
                                                    lb.queue_request(
                                                        request,
                                                        tcp_stream,
                                                        service_request_uuid,
                                                    );
                                                    lb.queue_container(new_container).await;
                                                }
                                                Err(container_start_error) => {
                                                    return_500(tcp_stream, container_start_error)
                                                        .await;

                                                    update_request_responded(
                                                        service_request_uuid, //formatting why do you do this man
                                                        &new_container.id,
                                                        500,
                                                    )
                                                    .await;
                                                }
                                            }
                                        }
                                        Err(container_create_error) => {
                                            return_500(tcp_stream, container_create_error).await;

                                            update_request_responded(
                                                service_request_uuid, //formatting why do you do this man
                                                &-1,
                                                500,
                                            )
                                            .await;
                                        }
                                    }
                                } else {
                                    lb.queue_request(request, tcp_stream, service_request_uuid);
                                }
                            }
                        };
                        drop(lb_hm);
                        //prune the containers //it is okay to fail here
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
    tracing::info!("recieved container uuid: {}", uuid);
    let awaited_container_mutex = AWAITED_CONTAINERS.get().unwrap();
    let mut awaited_containers = awaited_container_mutex.lock().await;
    println!("awaited containers {:#?}", awaited_containers);
    let load_balancer_key_option = awaited_containers.remove(uuid);
    drop(awaited_containers);
    if let Some(load_balancer_key) = load_balancer_key_option {
        let load_balacer_mutex = LOADBALANCERS.get().unwrap();
        let mut load_balancers = load_balacer_mutex.lock().await;
        let service_load_balancer = load_balancers.get_mut(&load_balancer_key).unwrap();
        let awaited_container_key_val_pair = service_load_balancer
            .awaited_containers
            .remove_entry(uuid)
            .unwrap();
        let readied_container = awaited_container_key_val_pair.1;

        let container_ref: &ServiceContainer =
            service_load_balancer.add_container(readied_container);
        let &ServiceContainer {
            id: container_fk,
            public_port: container_port,
            ..
        } = container_ref;
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
                service_load_balancer.mode = ELoadBalancerMode::FORWARD;
                let queue = service_load_balancer.empty_queue();
                for (request, tcp_stream, service_request_uuid) in queue.into_iter() {
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
                            let response_status = response.status().as_u16() as i32;
                            return_response(response, tcp_stream).await;
                            update_request_responded(
                                service_request_uuid,
                                &container_fk,
                                response_status,
                            )
                            .await;
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
    }

    Ok(())
}
