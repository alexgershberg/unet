use crate::packet::{ConnectionRequest, Packet, UnetId};
use crate::BUF_SIZE;
use std::io;
use std::net::{ToSocketAddrs, UdpSocket};
use std::rc::Rc;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ClientState {
    Disconnected,
    SendingConnectionRequest,
    SendingConnectionResponse,
    Connected,
}

#[derive(Clone, Debug)]
pub struct UnetClient {
    id: UnetId,
    socket: Rc<UdpSocket>,
    state: ClientState,
}

impl UnetClient {
    pub fn new<T: ToSocketAddrs>(target: T) -> io::Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_nonblocking(true)?;
        socket.connect(target).unwrap();

        let id = UnetId::new();

        let client = Self {
            id,
            socket: Rc::new(socket),
            state: ClientState::SendingConnectionRequest,
        };

        Ok(client)
    }

    pub fn update(&mut self) {
        self.receive_packets();
        self.send_packets();
    }

    fn send(&self, buf: &[u8]) -> io::Result<usize> {
        self.socket.send(buf)
    }

    fn send_packet(&self, packet: Packet) -> io::Result<usize> {
        println!("[send]: {packet:?}");
        let bytes = packet.as_bytes();
        self.send(&bytes)
    }

    fn send_packets(&self) {
        match self.state {
            ClientState::SendingConnectionRequest => {
                self.send_packet(Packet::ConnectionRequest(ConnectionRequest::new(self.id)))
                    .unwrap();
            }
            ClientState::SendingConnectionResponse => {
                self.send_packet(Packet::ChallengeResponse).unwrap();
            }
            ClientState::Connected => {
                self.send_packet(Packet::KeepAlive).unwrap();
            }
            ClientState::Disconnected => {}
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
        println!("[recv]: {packet:?}");
        match packet {
            Packet::ChallengeRequest => {
                if self.state == ClientState::SendingConnectionRequest {
                    self.state = ClientState::SendingConnectionResponse;
                }
            }
            Packet::Disconnect => {
                if self.state != ClientState::Disconnected {
                    self.state = ClientState::Disconnected;
                }
            }
            Packet::KeepAlive => {
                if self.state == ClientState::SendingConnectionResponse {
                    self.state = ClientState::Connected;
                }
            }
            Packet::Data => {}
            _ => {
                panic!("Client should never get this packet: {packet:#?}");
            }
        };
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn basic() {}
}
