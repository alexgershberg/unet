use crate::packet::Packet;
use crate::server::connection::ConnectionIdentifier;
use colored::Colorize;
use std::net::ToSocketAddrs;

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

pub const YELLOW: Color = Color {
    r: 255,
    g: 215,
    b: 0,
};

pub fn recv_dbg(
    packet: Packet,
    connection_identifier: Option<ConnectionIdentifier>,
    index: Option<usize>,
) {
    if let (Some(connection_identifier), Some(index)) = (connection_identifier, index) {
        let id = connection_identifier.id;
        let from = connection_identifier.addr;
        println!(
            "[{}] [{}] [{}] [{:0>3}]: {packet:?}",
            "recv".truecolor(RED.r, RED.g, RED.b),
            format!("{:0>16x}", id.0).truecolor(BLUE.r, BLUE.g, BLUE.b),
            from.to_string().truecolor(RED.r, RED.g, RED.b),
            index.to_string().truecolor(YELLOW.r, YELLOW.g, YELLOW.b)
        );
    } else if let Some(connection_identifier) = connection_identifier {
        let id = connection_identifier.id;
        let from = connection_identifier.addr;
        println!(
            "[{}] [{}] [{}]: {packet:?}",
            "recv".truecolor(RED.r, RED.g, RED.b),
            format!("{:0>16x}", id.0).truecolor(BLUE.r, BLUE.g, BLUE.b),
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
            format!("{:0>16x}", id.0).truecolor(BLUE.r, BLUE.g, BLUE.b),
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
    use crate::debug::{client_connect_dbg, client_disconnect_dbg, recv_dbg};
    use crate::packet::keep_alive::KeepAlive;
    use crate::packet::{Packet, UnetId};
    use crate::server::connection::ConnectionIdentifier;

    #[test]
    fn recv_debug_preview() {
        let packet = Packet::KeepAlive(KeepAlive::new(UnetId(10)));
        let connection_identifier = Some(ConnectionIdentifier::new(
            UnetId(567),
            "127.0.0.1:0".parse().unwrap(),
        ));
        let index = Some(26);
        recv_dbg(packet, connection_identifier, index);
    }

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
