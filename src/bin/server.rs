use std::thread::sleep;
use std::time::Duration;
use unet::server::UnetServer;

fn main() {
    let mut server = UnetServer::new("127.0.0.1:10010").unwrap();
    loop {
        println!("{server:#?}");
        server.update();
        sleep(Duration::from_millis(4000));
    }
}
