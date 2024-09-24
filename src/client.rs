use crate::debug::{recv_dbg, send_dbg};
use crate::packet::challenge_response::ChallengeResponse;
use crate::packet::connection_request::ConnectionRequest;
use crate::packet::disconnect::DisconnectReason;
use crate::packet::keep_alive::KeepAlive;
use crate::packet::{Packet, UnetId};
use crate::{BUF_SIZE, KEEP_ALIVE_FREQUENCY, SERVER_NOT_RESPONDING_TIMEOUT};
use colored::Colorize;
use std::collections::VecDeque;
use std::io;
use std::net::{ToSocketAddrs, UdpSocket};
use std::process::exit;
use std::time::Instant;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ClientState {
    Disconnected(DisconnectReason),
    SendingConnectionRequest,
    SendingConnectionResponse,
    Connected,
}

#[derive(Debug)]
pub struct UnetClient {
    pub id: UnetId,
    socket: UdpSocket,
    state: ClientState,
    send_queue: VecDeque<Packet>,
    pub time_since_last_packet_sent: Instant,
    pub time_since_last_packet_received: Instant,
}

impl UnetClient {
    pub fn new<T: ToSocketAddrs>(target: T) -> io::Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_nonblocking(true)?;
        socket.connect(target).unwrap();

        let id = UnetId::new();

        let client = Self {
            id,
            socket,
            state: ClientState::SendingConnectionRequest,
            send_queue: VecDeque::new(),
            time_since_last_packet_sent: Instant::now(),
            time_since_last_packet_received: Instant::now(),
        };

        Ok(client)
    }

    pub fn update(&mut self) {
        self.receive_packets();
        self.send_packets();

        if !self.check_server_response_ok() {
            println!("[{}]", "Server not responding".truecolor(255, 0, 255));
            exit(0)
        }
    }

    pub fn send(&mut self, packet: Packet) {
        self.send_queue.push_back(packet)
    }

    fn internal_send(&mut self, buf: &[u8]) -> io::Result<usize> {
        let res = self.socket.send(buf);
        self.time_since_last_packet_sent = Instant::now();
        res
    }

    fn send_packet(&mut self, packet: Packet) -> io::Result<usize> {
        send_dbg(packet, None);
        let bytes = packet.as_bytes();
        self.internal_send(&bytes)
    }

    fn send_packets(&mut self) {
        match self.state {
            ClientState::SendingConnectionRequest => {
                self.send_packet(Packet::ConnectionRequest(ConnectionRequest::new(self.id)))
                    .unwrap();
            }
            ClientState::SendingConnectionResponse => {
                self.send_packet(Packet::ChallengeResponse(ChallengeResponse::new(self.id)))
                    .unwrap();
            }
            ClientState::Connected => {
                if self.send_queue.is_empty() && self.should_send_keep_alive() {
                    self.send_packet(Packet::KeepAlive(KeepAlive::new(self.id)))
                        .unwrap();
                    return;
                }

                while let Some(packet) = self.send_queue.pop_front() {
                    self.send_packet(packet).unwrap();
                }
            }
            ClientState::Disconnected(reason) => {
                match reason {
                    DisconnectReason::Timeout => {
                        println!("[{}]", "Client timed out".truecolor(255, 0, 255));
                    }
                    DisconnectReason::ServerFull => {
                        println!("[{}]", "Server was full".truecolor(255, 0, 255));
                    }
                }
                exit(0)
            }
        }
    }

    fn receive(&self, buf: &mut [u8]) -> Option<usize> {
        let (n, from) = match self.socket.recv_from(buf) {
            Ok((n, from)) => (n, from),
            Err(e) => return None,
        };

        Some(n)
    }

    fn receive_packet(&self) -> Option<Packet> {
        let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];
        let n = self.receive(&mut buf)?;
        assert_ne!(n, 0);
        Packet::from_bytes(&buf[..n])
    }

    fn receive_packets(&mut self) {
        while let Some(packet) = self.receive_packet() {
            self.handle_packet(packet);
        }
    }

    fn handle_packet(&mut self, packet: Packet) {
        recv_dbg(packet, None);
        self.reset_timeout();
        match packet {
            Packet::ChallengeRequest => {
                if self.state == ClientState::SendingConnectionRequest {
                    self.state = ClientState::SendingConnectionResponse;
                }
            }
            Packet::Disconnect(disconnect) => {
                if !matches!(self.state, ClientState::Disconnected(..)) {
                    self.state = ClientState::Disconnected(disconnect.reason)
                }
            }
            Packet::KeepAlive(keep_alive) => {
                if self.state == ClientState::SendingConnectionResponse {
                    self.state = ClientState::Connected;
                }
            }
            Packet::Data(_) => {}
            _ => {
                panic!("Client should never get this packet: {packet:#?}");
            }
        };
    }

    fn reset_timeout(&mut self) {
        self.time_since_last_packet_received = Instant::now()
    }

    fn check_server_response_ok(&mut self) -> bool {
        self.time_since_last_packet_received.elapsed() < SERVER_NOT_RESPONDING_TIMEOUT
    }

    fn should_send_keep_alive(&self) -> bool {
        self.time_since_last_packet_sent.elapsed() > KEEP_ALIVE_FREQUENCY
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn basic() {}
}
