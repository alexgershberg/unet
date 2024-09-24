use std::time::Duration;

pub mod client;
pub mod debug;
pub mod packet;
pub mod server;
pub mod token;

pub const MAX_CONNECTIONS: usize = 256;
pub const BUF_SIZE: usize = 640;

pub const CLIENT_CONNECTION_TIMEOUT: Duration = Duration::from_secs(4);
pub const SERVER_NOT_RESPONDING_TIMEOUT: Duration = Duration::from_secs(4);
pub const KEEP_ALIVE_FREQUENCY: Duration = Duration::from_millis(200);
