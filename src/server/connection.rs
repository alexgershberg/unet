use crate::packet::{Packet, UnetId};
use crate::rolling_average::RollingAverage;
use crate::tick::Tick;
use crate::{DEFAULT_CLIENT_CONNECTION_TIMEOUT, DEFAULT_KEEP_ALIVE_FREQUENCY};
use std::net::SocketAddr;

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

#[derive(Clone, Debug)]
pub struct Connection {
    pub connection_identifier: ConnectionIdentifier,
    pub ticks_since_last_packet_sent: Tick,
    pub ticks_since_last_packet_received: Tick,
    pub rolling_packets_per_tick_received: RollingAverage,
    pub packets_per_tick_received: f32, // Packets received from Connection
    pub packet_sequence: u64,
    pub index: usize,
    pub client_connection_timeout: Tick,
    pub connected: bool,
}

impl Connection {
    pub fn new(connection_identifier: ConnectionIdentifier) -> Self {
        Self {
            connection_identifier,
            ticks_since_last_packet_sent: Tick { value: 0.0 },
            ticks_since_last_packet_received: Tick { value: 0.0 },
            rolling_packets_per_tick_received: RollingAverage::new(25),
            packets_per_tick_received: 0.0,
            packet_sequence: 0,
            index: 0,
            client_connection_timeout: DEFAULT_CLIENT_CONNECTION_TIMEOUT,
            connected: false,
        }
    }
    pub fn still_alive(&mut self) {
        self.ticks_since_last_packet_sent.value = 0.0;
    }

    pub fn should_send_keep_alive(&self) -> bool {
        self.ticks_since_last_packet_sent >= DEFAULT_KEEP_ALIVE_FREQUENCY
    }

    pub fn reset_timeout(&mut self) {
        self.ticks_since_last_packet_received.value = 0.0;
    }

    pub fn timed_out(&self) -> bool {
        self.ticks_since_last_packet_received >= self.client_connection_timeout
    }

    pub fn is_spamming(&self, max_packets_per_tick: f32) -> bool {
        self.rolling_packets_per_tick_received.value() >= max_packets_per_tick
    }

    pub fn is_packet_out_of_order(&self, packet: Packet) -> bool {
        let header = packet.header();
        self.packet_sequence >= header.sequence
    }
}
