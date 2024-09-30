use unet::config::server::ServerConfig;
use unet::server::UnetServer;

fn main() {
    let mut config = ServerConfig::new();
    // config.max_rolling_packets_per_tick = None;
    config.recv_debug = true;
    config.send_debug = true;
    let mut server = UnetServer::from_config(config).unwrap();
    loop {
        server.update();
    }
}
