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
use std::path::PathBuf;
use std::sync::Arc;
use tracing_subscriber;
use utils::orchestrator_utils::{create_instance, RouterDecoration, ORCHESTRATOR_PUBLIC_UUID};
use utils::{docker_utils, postgres_utils};
use uuid::Uuid;
mod controllers;
mod data;
mod db;
mod handlers;
mod models;
mod utils;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        dotenv().ok();

        let database_uri;
        let database_user;
        let database_pass;
        let database_name;
        let database_port;
        let listening_address;
        let listening_port;
        let _prefix;
        let orchestrator_public_uuid;
        let subscriber = tracing_subscriber::fmt()
            .compact()
            .with_file(true)
            .with_line_number(true)
            .with_target(false)
            .finish();
        tracing::subscriber::set_global_default(subscriber).unwrap();
        tracing::info!("LOADING env variables");
        match (env::var("STATE"), env::var("PREFIX")) {
            (Ok(state), Ok(env_prefix)) if state.to_string() == "DEV" => {
                _prefix = env_prefix;
                match (
                    env::var("DEV_DATABASE_URI"),
                    env::var("DEV_DATABASE_USER"),
                    env::var("DEV_DATABASE_PASS"),
                    env::var("DEV_DATABASE_NAME"),
                    env::var("DEV_DATABASE_PORT"),
                    env::var("DEV_LISTENING_ADDRESS"),
                    env::var("DEV_LISTENING_PORT"),
                    env::var("ORCHESTRATOR_PUBLIC_UUID"),
                ) {
                    (
                        Ok(d_d_uri),
                        Ok(d_d_user),
                        Ok(d_d_pass),
                        Ok(d_d_name),
                        Ok(d_d_port),
                        Ok(d_l_address),
                        Ok(d_l_port),
                        Ok(_orchestrator_public_uuid),
                    ) => {
                        (
                            database_uri,
                            database_user,
                            database_pass,
                            database_name,
                            database_port,
                            listening_address,
                            listening_port,
                            orchestrator_public_uuid,
                        ) = (
                            d_d_uri,
                            d_d_user,
                            d_d_pass,
                            d_d_name,
                            d_d_port,
                            d_l_address,
                            d_l_port,
                            _orchestrator_public_uuid,
                        );
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
                    env::var("PROD_DATABASE_PORT"),
                    env::var("PROD_LISTENING_ADDRESS"),
                    env::var("PROD_LISTENING_PORT"),
                    env::var("ORCHESTRATOR_PUBLIC_UUID"),
                ) {
                    (
                        Ok(p_d_uri),
                        Ok(p_d_user),
                        Ok(p_d_pass),
                        Ok(p_d_name),
                        Ok(p_d_port),
                        Ok(p_l_address),
                        Ok(p_l_port),
                        Ok(_orchestrator_public_uuid),
                    ) => {
                        (
                            database_uri,
                            database_user,
                            database_pass,
                            database_name,
                            database_port,
                            listening_address,
                            listening_port,
                            orchestrator_public_uuid,
                        ) = (
                            p_d_uri,
                            p_d_user,
                            p_d_pass,
                            p_d_name,
                            p_d_port,
                            p_l_address,
                            p_l_port,
                            _orchestrator_public_uuid,
                        );
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

        let docker = docker_utils::connect();
        let postgres_client = postgres_utils::connect(
            database_uri,
            database_user,
            database_pass,
            database_name,
            database_port,
        )
        .await;

        docker_utils::deallocate_non_running(docker.clone(), postgres_client.clone()).await;
        load_balancer_controller::init();
        let orchestrator_uri: String = format!("https://{}:{}", &listening_address, &listening_port);

        let uuid_parse_result = Uuid::parse_str(orchestrator_public_uuid.as_str());
        match uuid_parse_result {
            Ok(uuid_parsed) => {
                ORCHESTRATOR_PUBLIC_UUID.get_or_init(|| uuid_parsed);
                let instance_result: Option<reqwest::Client> =
                    create_instance(&postgres_client).await;
                // ORCHESTRATOR_PUBLIC_UUID.get_or_init(|| );
                //there is no way shape or form this would miss
                // let mut rx: Receiver<SenderParameter> = request_channel_init();
                if instance_result.is_some() {
                    let reqwest_client = instance_result.unwrap();
                    tracing::info!("Loading orchestrator routes");
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
                    tracing::info!("now listening!");
                    let bind_result = bind::<RouterDecoration>(
                        router,
                        format!("{}:{}", &listening_address, &listening_port).as_str(),
                        PathBuf::from("./src/certs/cert.crt"),
                        PathBuf::from("./src/certs/cert.key"),
                    )
                    .await;
                    if bind_result.is_err() {
                        println!("{:#?}", bind_result);
                    }
                } else {
                    tracing::error!("Unable to create orchestrator instance... exiting");
                }
            }
            Err(e) => {
                let err_string = e.to_string();
                let err = err_string.as_str();
                tracing::info!(err);
            }
        }
    });

    Ok(())
}
