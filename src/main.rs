extern crate dotenv;

use dotenv::dotenv;
use reqwest::ResponseBuilderExt;
use std::{default, env};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use log::warn;

use http::{HeaderMap, HeaderName, HeaderValue, Request, Response, StatusCode, Version};
mod models;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
	
	dotenv().ok();
	let address = env::var("ADDRESS");
	let port = env::var("PORT");
	
	if address.is_ok() && port.is_ok() {
		let address_val = address.unwrap();
		let port_val = port.unwrap();
		println!("Attempting to connect to {address}:{port}",address=&address_val, port=&port_val );
		let listener = TcpListener::bind(format!("{address}:{port}",address=address_val, port=port_val)).await?;
		
		println!("Now listening!");
		loop {
			let (mut stream, addr) = listener.accept().await?;
			
			tokio::spawn(async move {
				listen(stream).await;
			});
		}
		
	}else{
		warn!("Connection string variables are invalid! Exiting program");
	}
	
    return Ok(())
}


async fn listen(mut stream:TcpStream)->Result<(), Box<dyn std::error::Error>>{
	let mut buffer = [0; 1024];
	let _bytes_read = stream.read(&mut buffer).await?;
	
	//calculating eaders
	let mut headers = [httparse::EMPTY_HEADER; 64];
	let mut req = httparse::Request::new(&mut headers);
	let parse_result = req.parse(&buffer).unwrap();
	
	let byte_offset = match parse_result {
		httparse::Status::Complete(offset) => Ok(offset),
		httparse::Status::Partial => {
			Err(()) // would be nice if wecould create a response already and return an error with a malformed request
		}
    };
	let content_length = req.headers.iter().find(|h| h.name == "Content-Length")
		.and_then(|h| std::str::from_utf8(h.value).ok())
		.and_then(|v| v.parse::<usize>().ok()).unwrap_or(0);

	let body: &[u8] = &buffer[ byte_offset.unwrap()..(byte_offset.unwrap() + content_length)];
	let body_str: std::borrow::Cow<str> = String::from_utf8_lossy(body);
	
	let req_headers: &mut [httparse::Header<'_>] = req.headers;

	
	let response_builder = Response::builder();
	
	let response_body = b"Hello world";
	let constructed_response = response_builder.status(200)
		.header("content-Length", response_body.len())
		.header("Content-Type", "text/html")
		.body(response_body).unwrap();
	let (parts, body) = constructed_response.into_parts();
	
	let mut response_line = format!("{:#?} {}", parts.version, parts.status);
	parts.headers.iter().for_each(
		|(header_name, header_value)|{
			response_line = response_line.clone() + "\r\n"+ &format!("{}:{}", header_name.as_str(), header_value.to_str().unwrap());
		}		 
	);
	response_line = format!("{}\r\n\r\n{}", response_line,String::from_utf8(body.to_vec()).unwrap());
	
	println!("{}",response_line);
	let _stream_write = stream.write_all(response_line.as_bytes()).await;
	let _stream_flush = stream.flush().await;
	return Ok(())
	
}
