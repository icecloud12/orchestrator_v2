extern crate dotenv;

use controllers::load_balancer_controller::{self, LOADBALANCERS};
use custom_tcp_listener::models::listener::bind;
use custom_tcp_listener::models::route::{connect, delete, get, head, option, patch, post, put, trace};
use custom_tcp_listener::models::router::Router;
use dotenv::dotenv;
use handlers::route_to_service_handler::{container_ready, route_to_service_handler};
use utils::{docker_utils, mongodb_utils};
use std::{env};

mod models;
mod handlers;
mod utils;
mod controllers;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
	dotenv().ok();
	
	let mut database_uri = String::new();
	let mut database_name = String::new();
	let mut listening_address = String::new();
	let mut listening_port = String::new();
	let mut _prefix = String::new();
	match (env::var("STATE"), env::var("PREFIX")) {
		(Ok(state), Ok(env_prefix)) if state.to_string() == "DEV"=> {
			_prefix = env_prefix;
			match(env::var("DEV_DATABASE_URI"), env::var("DEV_DATABASE_NAME"), env::var("DEV_LISTENING_ADDRESS"), env::var("DEV_LISTENING_PORT")) {
				(Ok(d_d_uri), Ok(d_d_name), Ok(d_l_address), Ok(d_l_port)) => {
					(database_uri, database_name, listening_address, listening_port) = (d_d_uri, d_d_name, d_l_address, d_l_port);
					docker_utils::connect();
					mongodb_utils::connect(database_uri, database_name).await?;
				},
				_ => {
					panic!("One of the environment variables are not set");
				}
			};
		},
		(Ok(state), Ok(env_prefix)) if state.to_string() == "PROD"=> {
			_prefix = env_prefix;
			match(env::var("PROD_DATABASE_URI"), env::var("PROD_DATABASE_NAME"), env::var("PROD_LISTENING_ADDRESS"), env::var("PROD_LISTENING_PORT")) {
				(Ok(p_d_uri), Ok(p_d_name), Ok(p_l_address), Ok(p_l_port)) => {
					(database_uri, database_name, listening_address, listening_port) = (p_d_uri, p_d_name, p_l_address, p_l_port);
					docker_utils::connect();
					mongodb_utils::connect(database_uri, database_name).await?;
				},
				_ => {
					panic!("One of the environment variables are not set");
				}
			};
		},
		_ => {
			panic!("One of the environment variables are not set");
		}
 	};

	load_balancer_controller::init();

	//there is no way shape or form this would miss
	let router = Router::new()
		.route("/orchestrator/container/ready/:uuid:".to_string(), post(container_ready))
		.route("/*".to_string(), connect(route_to_service_handler))
		.route("/*".to_string(), get(route_to_service_handler))
		.route("/*".to_string(), delete(route_to_service_handler))
		.route("/*".to_string(), head(route_to_service_handler))
		.route("/*".to_string(), option(route_to_service_handler))
		.route("/*".to_string(), patch(route_to_service_handler))
		.route("/*".to_string(), post(route_to_service_handler))
		.route("/*".to_string(), put(route_to_service_handler))
		.route("/*".to_string(), trace(route_to_service_handler));
 	bind(router, format!("{listening_address}:{listening_port}").as_str()).await?;
    return Ok(())
}

