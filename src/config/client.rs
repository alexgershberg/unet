use crate::network::VirtualNetwork;
use crate::packet::UnetId;
use crate::tick::Tick;
use crate::{
    DEFAULT_KEEP_ALIVE_FREQUENCY, DEFAULT_SERVER_ADDR, DEFAULT_SERVER_NOT_RESPONDING_TIMEOUT,
    DEFAULT_TPS,
};
use std::net::SocketAddr;

#[derive(Debug)]
pub struct ClientConfig {
    pub virtual_network: Option<VirtualNetwork>,
    pub id: Option<UnetId>,
    pub target: SocketAddr,
    pub server_not_responding_timeout: Option<Tick>,
    pub keep_alive_frequency: Tick,
    pub tps: f32,
    pub ms_per_tick: u128,
    pub recv_debug: bool,
    pub send_debug: bool,
    pub action_trace: bool,
}

impl ClientConfig {
    pub fn new() -> Self {
        let target = DEFAULT_SERVER_ADDR;
        let server_not_responding_timeout = Some(DEFAULT_SERVER_NOT_RESPONDING_TIMEOUT);
        let keep_alive_frequency = DEFAULT_KEEP_ALIVE_FREQUENCY;
        let tps = DEFAULT_TPS;
        let ms_per_tick = (1000.0 / tps) as u128;

        let recv_debug = false;
        let send_debug = false;
        let action_trace = false;

        Self {
            virtual_network: None,
            id: None,
            target,
            server_not_responding_timeout,
            keep_alive_frequency,
            tps,
            ms_per_tick,
            recv_debug,
            send_debug,
            action_trace,
        }
    }

    pub fn test() -> Self {
        todo!()
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self::new()
    }
}
