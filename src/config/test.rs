use crate::config::client::ClientConfig;
use crate::config::server::ServerConfig;
use crate::network::VirtualNetwork;
use std::sync::mpsc::channel;

pub fn test_config() -> (ServerConfig, ClientConfig) {
    let (server_tx, server_rx) = channel();
    let (client_tx, client_rx) = channel();

    let client_network = VirtualNetwork {
        tx: server_tx,
        rx: client_rx,
    };
    let server_network = VirtualNetwork {
        tx: client_tx,
        rx: server_rx,
    };

    let mut server_config = ServerConfig::new();
    server_config.virtual_network = Some(server_network);

    let mut client_config = ClientConfig::new();
    client_config.virtual_network = Some(client_network);

    (server_config, client_config)
}
