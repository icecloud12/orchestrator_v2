use std::borrow::Cow;
use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::{Arc, Mutex, OnceLock};

pub static TEMP:OnceLock<Arc<Mutex<HashMap<String, Arc<Mutex<Vec<TcpStream>>>>>>> = OnceLock::new();
pub struct MessageQueue {
	pub route: String,
	pub request_streams: Arc<Mutex<HashMap<String, TcpStream>>>
}

pub struct MessageQueueContainerReadyInquiry {
	pub stream : TcpStream,
	pub route: String,
}

pub struct MessageQueueContainerReadyAnnouncement {
	pub route: String,
	pub container_id: String
}

pub struct MessageQueueRequest {
	/// The request method, such as `GET`.
	pub method: Option<String>,
	/// The request path, such as `/about-us`.
	pub path: Option<String>,
	/// The request minor version, such as `1` for `HTTP/1.1`.
	pub version: Option<u8>,
	/// The request headers.
	pub headers: HashMap<String, String>,
	// // The request body 
	pub body: Option<Box<Vec<u8>>>
}
