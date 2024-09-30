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
    client.tick(); // Client sends ConnectionRequest
    assert_eq!(client.state, ClientState::SendingConnectionRequest);

    server.tick(); // Server receives ConnectionRequest and responds with ChallengeRequest

    client.tick(); // Client receives ChallengeRequest and responds with ChallengeResponse
    assert_eq!(client.state, ClientState::SendingConnectionResponse);

    server.tick(); // Server receives ChallengeResponse, the client is now connected, the server responds with a KeepAlive packet

    client.tick(); // Client receives KeepAlive, it is now connected
    assert_eq!(client.state, ClientState::Connected);
}
