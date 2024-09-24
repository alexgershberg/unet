use unet::client::UnetClient;
use unet::packet::data::Data;
use unet::packet::Packet;

fn main() {
    let mut client = UnetClient::new("127.0.0.1:10010", None).unwrap();
    let mut count = 0;
    loop {
        client.send(Packet::Data(Data::new(client.id, count)));
        client.update();
        count += 1;
    }
}
