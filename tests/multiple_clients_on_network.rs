use std::rc::Rc;
use std::sync::{Arc, Mutex};
use unet::client::UnetClient;
use unet::config::client::ClientConfig;
use unet::config::server::ServerConfig;
use unet::network::{VirtualNetwork, VirtualNetworkV2};
use unet::server::UnetServer;

#[test]
fn multiple_clients_on_network() {
    let mut virtual_network = Arc::new(Mutex::new(VirtualNetworkV2::new()));
    let server_addr = "1.1.1.1:1".parse().unwrap();

    let mut client_config_1 = ClientConfig::new();
    client_config_1.virtual_network = Some(virtual_network.clone());
    client_config_1.target = server_addr;

    let mut client_config_2 = ClientConfig::new();
    client_config_2.virtual_network = Some(virtual_network.clone());
    client_config_2.target = server_addr;

    let mut server_config = ServerConfig::new();
    server_config.virtual_network = Some(virtual_network.clone());
    server_config.addr = server_addr;

    let mut client1 = UnetClient::from_config(client_config_1).unwrap();
    let mut client2 = UnetClient::from_config(client_config_2).unwrap();
    let mut server = UnetServer::from_config(server_config).unwrap();

    let mut tick = || {
        client1.tick();
        server.tick();
        let virtual_network = virtual_network.lock().unwrap();
        virtual_network.tick();
    };

    tick();
    tick();
    tick();
    tick();
    tick();
}
