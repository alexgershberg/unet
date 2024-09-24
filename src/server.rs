use crate::debug::{client_connect_dbg, client_disconnect_dbg, recv_dbg, send_dbg};
use crate::packet::disconnect::{Disconnect, DisconnectReason};
use crate::packet::keep_alive::KeepAlive;
use crate::packet::{Packet, UnetId};
use crate::{BUF_SIZE, CLIENT_CONNECTION_TIMEOUT, KEEP_ALIVE_FREQUENCY, MAX_CONNECTIONS};
use std::collections::VecDeque;
use std::io;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
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
}

impl Connection {
    fn still_alive(&mut self) {
        self.time_since_last_packet_sent = Instant::now();
    }

    fn should_send_keep_alive(&self) -> bool {
        self.time_since_last_packet_sent.elapsed() > KEEP_ALIVE_FREQUENCY
    }

    fn reset_timeout(&mut self) {
        self.time_since_last_packet_received = Instant::now()
    }

    fn timed_out(&self) -> bool {
        self.time_since_last_packet_received.elapsed() > CLIENT_CONNECTION_TIMEOUT
    }
}

#[derive(Debug)]
pub struct UnetServer {
    socket: UdpSocket,
    pub connections: [Option<Connection>; MAX_CONNECTIONS],
    receive_buffer: VecDeque<(Packet, SocketAddr)>,
}

impl UnetServer {
    pub fn new<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let socket = UdpSocket::bind(addr)?;
        socket.set_nonblocking(true)?;
        let connections = [None; MAX_CONNECTIONS];

        Ok(Self {
            socket,
            connections,
            receive_buffer: VecDeque::new(),
        })
    }

    pub fn update(&mut self) {
        self.receive_packets();
        self.handle_packets();
        self.send_packets();
        self.kick_timed_out_connections();
    }

    fn send_to(
        &mut self,
        buf: &[u8],
        connection_identifier: ConnectionIdentifier,
    ) -> io::Result<usize> {
        let to = connection_identifier.addr;
        if let Some(index) = self.find_client_index_by_connection_identifier(connection_identifier)
        {
            if let Some(connection) = &mut self.connections[index] {
                connection.still_alive();
            }
        };
        self.socket.send_to(buf, to)
    }

    fn send_packet_to(&mut self, packet: Packet, to: ConnectionIdentifier) -> io::Result<usize> {
        send_dbg(packet, Some(to));

        let bytes = packet.as_bytes();
        self.send_to(&bytes, to)
    }

    fn send_packets(&mut self) {
        for connection in self.connections.into_iter().flatten() {
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
            self.handle_packet(packet, from)
        }
    }

    fn handle_packet(&mut self, packet: Packet, from: SocketAddr) {
        let header = packet.header();
        let connection_identifier = ConnectionIdentifier::new(header.client_id, from);
        // recv_dbg(packet, Some(connection_identifier));
        self.reset_timeout_for_connection(ConnectionIdentifier::new(header.client_id, from));
        match packet {
            Packet::ConnectionRequest(connection_request) => {
                if self.accept_connection(connection_identifier) {
                    let index = self
                        .find_client_index_by_connection_identifier(connection_identifier)
                        .unwrap();
                    client_connect_dbg(connection_identifier, index)
                }
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

        if let Some(index) = self.find_vacant_space() {
            self.connections[index] = Some(Connection {
                connection_identifier,
                time_since_last_packet_sent: Instant::now(),
                time_since_last_packet_received: Instant::now(),
            });
        } else {
            self.send_disconnect_packet(connection_identifier, DisconnectReason::ServerFull);

            return false;
        }

        self.send_challenge_packet(connection_identifier);

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
            // Find first vacant space
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

    fn reset_timeout_for_connection(&mut self, connection_identifier: ConnectionIdentifier) {
        if let Some(index) = self.find_client_index_by_connection_identifier(connection_identifier)
        {
            if let Some(connection) = &mut self.connections[index] {
                connection.reset_timeout()
            }
        }
    }

    fn kick_timed_out_connections(&mut self) {
        for connection in self.connections.into_iter().flatten() {
            if connection.timed_out() {
                self.kick(connection.connection_identifier, DisconnectReason::Timeout)
            }
        }
    }

    fn kick(&mut self, connection_identifier: ConnectionIdentifier, reason: DisconnectReason) {
        if let Some(index) = self.find_client_index_by_connection_identifier(connection_identifier)
        {
            assert!(self.connections[index].is_some());
            if let Some(connection) = self.connections[index].take() {
                client_disconnect_dbg(connection_identifier, index);
                self.send_disconnect_packet(connection.connection_identifier, reason)
            }
        } else {
            panic!("Just tried kicking a connection that doesn't exist? {connection_identifier:#?}")
        }
    }
}
