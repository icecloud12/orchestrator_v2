use mongodb::bson::doc;

use crate::{models::service_route_model::ServiceRoute, utils::mongodb_utils::MongoCollections};



pub async fn route_resolver(uri_string: String) -> Result<Option<ServiceRoute>, Box< dyn std::error::Error>>{
	println!("uri_string: {}",uri_string);
	let pipeline = vec![
		doc!{
			"$match": {
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
		}},
		doc!{
			"$sort": {
				"segments": -1
			}
		},
		doc!{
			"$limit": 1
		}
	];
	let mut route_collection = MongoCollections::Routes.as_collection::<ServiceRoute>().aggregate(pipeline).with_type::<Option<ServiceRoute>>().await?; //resolves at max 1 route
	let mut service_route : Option<ServiceRoute> = None;
	if route_collection.advance().await? {
		if let Ok(deserialized_content) = route_collection.deserialize_current() {
			service_route = deserialized_content;
		}else{
			println!("Could not properly deserialize route for {}", uri_string);
		}
	}
	Ok(service_route)
}