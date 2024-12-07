extern crate dotenv;

use custom_tcp_listener::models::listener::bind;
use custom_tcp_listener::models::route::{connect, delete, get, head, option, patch, post, put, trace};
use custom_tcp_listener::models::router::Router;
use dotenv::dotenv;
use handlers::orchestrator_handler::{route_to_service_handler};
use utils::mongodb_utils;
use std::{default, env};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use http::Response;
mod models;
mod handlers;
mod utils;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
	dotenv().ok();
	
	let mut database_uri = String::new();
	let mut database_name = String::new();
	let mut listening_address = String::new();
	let mut listening_port = String::new();
	let mut prefix = String::new();
	match (env::var("STATE"), env::var("PREFIX")) {
		(Ok(state), Ok(env_prefix)) if state.to_string() == "DEV"=> {
			prefix = env_prefix;
			match(env::var("DEV_DATABASE_URI"), env::var("DEV_DATABASE_NAME"), env::var("DEV_LISTENING_ADDRESS"), env::var("DEV_LISTENING_PORT")) {
				(Ok(d_d_uri), Ok(d_d_name), Ok(d_l_address), Ok(d_l_port)) => {
					(database_uri, database_name, listening_address, listening_port) = (d_d_uri, d_d_name, d_l_address, d_l_port);
					mongodb_utils::connect(database_uri, database_name).await?;
				},
				_ => {
					panic!("One of the environment variables are not set");
				}
			};
		},
		(Ok(state), Ok(env_prefix)) if state.to_string() == "PROD"=> {
			prefix = env_prefix;
			match(env::var("PROD_DATABASE_URI"), env::var("PROD_DATABASE_NAME"), env::var("PROD_LISTENING_ADDRESS"), env::var("PROD_LISTENING_PORT")) {
				(Ok(p_d_uri), Ok(p_d_name), Ok(p_l_address), Ok(p_l_port)) => {
					(database_uri, database_name, listening_address, listening_port) = (p_d_uri, p_d_name, p_l_address, p_l_port);
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
	//there is no way shape or form this would miss
	let mut router = Router::new()
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

