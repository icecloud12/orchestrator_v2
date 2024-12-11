use std::{process::exit, sync::OnceLock};

use bollard::Docker;

pub static DOCKER : OnceLock<Docker> = OnceLock::new();

pub fn connect(){
	DOCKER.get_or_init(|| match Docker::connect_with_local_defaults() {
		Ok(docker_connection) => docker_connection,
		Err(err) => {
			println!("{}", err);
			exit(0x0100)
		},
	});
}