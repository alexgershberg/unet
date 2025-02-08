use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::io;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::sync::mpsc::{channel, Receiver, SendError, Sender, TryRecvError, TrySendError};
use std::sync::Mutex;

#[derive(Debug)]
struct Message {
    to: SocketAddr,
    from: SocketAddr,
    data: Vec<u8>,
}

#[derive(Debug)]
pub enum VirtualNetwork {
    V1(VirtualNetworkV1),
    V2(VirtualNetworkV2),
}

#[derive(Debug)]
pub struct VirtualNetworkV1 {
    pub tx: Sender<Vec<u8>>,
    pub rx: Receiver<Vec<u8>>,
}

pub struct VirtualSocket {
    addr: SocketAddr,
    target: Option<SocketAddr>,
    socket_rx: Receiver<(Vec<u8>, SocketAddr)>,
    network_tx: Sender<Message>,
    // send_callback: Box<Mutex<dyn FnMut() + 'a>>,
    // recv_callback: Box<Mutex<dyn FnMut() + 'a>>,
}

impl VirtualSocket {
    pub fn connect(&mut self, target: SocketAddr) {
        self.target = Some(target);
    }

    pub fn send(&self, buf: &[u8]) -> Result<usize, SendError<Message>> {
        let target = self.target.unwrap();
        self.send_to(buf, target)
    }

    pub fn send_to(&self, buf: &[u8], to: SocketAddr) -> Result<usize, SendError<Message>> {
        let message = Message {
            to,
            from: self.addr,
            data: buf.to_vec(),
        };
        self.network_tx.send(message)?;
        // let mut send_callback = self.send_callback.lock().unwrap();
        // send_callback();

        Ok(buf.len())
    }

    pub fn recv_from(&self) -> Result<(Vec<u8>, SocketAddr), TryRecvError> {
        // let mut recv_callback = self.recv_callback.lock().unwrap();
        // recv_callback();
        self.socket_rx.try_recv()
    }
}

impl Debug for VirtualSocket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VirtualSocket")
            .field("target", &self.target)
            .finish()
    }
}

#[derive(Debug)]
pub struct VirtualNetworkV2 {
    network_tx: Sender<Message>,
    network_rx: Receiver<Message>,
    connections: Mutex<HashMap<SocketAddr, Sender<(Vec<u8>, SocketAddr)>>>,
}

impl VirtualNetworkV2 {
    pub fn new() -> Self {
        let (network_tx, network_rx) = channel();
        Self {
            network_tx,
            network_rx,
            connections: Mutex::new(HashMap::new()),
        }
    }

    pub fn bind(&self, addr: SocketAddr) -> Option<VirtualSocket> {
        let (socket_tx, socket_rx) = channel();
        let mut connections = self.connections.lock().unwrap();
        match connections.entry(addr) {
            Entry::Occupied(entry) => {
                println!("Entry was occupied for addr: {addr}");
                return None;
            }
            Entry::Vacant(entry) => entry.insert(socket_tx),
        };
        let network_tx = self.network_tx.clone();

        // let send_callback = Box::new(Mutex::new(|| {
        //     self.tick();
        // }));
        // let recv_callback = Box::new(Mutex::new(|| {
        //     self.tick();
        // }));

        Some(VirtualSocket {
            addr,
            target: None,
            network_tx,
            socket_rx,
            // send_callback,
            // recv_callback,
        })
    }

    pub fn tick(&self) {
        while let Ok(message) = self.network_rx.try_recv() {
            let Message { to, from, data } = message;

            // println!("network::tick() | to: {to:?} | from: {from:?} | data: {data:?}");
            let mut connections = self.connections.lock().unwrap();
            if let Entry::Occupied(tx) = connections.entry(to) {
                let tx = tx.get();
                // println!("sending data to...");
                tx.send((data, from)).unwrap()
            } else {
                panic!()
            }
        }
    }

    pub fn v1(&self) -> VirtualNetworkV1 {
        todo!()
    }
}

#[derive(Debug)]
pub enum UnetSocket {
    Real(UdpSocket),
    Virtual(VirtualSocket),
}

impl UnetSocket {
    pub fn send_to(&self, buf: &[u8], to: SocketAddr) -> io::Result<usize> {
        match self {
            UnetSocket::Real(socket) => socket.send_to(buf, to),
            UnetSocket::Virtual(socket) => {
                socket.send_to(buf, to).unwrap();
                Ok(buf.len())
            }
        }
    }

    pub fn send(&self, buf: &[u8]) -> io::Result<usize> {
        match self {
            UnetSocket::Real(socket) => socket.send(buf),
            UnetSocket::Virtual(socket) => socket.send(buf).map_err(|e| match e {
                SendError(_) => {
                    unreachable!()
                }
            }),
        }
    }

    pub fn recv_from(&self, mut buf: &mut [u8]) -> Option<(usize, SocketAddr)> {
        match self {
            UnetSocket::Real(socket) => {
                let (n, from) = match socket.recv_from(buf) {
                    Ok((n, from)) => (n, from),
                    Err(e) => return None,
                };
                Some((n, from))
            }
            UnetSocket::Virtual(socket) => {
                let (output, from) = socket.recv_from().unwrap_or_else(|e| match e {
                    TryRecvError::Empty => (vec![], "0.0.0.0:0".parse().unwrap()),
                    TryRecvError::Disconnected => (vec![], "0.0.0.0:0".parse().unwrap()),
                });

                if output.is_empty() {
                    return None;
                }

                buf = &mut buf[..output.len()];
                buf.clone_from_slice(&*output);

                Some((output.len(), from))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::network::VirtualNetworkV2;

    #[test]
    fn basic() {
        let mut virtual_network = VirtualNetworkV2::new();
        let mut socket1 = virtual_network.bind("1.1.1.1:1".parse().unwrap()).unwrap();
        let socket2 = virtual_network.bind("2.2.2.2:2".parse().unwrap()).unwrap();

        socket1.connect("2.2.2.2:2".parse().unwrap());
        socket1.send(b"1").unwrap();
        socket1.send(b"2").unwrap();
        socket1.send(b"3").unwrap();
        let (data, from) = socket2.recv_from().unwrap();
        println!("1 data: {data:?} | from: {from}");

        let (data, from) = socket2.recv_from().unwrap();
        println!("2 data: {data:?} | from: {from}");

        let (data, from) = socket2.recv_from().unwrap();
        println!("3 data: {data:?} | from: {from}");

        socket2
            .send_to(b"hi world?", "1.1.1.1:1".parse().unwrap())
            .unwrap();
        let (data, from) = socket1.recv_from().unwrap();
        println!("data: {data:?} | from: {from}")
    }
}
