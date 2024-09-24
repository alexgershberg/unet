use crate::packet::{Packet, UnetId};
use crate::server::ConnectionIdentifier;
use colored::Colorize;
use std::net::{SocketAddr, ToSocketAddrs};

pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
pub const RED: Color = Color {
    r: 255,
    g: 66,
    b: 48,
};
pub const GREEN: Color = Color {
    r: 66,
    g: 255,
    b: 48,
};
pub const BLUE: Color = Color {
    r: 0,
    g: 187,
    b: 255,
};

pub fn recv_dbg(packet: Packet, connection_identifier: Option<ConnectionIdentifier>) {
    if let Some(connection_identifier) = connection_identifier {
        let id = connection_identifier.id;
        let from = connection_identifier.addr;
        println!(
            "[{}] [{:016}] [{}]: {packet:?}",
            "recv".truecolor(RED.r, RED.g, RED.b),
            format!("{:x}", id.0).truecolor(BLUE.r, BLUE.g, BLUE.b),
            from.to_string().truecolor(RED.r, RED.g, RED.b)
        );
    } else {
        println!("[{}]: {packet:?}", "recv".truecolor(RED.r, RED.g, RED.b));
    }
}

pub fn send_dbg(packet: Packet, connection_identifier: Option<ConnectionIdentifier>) {
    if let Some(connection_identifier) = connection_identifier {
        let id = connection_identifier.id;
        let to = connection_identifier.addr;
        println!(
            "[{}] [{}] [{}]: {packet:?}",
            "send".truecolor(GREEN.r, GREEN.g, GREEN.b),
            format!("{:x}", id.0).truecolor(BLUE.r, BLUE.g, BLUE.b),
            to.to_string().truecolor(GREEN.r, GREEN.g, GREEN.b),
        );
    } else {
        println!(
            "[{}]: {packet:?}",
            "send".truecolor(GREEN.r, GREEN.g, GREEN.b),
        );
    }
}

pub fn client_connect_dbg(connection_identifier: ConnectionIdentifier, index: usize) {
    let id = connection_identifier.id;
    let addr = connection_identifier.addr;
    println!(
        "[{}] [{}] [{:}] on connection slot {}",
        "connected".truecolor(255, 215, 0),
        format!("{:16x}", id.0).truecolor(BLUE.r, BLUE.g, BLUE.b),
        addr.to_string().truecolor(255, 215, 0),
        index.to_string().truecolor(255, 215, 0),
    )
}

pub fn client_disconnect_dbg(connection_identifier: ConnectionIdentifier, index: usize) {
    let id = connection_identifier.id;
    let addr = connection_identifier.addr;
    println!(
        "[{}] [{}] [{}] on connection slot {}",
        "disconnected".truecolor(255, 215, 0),
        format!("{:16x}", id.0).truecolor(BLUE.r, BLUE.g, BLUE.b),
        addr.to_string().truecolor(255, 215, 0),
        index.to_string().truecolor(255, 215, 0),
    )
}

#[cfg(test)]
mod tests {
    use crate::debug::{client_connect_dbg, client_disconnect_dbg};
    use crate::packet::UnetId;
    use crate::server::ConnectionIdentifier;

    #[test]
    fn preview() {
        let connection_identifier =
            ConnectionIdentifier::new(UnetId(u64::MAX), "255.255.255.255:65535".parse().unwrap());
        client_connect_dbg(connection_identifier, 0);
        client_disconnect_dbg(connection_identifier, 0);
        let connection_identifier =
            ConnectionIdentifier::new(UnetId(0xdeadbeef), "0.0.0.0:0".parse().unwrap());
        client_connect_dbg(connection_identifier, 0);
        client_disconnect_dbg(connection_identifier, 0);
    }
}
