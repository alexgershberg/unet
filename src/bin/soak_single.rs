use std::thread::sleep;
use std::time::Duration;
use unet::client::UnetClient;
use unet::config::client::ClientConfig;
use unet::packet::data::Data;
use unet::packet::Packet;

fn main() {
    let mut config = ClientConfig::new();
    config.server_not_responding_timeout = None;
    let mut client = UnetClient::from_config(config).unwrap();
    let mut count = 0;
    while client.update() {
        client.send(Packet::Data(Data::new(client.id, count)));
        count += 1;
        sleep(Duration::from_millis(1));
    }
}
