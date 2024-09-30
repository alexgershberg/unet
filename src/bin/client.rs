use console::Term;
use std::sync::mpsc::channel;
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use unet::client::UnetClient;
use unet::config::client::ClientConfig;
use unet::packet::data::Data;
use unet::packet::Packet;

fn main() {
    let (rx, tx) = channel();
    let j1 = thread::spawn(move || {
        let mut config = ClientConfig::default();
        config.recv_debug = true;
        config.send_debug = true;

        let mut client = UnetClient::from_config(config).unwrap();

        while client.update() {
            while let Ok(val) = tx.try_recv() {
                client.send(Packet::Data(Data::new(client.id, val)))
            }
        }
    });

    let j2 = thread::spawn(move || {
        let term = Term::stdout();
        loop {
            let char = term.read_char().unwrap();
            let i = char as i32;
            rx.send(i).unwrap();
        }
    });

    j1.join().unwrap();
    j2.join().unwrap();
}
