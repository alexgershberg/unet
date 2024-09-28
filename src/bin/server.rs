use unet::server::UnetServer;

fn main() {
    let mut server = UnetServer::new().unwrap();
    loop {
        server.update();
    }
}
