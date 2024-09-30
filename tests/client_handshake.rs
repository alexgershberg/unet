use unet::client::{ClientState, UnetClient};
use unet::config::test::test_config;
use unet::server::UnetServer;

#[test]
fn client_handshake() {
    let (mut server_config, mut client_config) = test_config();
    server_config.send_debug = true;
    server_config.recv_debug = true;
    client_config.send_debug = true;
    client_config.recv_debug = true;

    let mut server = UnetServer::from_config(server_config).unwrap();
    let mut client = UnetClient::from_config(client_config).unwrap();

    assert_eq!(client.state, ClientState::SendingConnectionRequest);

    println!("client::tick 1");
    client.tick();
    println!();

    println!("server::tick 1");
    server.tick();
    println!();

    println!("client::tick 2");
    client.tick();
    println!();

    assert_eq!(client.state, ClientState::SendingConnectionResponse);

    println!("server::tick 2");
    server.tick();
    println!();

    println!("client::tick 3");
    client.tick();
    println!();

    assert_eq!(client.state, ClientState::Connected);
}
