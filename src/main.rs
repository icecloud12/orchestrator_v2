extern crate dotenv;

use controllers::load_balancer_controller;
use custom_tcp_listener::models::listener::bind;
use custom_tcp_listener::models::route::{
    connect, delete, get, head, option, patch, post, put, trace,
};
use custom_tcp_listener::models::router::Router;
use dotenv::dotenv;
use handlers::route_to_service_handler::{container_ready, route_to_service_handler};
use std::env;
use tokio::sync::mpsc::Receiver;
use utils::orchestrator_utils::ORCHESTRATOR_URI;
use utils::{docker_utils, postgres_utils};

mod controllers;
mod handlers;
mod models;
mod utils;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let mut database_uri = String::new();
    let mut database_user = String::new();
    let mut database_pass = String::new();
    let mut database_name = String::new();
    let mut listening_address = String::new();
    let mut listening_port = String::new();
    let mut _prefix = String::new();
    match (env::var("STATE"), env::var("PREFIX")) {
        (Ok(state), Ok(env_prefix)) if state.to_string() == "DEV" => {
            _prefix = env_prefix;
            match (
                env::var("DEV_DATABASE_URI"),
                env::var("DEV_DATABASE_USER"),
                env::var("DEV_DATABASE_PASS"),
                env::var("DEV_DATABASE_NAME"),
                env::var("DEV_LISTENING_ADDRESS"),
                env::var("DEV_LISTENING_PORT"),
            ) {
                (
                    Ok(d_d_uri),
                    Ok(d_d_user),
                    Ok(d_d_pass),
                    Ok(d_d_name),
                    Ok(d_l_address),
                    Ok(d_l_port),
                ) => {
                    (
                        database_uri,
                        database_user,
                        database_pass,
                        database_name,
                        listening_address,
                        listening_port,
                    ) = (d_d_uri, d_d_user, d_d_pass, d_d_name, d_l_address, d_l_port);
                }
                _ => {
                    panic!("One of the environment variables are not set");
                }
            };
        }
        (Ok(state), Ok(env_prefix)) if state.to_string() == "PROD" => {
            _prefix = env_prefix;
            match (
                env::var("PROD_DATABASE_URI"),
                env::var("PROD_DATABASE_USER"),
                env::var("PROD_DATABASE_PASS"),
                env::var("PROD_DATABASE_NAME"),
                env::var("PROD_LISTENING_ADDRESS"),
                env::var("PROD_LISTENING_PORT"),
            ) {
                (
                    Ok(p_d_uri),
                    Ok(p_d_user),
                    Ok(p_d_pass),
                    Ok(p_d_name),
                    Ok(p_l_address),
                    Ok(p_l_port),
                ) => {
                    (
                        database_uri,
                        database_user,
                        database_pass,
                        database_name,
                        listening_address,
                        listening_port,
                    ) = (p_d_uri, p_d_user, p_d_pass, p_d_name, p_l_address, p_l_port);
                }
                _ => {
                    panic!("One of the environment variables are not set");
                }
            };
        }
        _ => {
            panic!("One of the environment variables are not set");
        }
    };

    docker_utils::connect();
    postgres_utils::connect(database_uri, database_user, database_pass, database_name).await;
    load_balancer_controller::init();
    let address: String = format!("http://{}:{}", &listening_address, &listening_port);

    ORCHESTRATOR_URI.get_or_init(|| address);
    //there is no way shape or form this would miss
    // let mut rx: Receiver<SenderParameter> = request_channel_init();

    let router = Router::new()
        .route(
            "/orchestrator/container/ready/:uuid:".to_string(),
            post(container_ready),
        )
        .route("/*".to_string(), connect(route_to_service_handler))
        .route("/*".to_string(), get(route_to_service_handler))
        .route("/*".to_string(), delete(route_to_service_handler))
        .route("/*".to_string(), head(route_to_service_handler))
        .route("/*".to_string(), option(route_to_service_handler))
        .route("/*".to_string(), patch(route_to_service_handler))
        .route("/*".to_string(), post(route_to_service_handler))
        .route("/*".to_string(), put(route_to_service_handler))
        .route("/*".to_string(), trace(route_to_service_handler));
    let _ = bind(
        router,
        format!("{}:{}", &listening_address, &listening_port).as_str(),
    )
    .await;

    Ok(())
}
