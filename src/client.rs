use crate::config::client::ClientConfig;
use crate::debug::{recv_dbg, send_dbg, BLUE};
use crate::network::Network::{Real, Virtual};
use crate::network::{Network, VirtualNetwork};
use crate::packet::challenge_response::ChallengeResponse;
use crate::packet::connection_request::ConnectionRequest;
use crate::packet::disconnect::{Disconnect, DisconnectReason};
use crate::packet::keep_alive::KeepAlive;
use crate::packet::{Packet, UnetId};
use crate::tick::Tick;
use crate::{BUF_SIZE, DEFAULT_KEEP_ALIVE_FREQUENCY, DEFAULT_SERVER_NOT_RESPONDING_TIMEOUT};
use colored::Colorize;
use std::collections::VecDeque;
use std::io;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::process::exit;
use std::sync::mpsc::TryRecvError;
use std::thread::sleep;
use std::time::{Duration, Instant};

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
    target: SocketAddr,
    pub server_index: Option<usize>,
    network: Network,
    pub state: ClientState,
    pub send_queue: VecDeque<Packet>,
    pub config: ClientConfig,
    pub ticks_since_last_packet_sent: Tick,
    pub ticks_since_last_packet_received: Tick,
    pub sequence: u64,
    previous: Instant,
    lag: u128,
    terminate: bool,
}

impl UnetClient {
    pub fn new() -> io::Result<Self> {
        Self::from_config(ClientConfig::new())
    }

    pub fn from_config(mut config: ClientConfig) -> io::Result<Self> {
        let target = config.target;

        let network = if let Some(virtual_network) = config.virtual_network.take() {
            Virtual(virtual_network)
        } else {
            let socket = UdpSocket::bind("0.0.0.0:0")?;
            socket.set_nonblocking(true)?;
            socket.connect(target)?;
            Real(socket)
        };

        let mut client_id = UnetId::new();
        if let Some(id) = config.id {
            client_id = id;
        }

        let client = Self {
            id: client_id,
            target: target.to_socket_addrs().unwrap().next().unwrap(),
            server_index: None,
            network,
            state: ClientState::SendingConnectionRequest,
            send_queue: VecDeque::new(),
            config,
            ticks_since_last_packet_sent: Tick { value: 0.0 },
            ticks_since_last_packet_received: Tick { value: 0.0 },
            sequence: 0,
            previous: Instant::now(),
            lag: 0,
            terminate: false,
        };

        connecting_dbg(client_id, target.to_socket_addrs().unwrap().next().unwrap());
        Ok(client)
    }

    pub fn update(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = self.previous.elapsed();

        self.lag += elapsed.as_millis();
        if self.lag >= self.config.ms_per_tick {
            if !self.tick() {
                return false;
            }

            self.lag -= self.config.ms_per_tick;
        } else {
            sleep(Duration::from_millis(
                (self.config.ms_per_tick - self.lag) as u64,
            ));
        }

        self.previous = now;
        true
    }

    pub fn tick(&mut self) -> bool {
        if self.terminate {
            return false;
        }

        self.print_state();
        self.receive_packets();
        self.send_packets();

        if !self.check_server_response_ok() {
            disconnect_dbg(self.id, self.target, "Server not responding".to_string());
            self.exit();
        }

        self.ticks_since_last_packet_sent.value += 1.0;
        self.ticks_since_last_packet_received.value += 1.0;

        true
    }

    pub fn send(&mut self, packet: Packet) {
        self.send_queue.push_back(packet)
    }

    fn internal_send(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.network.send(buf)
    }

    pub fn send_packet(&mut self, mut packet: Packet) -> io::Result<usize> {
        packet.set_sequence(self.sequence);

        if self.config.send_debug {
            send_dbg(packet, None, None);
        }

        let bytes = packet.as_bytes();
        let res = self.internal_send(&bytes);
        self.ticks_since_last_packet_sent.value = 0.0;
        self.sequence += 1;

        res
    }

    pub fn send_packets(&mut self) {
        match self.state {
            ClientState::SendingConnectionRequest => {
                self.send_connection_request_packet().unwrap();
            }
            ClientState::SendingConnectionResponse => {
                self.send_connection_response_packet().unwrap();
            }
            ClientState::Connected => {
                if self.send_queue.is_empty() && self.should_send_keep_alive() {
                    self.send_keep_alive_packet().unwrap();
                    return;
                }

                while let Some(packet) = self.send_queue.pop_front() {
                    self.send_packet(packet).unwrap();
                }
            }
            ClientState::Disconnected(reason) => {
                match reason {
                    DisconnectReason::Timeout => {
                        disconnect_dbg(self.id, self.target, "Client timed out".to_string());
                    }
                    DisconnectReason::ServerFull => {
                        disconnect_dbg(self.id, self.target, "Server was full".to_string());
                    }
                    DisconnectReason::Spam => {
                        disconnect_dbg(
                            self.id,
                            self.target,
                            "Kicked for spamming the server".to_string(),
                        );
                    },
                    DisconnectReason::ConnectionResetByPeer => {
                        disconnect_dbg(self.id, self.target, "Connection reset by peer".to_string());
                    }
                }
                self.exit()
            }
        }
    }

    pub fn send_connection_request_packet(&mut self) -> io::Result<usize> {
        self.send_packet(Packet::ConnectionRequest(ConnectionRequest::new(self.id)))
    }

    pub fn send_connection_response_packet(&mut self) -> io::Result<usize> {
        self.send_packet(Packet::ChallengeResponse(ChallengeResponse::new(self.id)))
    }

    pub fn send_keep_alive_packet(&mut self) -> io::Result<usize> {
        self.send_packet(Packet::KeepAlive(KeepAlive::new(self.id)))
    }
    
    pub fn send_disconnect_packet(&mut self) -> io::Result<usize> {
        self.send_packet(Packet::Disconnect(Disconnect::new(self.id, DisconnectReason::ConnectionResetByPeer)))
    }

    fn receive(&self, buf: &mut [u8]) -> Option<usize> {
        let (n, _from) = self.network.recv_from(buf)?;
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
        if self.config.recv_debug {
            recv_dbg(packet, None, None);
        }

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
                    connected_dbg(self.id, self.target);
                }
            }
            Packet::Data(_) => {}
            _ => {
                panic!("Client should never get this packet: {packet:#?}");
            }
        };
    }

    fn reset_timeout(&mut self) {
        self.ticks_since_last_packet_received.value = 0.0;
    }

    fn check_server_response_ok(&mut self) -> bool {
        if let Some(server_not_responding_timeout) = self.config.server_not_responding_timeout {
            self.ticks_since_last_packet_received <= server_not_responding_timeout
        } else {
            // No timeout specified, we keep going forever
            true
        }
    }

    fn should_send_keep_alive(&self) -> bool {
        self.ticks_since_last_packet_sent >= DEFAULT_KEEP_ALIVE_FREQUENCY
    }

    fn exit(&mut self) {
        self.terminate = true;
    }

    fn print_state(&self) {}
}

pub fn connecting_dbg(id: UnetId, to: SocketAddr) {
    println!(
        "{} connecting to server {}...",
        format!("{:16x}", id.0).truecolor(BLUE.r, BLUE.g, BLUE.b),
        to.to_string().truecolor(255, 215, 0),
    )
}

pub fn connected_dbg(id: UnetId, to: SocketAddr) {
    println!(
        "{} connected!",
        format!("{:16x}", id.0).truecolor(BLUE.r, BLUE.g, BLUE.b),
    )
}

pub fn disconnect_dbg(id: UnetId, to: SocketAddr, disconnect_message: String) {
    println!(
        "{} disconnected from server {} | {}",
        format!("{:16x}", id.0).truecolor(BLUE.r, BLUE.g, BLUE.b),
        to.to_string().truecolor(255, 215, 0),
        disconnect_message
    )
}

#[cfg(test)]
mod tests {
    use crate::client::{connected_dbg, connecting_dbg, disconnect_dbg};
    use crate::packet::UnetId;

    #[test]
    fn preview() {
        connecting_dbg(UnetId(u64::MAX), "255.255.255.255:65535".parse().unwrap());
        connecting_dbg(UnetId(0xdeadbeef), "0.0.0.0:0".parse().unwrap());
    }
}
