use mongodb::bson::doc;

use crate::{models::service_route_model::ServiceRoute, utils::mongodb_utils::MongoCollections};



pub async fn route_resolver(uri_string: String) -> Result<Option<ServiceRoute>, String>{
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
	let route_collection_result = MongoCollections::Routes.as_collection::<ServiceRoute>().aggregate(pipeline).with_type::<Option<ServiceRoute>>().await; //resolves at max 1 route
	match route_collection_result {
		Ok(mut route_collection) => {
			match route_collection.advance().await {
				Ok(advance_result) => {
					if advance_result {
						if let Ok(deserialized_content) = route_collection.deserialize_current() {
							Ok(deserialized_content)
						}else{
							Ok(None)
						}
					}else{
						Ok(None)
					}
				},
				Err(err) => Err(err.to_string()),
			}
		},
		Err(err) => Err(err.to_string()),
	}
}