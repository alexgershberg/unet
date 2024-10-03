use unet::client::UnetClient;
use unet::config::test::test_config;
use unet::packet::data::Data;
use unet::packet::Packet;
use unet::server::UnetServer;

#[test]
fn client_sends_packets_out_of_order() {
    let (mut server_config, mut client_config) = test_config();
    server_config.send_debug = true;
    server_config.recv_debug = true;
    client_config.send_debug = true;
    client_config.recv_debug = true;

    let mut server = UnetServer::from_config(server_config).unwrap();
    let mut client = UnetClient::from_config(client_config).unwrap();

    client.send_connection_request_packet().unwrap();
    client
        .send_packet(Packet::Data(Data::new(client.id, 10)))
        .unwrap();
    client.send_disconnect_packet().unwrap();

    server.tick();
    assert!(server.connections[0].is_none());

    client.send_keep_alive_packet().unwrap();
    server.tick();
}
