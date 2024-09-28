use crate::tick::Tick;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::time::Duration;

pub mod client;
pub mod config;
pub mod debug;
pub mod packet;
pub mod rolling_average;
pub mod server;
pub mod tick;
pub mod token;

pub const DEFAULT_SERVER_ADDR: SocketAddr =
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 10010));

pub const MAX_CONNECTIONS: usize = 256;
pub const BUF_SIZE: usize = 640;

pub const DEFAULT_TPS: f32 = 20.0;
pub const DEFAULT_CLIENT_CONNECTION_TIMEOUT: Tick =
    Tick::from_duration(Duration::from_secs(4), DEFAULT_TPS);
pub const DEFAULT_SERVER_NOT_RESPONDING_TIMEOUT: Tick =
    Tick::from_duration(Duration::from_secs(4), DEFAULT_TPS);
pub const DEFAULT_KEEP_ALIVE_FREQUENCY: Tick =
    Tick::from_duration(Duration::from_millis(200), DEFAULT_TPS);
