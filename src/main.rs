extern crate dotenv;

use dotenv::dotenv;
use std::{default, env};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use log::warn;

use http::Response;
mod models;
mod handlers;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
	dotenv().ok();
	
	let mut database_uri = String::new();
	let mut database_name = String::new();
	let mut listening_address = String::new();
	let mut listening_port = String::new();
	
	match env::var("STATE") {
		
		Ok(state) if state.to_string() == "DEV"=> {
			match(env::var("DEV_DATABASE_URI"), env::var("DEV_DATABASE_NAME"), env::var("DEV_LISTENING_ADDRESS"), env::var("DEV_LISTENING_PORT")) {
				(Ok(d_d_uri), Ok(d_d_name), Ok(d_l_address), Ok(d_l_port)) => {
					(database_uri, database_name, listening_address, listening_port) = (d_d_uri, d_d_name, d_l_address, d_l_port);
				},
				_ => {
					panic!("One of the environment variables are not set");
				}
			};
		},
		Ok(state) if state.to_string() == "PROD"=> {
			match(env::var("PROD_DATABASE_URI"), env::var("PROD_DATABASE_NAME"), env::var("PROD_LISTENING_ADDRESS"), env::var("PROD_LISTENING_PORT")) {
				(Ok(p_d_uri), Ok(p_d_name), Ok(p_l_address), Ok(p_l_port)) => {
					(database_uri, database_name, listening_address, listening_port) = (p_d_uri, p_d_name, p_l_address, p_l_port);
				},
				_ => {
					panic!("One of the environment variables are not set");
				}
			};
		},
		_ => {
			panic!("One of the environment variables are not set");
		}
 	}
	
    return Ok(())
}

