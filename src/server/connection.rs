use crate::packet::{Packet, UnetId};
use crate::{CLIENT_CONNECTION_TIMEOUT, KEEP_ALIVE_FREQUENCY};
use std::net::SocketAddr;
use std::time::Instant;

#[derive(Copy, Clone, Debug)]
pub struct ConnectionIdentifier {
    pub id: UnetId,
    pub addr: SocketAddr,
}
impl ConnectionIdentifier {
    pub fn new(id: UnetId, addr: SocketAddr) -> Self {
        Self { id, addr }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Connection {
    pub connection_identifier: ConnectionIdentifier,
    pub time_since_last_packet_sent: Instant,
    pub time_since_last_packet_received: Instant,
    pub packets_per_update_received: u32, // From Connection
    pub packet_sequence: u64,
    pub index: usize,
}

impl Connection {
    pub fn still_alive(&mut self) {
        self.time_since_last_packet_sent = Instant::now();
    }

    pub fn should_send_keep_alive(&self) -> bool {
        self.time_since_last_packet_sent.elapsed() > KEEP_ALIVE_FREQUENCY
    }

    pub fn reset_timeout(&mut self) {
        self.time_since_last_packet_received = Instant::now();
    }

    pub fn timed_out(&self) -> bool {
        self.time_since_last_packet_received.elapsed() > CLIENT_CONNECTION_TIMEOUT
    }

    pub fn is_spamming(&self, max_packets_per_update: f32) -> bool {
        self.packets_per_update_received as f32 >= max_packets_per_update
    }

    pub fn is_packet_out_of_order(&self, packet: Packet) -> bool {
        let header = packet.header();
        self.packet_sequence >= header.sequence
    }
}

impl Default for Connection {
    fn default() -> Self {
        Self {
            connection_identifier: ConnectionIdentifier::new(
                UnetId(0),
                SocketAddr::new("0.0.0.0".parse().unwrap(), 0),
            ),
            time_since_last_packet_sent: Instant::now(),
            time_since_last_packet_received: Instant::now(),
            packets_per_update_received: 0,
            packet_sequence: 0,
            index: 0,
        }
    }
}
