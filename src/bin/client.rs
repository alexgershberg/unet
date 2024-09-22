use std::thread::sleep;
use std::time::Duration;
use unet::client::UnetClient;

fn main() {
    let mut client = UnetClient::new("127.0.0.1:10010").unwrap();
    loop {
        client.update();
        sleep(Duration::from_millis(4000));
    }
}
