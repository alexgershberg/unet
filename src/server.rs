use crate::packet::{Packet, UnetId};
use crate::BUF_SIZE;
use std::io;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

const MAX_NUMER_OF_CONNECTIONS: usize = 10;

#[derive(Copy, Clone, Debug)]
struct Connection {
    id: UnetId,
    addr: SocketAddr,
}

#[derive(Debug)]
pub struct UnetServer {
    socket: UdpSocket,
    connections: [Option<Connection>; 10],
}

impl UnetServer {
    pub fn new<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let socket = UdpSocket::bind(addr)?;
        socket.set_nonblocking(true)?;
        let connections = [None; 10];

        Ok(Self {
            socket,
            connections,
        })
    }

    pub fn update(&mut self) {
        self.receive_packets();
        self.send_packets();
    }

    fn send_to(&self, buf: &[u8], to: SocketAddr) -> io::Result<usize> {
        println!("[send]: {buf:#?}");
        self.socket.send_to(buf, to)
    }

    fn send_packet_to(&self, packet: Packet, to: SocketAddr) -> io::Result<usize> {
        let bytes = packet.as_bytes();
        self.send_to(&bytes, to)
    }

    fn send_packets(&self) {}

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
            self.handle_packet(packet, from)
        }
    }

    fn handle_packet(&mut self, packet: Packet, from: SocketAddr) {
        println!("[{from:?}]: {packet:?}");
        match packet {
            Packet::ConnectionRequest(connection_request) => {
                if self
                    .find_client_index_by_id(connection_request.client_id)
                    .is_some()
                {
                    // Already connected, just ignore
                    return;
                }

                if let Some(index) = self.find_vacant_space() {
                    self.connections[index] = Some(Connection {
                        id: connection_request.client_id,
                        addr: from,
                    });
                } else {
                    panic!("No more space, all connections are occupied!")
                }

                self.send_challenge_packet(from);
            }
            Packet::ChallengeRequest => {}
            Packet::ChallengeResponse => {
                self.send_keep_alive_packet(from);
            }
            Packet::Disconnect => {}
            Packet::KeepAlive => {}
            Packet::Data => {}
            _ => {
                panic!("server got weird packet: {packet:#?}");
            }
        };
    }

    fn find_client_index_by_id(&self, id: UnetId) -> Option<usize> {
        let mut index = None;
        for (idx, connection) in self.connections.iter().enumerate() {
            if let Some(connection) = connection {
                if connection.id == id {
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

    fn send_challenge_packet(&self, to: SocketAddr) {
        let packet = Packet::ChallengeRequest;
        self.send_packet_to(packet, to).unwrap();
    }

    fn send_keep_alive_packet(&self, to: SocketAddr) {
        let packet = Packet::KeepAlive;
        self.send_packet_to(packet, to).unwrap();
    }
}
