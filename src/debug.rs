use crate::packet::Packet;
use crate::server::ConnectionIdentifier;
use colored::Colorize;

struct Color {
    r: u8,
    g: u8,
    b: u8,
}
const RED: Color = Color {
    r: 255,
    g: 66,
    b: 48,
};
const GREEN: Color = Color {
    r: 66,
    g: 255,
    b: 48,
};
const BLUE: Color = Color {
    r: 0,
    g: 187,
    b: 255,
};

pub fn recv_dbg(packet: Packet, connection_identifier: Option<ConnectionIdentifier>) {
    if let Some(connection_identifier) = connection_identifier {
        let id = connection_identifier.id;
        let from = connection_identifier.addr;
        println!(
            "[{}] [{}] [{}]: {packet:?}",
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
