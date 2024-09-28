pub mod connection;

use crate::config::server::ServerConfig;
use crate::debug::{client_connect_dbg, client_disconnect_dbg, recv_dbg, send_dbg, YELLOW};
use crate::packet::disconnect::{Disconnect, DisconnectReason};
use crate::packet::keep_alive::KeepAlive;
use crate::packet::Packet;
use crate::server::connection::{Connection, ConnectionIdentifier};
use crate::{BUF_SIZE, MAX_CONNECTIONS};
use colored::Colorize;
use std::collections::VecDeque;
use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::thread::sleep;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct UnetServer {
    socket: UdpSocket,
    pub connections: Vec<Option<Connection>>,
    receive_buffer: VecDeque<(Packet, SocketAddr)>,
    config: ServerConfig,
    previous: Instant,
    lag: u128,
}

impl UnetServer {
    pub fn new() -> io::Result<Self> {
        Self::from_config(ServerConfig::default())
    }

    pub fn from_config(config: ServerConfig) -> io::Result<Self> {
        let socket = UdpSocket::bind(config.addr)?;
        socket.set_nonblocking(true)?;
        let connections = vec![None; MAX_CONNECTIONS];

        let server = Self {
            socket,
            connections,
            receive_buffer: VecDeque::new(),
            config,
            previous: Instant::now(),
            lag: 0,
        };

        server_starting_dbg(&server);

        Ok(server)
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let elapsed = self.previous.elapsed();

        self.lag += elapsed.as_millis();
        if self.lag >= self.config.ms_per_update {
            self.print_state();
            self.reset_connections_stats();
            self.receive_packets();
            self.handle_packets();
            self.send_packets();
            self.kick_timed_out_connections();
            self.kick_spamming_connections();

            self.lag -= self.config.ms_per_update;
        } else {
            sleep(Duration::from_millis(
                (self.config.ms_per_update - self.lag) as u64,
            ));
        }

        self.previous = now;
    }

    fn send_to(
        &mut self,
        buf: &[u8],
        connection_identifier: ConnectionIdentifier,
    ) -> io::Result<usize> {
        let to = connection_identifier.addr;
        if let Some(connection) = self.get_connection(connection_identifier) {
            connection.still_alive();
        }

        self.socket.send_to(buf, to)
    }

    fn send_packet_to(&mut self, packet: Packet, to: ConnectionIdentifier) -> io::Result<usize> {
        if self.config.send_debug {
            send_dbg(packet, Some(to));
        }

        let bytes = packet.as_bytes();
        self.send_to(&bytes, to)
    }

    fn send_packets(&mut self) {
        for connection in self.connections.clone().into_iter().flatten() {
            if connection.should_send_keep_alive() {
                self.send_keep_alive_packet(connection.connection_identifier);
            }
        }
    }

    fn receive(&self, buf: &mut [u8]) -> Option<(usize, SocketAddr)> {
        let (n, from) = match self.socket.recv_from(buf) {
            Ok((n, from)) => (n, from),
            Err(e) => return None,
        };

        Some((n, from))
    }

    fn receive_packet(&self) -> Option<(Packet, SocketAddr)> {
        let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];
        let (n, from) = self.receive(&mut buf)?;
        assert_ne!(n, 0);

        Some((Packet::from_bytes(&buf[..n])?, from))
    }

    fn receive_packets(&mut self) {
        while let Some((packet, from)) = self.receive_packet() {
            self.receive_buffer.push_back((packet, from));
        }
    }

    fn handle_packets(&mut self) {
        while let Some((packet, from)) = self.receive_buffer.pop_back() {
            self.handle_packet(packet, from);
        }
    }

    fn handle_packet(&mut self, packet: Packet, from: SocketAddr) {
        let header = packet.header();
        let connection_identifier = ConnectionIdentifier::new(header.client_id, from);

        let recv_debug = self.config.recv_debug;

        if let Some(connection) = self.get_connection(connection_identifier) {
            if recv_debug {
                recv_dbg(packet, Some(connection_identifier), Some(connection.index));
            }

            if connection.is_packet_out_of_order(packet) {
                return;
            }
            connection.reset_timeout();
            connection.packets_per_update_received += 1;
            connection.packet_sequence += 1;
        } else if recv_debug {
            recv_dbg(packet, Some(connection_identifier), None);
        }

        match packet {
            Packet::ConnectionRequest(connection_request) => {
                self.accept_connection(connection_identifier);
            }
            Packet::ChallengeResponse(challenge_response) => {
                let header = challenge_response.header;
                let connection_identifier = ConnectionIdentifier::new(header.client_id, from);
                self.send_keep_alive_packet(connection_identifier);
            }
            Packet::Disconnect(disconnect) => {
                let header = disconnect.header;
                let connection_identifier = ConnectionIdentifier::new(header.client_id, from);
                self.kick(connection_identifier, disconnect.reason);
            }
            Packet::KeepAlive(_) => {}
            Packet::Data(data) => {}
            _ => {
                panic!("server got weird packet: {packet:#?}");
            }
        };
    }

    fn accept_connection(&mut self, connection_identifier: ConnectionIdentifier) -> bool {
        if self
            .find_client_index_by_connection_identifier(connection_identifier)
            .is_some()
        {
            // Already connected, just ignore
            return false;
        }

        let Some(index) = self.find_vacant_space() else {
            self.send_disconnect_packet(connection_identifier, DisconnectReason::ServerFull);
            return false;
        };

        self.connections[index] = Some(Connection {
            connection_identifier,
            time_since_last_packet_sent: Instant::now(),
            time_since_last_packet_received: Instant::now(),
            packets_per_update_received: 0,
            packet_sequence: 0,
            index,
        });

        self.send_challenge_packet(connection_identifier);

        client_connect_dbg(connection_identifier, index);
        true
    }

    fn find_client_index_by_connection_identifier(
        &self,
        connection_identifier: ConnectionIdentifier,
    ) -> Option<usize> {
        let id = connection_identifier.id;
        let addr = connection_identifier.addr;
        let mut index = None;
        for (idx, connection) in self.connections.iter().enumerate() {
            if let Some(connection) = connection {
                let identifier = connection.connection_identifier;
                if identifier.id == id && identifier.addr == addr {
                    index = Some(idx);
                    break;
                }
            }
        }

        index
    }

    fn find_vacant_space(&self) -> Option<usize> {
        let mut index = None;
        for (idx, connection) in self.connections.iter().enumerate() {
            if connection.is_none() {
                index = Some(idx);
                break;
            }
        }

        index
    }

    fn send_challenge_packet(&mut self, connection_identifier: ConnectionIdentifier) {
        let packet = Packet::ChallengeRequest;
        self.send_packet_to(packet, connection_identifier).unwrap();
    }

    fn send_keep_alive_packet(&mut self, connection_identifier: ConnectionIdentifier) {
        let client_id = connection_identifier.id;
        let packet = Packet::KeepAlive(KeepAlive::new(client_id));
        self.send_packet_to(packet, connection_identifier).unwrap();
    }

    fn send_disconnect_packet(
        &mut self,
        connection_identifier: ConnectionIdentifier,
        reason: DisconnectReason,
    ) {
        let client_id = connection_identifier.id;
        let packet = Packet::Disconnect(Disconnect::new(client_id, reason));
        self.send_packet_to(packet, connection_identifier).unwrap();
    }

    fn get_connection(
        &mut self,
        connection_identifier: ConnectionIdentifier,
    ) -> Option<&mut Connection> {
        let index = self.find_client_index_by_connection_identifier(connection_identifier)?;

        let Some(connection) = &mut self.connections[index] else {
            return None;
        };

        Some(connection)
    }

    fn kick_timed_out_connections(&mut self) {
        for connection in self.connections.clone().into_iter().flatten() {
            if connection.timed_out() {
                self.kick(connection.connection_identifier, DisconnectReason::Timeout)
            }
        }
    }

    fn kick_spamming_connections(&mut self) {
        for connection in &mut self.connections.clone().iter().flatten() {
            if connection.is_spamming(self.config.max_packets_per_update) {
                self.kick(connection.connection_identifier, DisconnectReason::Spam)
            }
        }
    }

    fn kick(&mut self, connection_identifier: ConnectionIdentifier, reason: DisconnectReason) {
        if let Some(index) = self.find_client_index_by_connection_identifier(connection_identifier)
        {
            if let Some(connection) = self.connections[index].take() {
                client_disconnect_dbg(connection_identifier, index);
                self.send_disconnect_packet(connection.connection_identifier, reason)
            }
        } else {
            panic!("Just tried kicking a connection that doesn't exist? {connection_identifier:#?}")
        }
    }

    fn reset_connections_stats(&mut self) {
        for connection in self.connections.iter_mut().flatten() {
            connection.packets_per_update_received = 0;
        }
    }

    fn print_state(&self) {
        for connection in self.connections.iter().flatten() {
            connection_state_dbg(connection);
        }
    }
}

pub fn server_starting_dbg(server: &UnetServer) {
    println!(
        "Starting server on {}",
        server
            .config
            .addr
            .to_string()
            .truecolor(YELLOW.r, YELLOW.g, YELLOW.b),
    );

    dbg!(server.config);
}

pub fn connection_state_dbg(connection: &Connection) {
    println!(
        r#"{:>16x}:    time_since_last_packet_received:   {:.2?}
                     time_since_last_packet_send:       {:.2?}
                "#,
        connection.connection_identifier.id.0,
        connection.time_since_last_packet_received.elapsed(),
        connection.time_since_last_packet_sent.elapsed()
    );
}

#[cfg(test)]
mod tests {
    use crate::server::connection::Connection;
    use crate::server::connection_state_dbg;

    #[test]
    fn preview() {
        let connection = Connection::default();
        connection_state_dbg(&connection);
    }
}
