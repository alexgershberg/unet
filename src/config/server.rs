use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

#[derive(Clone, Copy, Debug)]
pub struct ServerConfig {
    pub addr: SocketAddr,
    pub ups: f32,
    pub ms_per_update: u128,
    max_packets_per_second: f32,
    pub max_packets_per_update: f32,
    pub recv_debug: bool,
    pub send_debug: bool,
}

impl ServerConfig {
    pub fn new() -> Self {
        let addr = SocketAddr::new(IpAddr::from_str("127.0.0.1").unwrap(), 10010);
        let ups = 1.0;
        let ms_per_update = (1000.0 / ups) as u128;
        let max_packets_per_second = 50.0;
        let max_packets_per_update = max_packets_per_update(max_packets_per_second, ups);

        let recv_debug = true;
        let send_debug = true;

        Self {
            addr,
            ups,
            ms_per_update,
            max_packets_per_second,
            max_packets_per_update,
            recv_debug,
            send_debug,
        }
    }
}

fn max_packets_per_second(max_packets_per_update: f32, ups: f32) -> f32 {
    max_packets_per_update * ups
}

fn max_packets_per_update(max_packets_per_second: f32, ups: f32) -> f32 {
    max_packets_per_second / ups
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::config::server::{max_packets_per_second, max_packets_per_update};

    #[test]
    fn max_packets_per_second_calculation() {
        let ups = 20.0;
        let max_packets_per_update = 2.0;
        assert_eq!(max_packets_per_second(max_packets_per_update, ups), 40.0);
    }

    #[test]
    fn max_packets_per_update_calculation() {
        let ups = 20.0;
        let max_packets_per_second = 50.0;
        assert_eq!(max_packets_per_update(max_packets_per_second, ups), 2.5);
    }
}
