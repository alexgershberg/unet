use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc::{Receiver, Sender, TryRecvError};

#[derive(Debug)]
pub struct VirtualNetwork {
    pub tx: Sender<Vec<u8>>,
    pub rx: Receiver<Vec<u8>>,
}

#[derive(Debug)]
pub enum Network {
    Real(UdpSocket),
    Virtual(VirtualNetwork),
}

impl Network {
    pub fn send_to(&self, buf: &[u8], to: SocketAddr) -> io::Result<usize> {
        match self {
            Network::Real(socket) => socket.send_to(buf, to),
            Network::Virtual(virtual_network) => {
                virtual_network.tx.send(buf.to_vec()).unwrap();
                Ok(buf.len())
            }
        }
    }

    pub fn send(&self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Network::Real(socket) => socket.send(buf),
            Network::Virtual(virtual_network) => {
                virtual_network.tx.send(buf.to_vec()).unwrap();
                Ok(buf.len())
            }
        }
    }

    pub fn recv_from(&self, mut buf: &mut [u8]) -> Option<(usize, SocketAddr)> {
        match self {
            Network::Real(socket) => {
                let (n, from) = match socket.recv_from(buf) {
                    Ok((n, from)) => (n, from),
                    Err(e) => return None,
                };
                Some((n, from))
            }
            Network::Virtual(virtual_network) => {
                let output = &virtual_network.rx.try_recv().unwrap_or_else(|e| match e {
                    TryRecvError::Empty => {
                        vec![]
                    }
                    TryRecvError::Disconnected => {
                        unreachable!()
                    }
                });

                if output.is_empty() {
                    return None;
                }

                buf = &mut buf[..output.len()];
                buf.clone_from_slice(output);

                Some((output.len(), "0.0.0.0:0".parse().unwrap()))
            }
        }
    }
}
