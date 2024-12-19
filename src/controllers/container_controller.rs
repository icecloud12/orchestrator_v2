use std::{collections::HashMap, fmt::format, process::{exit, ExitCode}};

use bollard::{container::{Config, CreateContainerOptions}, secret::{ContainerCreateResponse, HostConfig, PortBinding}};
use mongodb::bson::{doc, oid::ObjectId};
use rand::Rng;
use uuid::Uuid;

use crate::{models::service_container_models::{ServiceContainer, ServiceContainerBody}, utils::{docker_utils::DOCKER, mongodb_utils::{MongoCollections, DATABASE}}};


///load_balancer_key is the docker_image_id
pub async fn create_container(docker_image_id: &String, exposed_port: &String) -> Result<ServiceContainer, String>{
	//create docker container

	let docker = DOCKER.get().unwrap();
	
	let starting_port = std::env::var("STARTING_PORT_RANGE").unwrap().parse::<usize>().unwrap();

	let ending_port = std::env::var("ENDING_PORT_RANGE").unwrap().parse::<usize>().unwrap();
	let local_port = rand::thread_rng().gen_range(starting_port..ending_port);

	let mut port_binding = HashMap::new();
	port_binding.insert(format!("{}/tcp",exposed_port), Some(vec![PortBinding{
		host_port: Some(local_port.clone().to_string()),
		host_ip: Some("0.0.0.0".to_string())
	}]));

	let options = Some(CreateContainerOptions::<String>{..Default::default() });
	let host_config:HostConfig = HostConfig {
		port_bindings : Some(port_binding),
		..Default::default()
	};
	let uuid = Uuid::new_v4().simple().to_string();
	let config = Config {
		image: Some(docker_image_id.to_owned()),
		host_config: Some(host_config),
		env: Some(vec![format!("uuid={}",uuid.clone())]),
		..Default::default()
	};
	let create_container_result = docker.create_container(options, config).await;
	match create_container_result {
		Ok(res) => {
			//insert into db the container
			
			
			let insert_result = MongoCollections::Containers.as_collection::<ServiceContainerBody>().insert_one(ServiceContainerBody {
				container_id: res.id.clone(),
				public_port: local_port.clone(),
				uuid: uuid.clone()
			}).await;
			match insert_result {
				Ok(insert_result) => Ok(ServiceContainer{
					_id: insert_result.inserted_id.as_object_id().unwrap(),
					container_id: res.id,
					public_port: local_port,
					uuid: uuid.clone()
				}),
				Err(err) => Err(err.to_string()),
			}
			
		},
		Err(err ) => {
			Err(format!("Docker Create Container Error: {:#?}", err))
		}
	}
}