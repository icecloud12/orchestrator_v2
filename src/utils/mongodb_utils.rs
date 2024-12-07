use std::{error::Error, sync::OnceLock};

use mongodb::{Client, Collection, Database};


pub static DATABASE: OnceLock<Database> = OnceLock::new(); 

pub async fn connect(database_uri:String, database_name:String) -> Result<(), Box<dyn Error>> {
	let client:Client = Client::with_uri_str(database_uri).await?;
	DATABASE.get_or_init(|| client.database(&database_name));
	Ok(())

}

pub enum MongoCollections {
	Images,
	Containers,
	LoadBalancers,
	Routes
}

//sadly we cannot definite them early as the Generic is used to Serialize and Deserialize the document props.
impl MongoCollections {
	pub fn as_collection<T : Send + Sync>(&self) -> Collection<T>{
		let db = DATABASE.get().unwrap();
		db.collection::<T>( &self.to_string())
	}
	fn to_string(&self)->String{
		match &self {
			MongoCollections::Images => "images".to_string(),
			MongoCollections::Containers => "containers".to_string(),
			MongoCollections::LoadBalancers => "load_balancers".to_string(),
			MongoCollections::Routes => "routes".to_string(),
		}
	}
}
