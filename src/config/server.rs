use crate::network::VirtualNetwork;
use crate::{
    Tick, DEFAULT_CLIENT_CONNECTION_TIMEOUT, DEFAULT_KEEP_ALIVE_FREQUENCY, DEFAULT_SERVER_ADDR,
    DEFAULT_TPS,
};
use std::net::SocketAddr;

#[derive(Debug)]
pub struct ServerConfig {
    pub virtual_network: Option<VirtualNetwork>,
    pub addr: SocketAddr,
    pub client_connection_timeout: Tick,
    pub keep_alive_frequency: Tick,
    pub tps: f32,
    pub ms_per_tick: u128,
    max_rolling_packets_per_second: Option<f32>,
    pub max_rolling_packets_per_tick: Option<f32>, // If this is not specified, clients can spam as much as they want
    pub recv_debug: bool,
    pub send_debug: bool,
}

impl ServerConfig {
    pub fn new() -> Self {
        let addr = DEFAULT_SERVER_ADDR;
        let client_connection_timeout = DEFAULT_CLIENT_CONNECTION_TIMEOUT;
        let keep_alive_frequency = DEFAULT_KEEP_ALIVE_FREQUENCY;
        let tps = DEFAULT_TPS;
        let ms_per_tick = (1000.0 / tps) as u128;
        let max_rolling_packets_per_tick = Some(3.0);
        let max_rolling_packets_per_second = Some(max_rolling_packets_per_second(
            max_rolling_packets_per_tick.unwrap(),
            tps,
        ));

        let recv_debug = false;
        let send_debug = false;

        Self {
            virtual_network: None,
            addr,
            client_connection_timeout,
            keep_alive_frequency,
            tps,
            ms_per_tick,
            max_rolling_packets_per_second,
            max_rolling_packets_per_tick,
            recv_debug,
            send_debug,
        }
    }

    pub fn test() -> Self {
        todo!()
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self::new()
    }
}

fn max_rolling_packets_per_second(max_packets_per_tick: f32, ups: f32) -> f32 {
    max_packets_per_tick * ups
}

fn max_rolling_packets_per_tick(max_packets_per_second: f32, ups: f32) -> f32 {
    max_packets_per_second / ups
}

#[cfg(test)]
mod tests {
    use crate::config::server::{max_rolling_packets_per_second, max_rolling_packets_per_tick};

    #[test]
    fn max_rolling_packets_per_second_calculation() {
        let ups = 20.0;
        let max_packets_per_tick = 2.0;
        assert_eq!(
            max_rolling_packets_per_second(max_packets_per_tick, ups),
            40.0
        );
    }

    #[test]
    fn max_rolling_packets_per_tick_calculation() {
        let ups = 20.0;
        let max_packets_per_second = 50.0;
        assert_eq!(
            max_rolling_packets_per_tick(max_packets_per_second, ups),
            2.5
        );
    }
}
