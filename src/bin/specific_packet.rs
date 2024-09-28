use std::net::UdpSocket;
use std::thread::sleep;
use std::time::Duration;
use unet::packet::keep_alive::KeepAlive;
use unet::packet::{Packet, UnetId};

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    socket.connect("127.0.0.1:10010").unwrap();
    socket.set_nonblocking(true).unwrap();

    loop {
        let n = socket
            .send(&Packet::KeepAlive(KeepAlive::new(UnetId(1))).as_bytes())
            .unwrap();
        println!("n: {n}");
        sleep(Duration::from_secs(3));
    }
}
