use crate::packet::Packet;
use colored::Colorize;
use std::net::SocketAddr;

struct Col(u8, u8, u8);

pub fn recv_dbg(packet: Packet, from: Option<SocketAddr>) {
    if let Some(from) = from {
        println!(
            "[{}] [{}]: {packet:?}",
            "recv".truecolor(255, 0, 0),
            from.to_string().truecolor(255, 0, 0)
        );
    } else {
        println!("[{}]: {packet:?}", "recv".truecolor(255, 0, 0));
    }
}

pub fn send_dbg(packet: Packet, to: Option<SocketAddr>) {
    if let Some(to) = to {
        println!(
            "[{}] [{}]: {packet:?}",
            "send".truecolor(0, 255, 0),
            to.to_string().truecolor(0, 255, 0)
        );
    } else {
        println!("[{}]: {packet:?}", "send".truecolor(0, 255, 0));
    }
}
