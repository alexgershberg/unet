use unet::client::UnetClient;
use unet::config::client::ClientConfig;
use unet::packet::data::Data;
use unet::packet::Packet;

fn main() {
    let mut config = ClientConfig::new();
    config.server_not_responding_timeout = None;

    let mut client = UnetClient::from_config(config).unwrap();
    let mut count = 0;
    loop {
        client.send(Packet::Data(Data::new(client.id, count)));
        client.tick();
        count += 1;
    }
}
