use std::error::Error;

use custom_tcp_listener::models::types::Request;
use mongodb::bson::doc;
use tokio::net::TcpStream;

use crate::{models::{service_image_models::ServiceImage, service_route_model::ServiceRoute}, utils::mongodb_utils::MongoCollections};


pub async fn route_to_service_handler (request:Request, tcp_stream: TcpStream) -> Result<(), Box<dyn Error>> {
	println!("herer");
	let uri_string = request.path.clone();
	println!("uri_string: {}",uri_string);
	let mut route_collection = MongoCollections::Routes.as_collection::<ServiceRoute>().find(doc! {
		"$expr": {
			"$eq": [
				{
					"$indexOfBytes": [
						uri_string.clone(),
						"$address"
						
					]
				},
				0
			]
		}
	}).await?;
	while route_collection.advance().await? {
		println!("{:#?}", route_collection.deserialize_current())
	}
	Ok(())	
}