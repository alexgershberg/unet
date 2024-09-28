use std::cmp::max;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub addr: SocketAddr,
    pub ups: f32,
    pub ms_per_update: u128,
    max_packets_per_second: f32,
    pub max_packets_per_update: f32,
}

impl Config {
    pub fn new() -> Self {
        let addr = SocketAddr::new(IpAddr::from_str("127.0.0.1").unwrap(), 10010);
        let ups = 20.0;
        let ms_per_update = (1000.0 / ups) as u128;
        let max_packets_per_second = 50.0;
        let max_packets_per_update = max_packets_per_update(max_packets_per_second, ups);

        Self {
            addr,
            ups,
            ms_per_update,
            max_packets_per_second,
            max_packets_per_update,
        }
    }
}

fn max_packets_per_second(max_packets_per_update: f32, ups: f32) -> f32 {
    max_packets_per_update * ups
}

fn max_packets_per_update(max_packets_per_second: f32, ups: f32) -> f32 {
    max_packets_per_second / ups
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::config::{max_packets_per_second, max_packets_per_update};

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
