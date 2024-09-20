use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::ops::Index;
use std::sync::Arc;
use tokio::net::{lookup_host, ToSocketAddrs, UdpSocket};

struct UnetServer {
    name: String,
    socket: UdpSocket,
    connections: HashMap<SocketAddr, UnetClient>
}

impl UnetServer {
    async fn bind<A: ToSocketAddrs>(addr: A, name: String) -> io::Result<Self> {
        let socket = UdpSocket::bind(addr).await?;
        println!(
            "new UnetServer [{name}] | local: {}",
            socket.local_addr().unwrap(),
        );
        Ok(Self { name, socket, connections: HashMap::new() })
    }

    async fn accept(&mut self) -> UnetClient {
        loop {
            let mut buf: [u8; 2] = [0; 2];
            let (n, addr) = match self.socket.recv_from(&mut buf).await {
                Ok((n, addr)) => (n, addr),

                Err(_) => todo!(),
            };

            dbg!(&buf);

            dbg!(&self.connections);
            if !self.connections.contains_key(&addr) {
                println!("Connections doesn't contain {addr} | connections len: {}", self.connections.len());
                let client = UnetClient::connect(addr, format!("Accept | {}", self.connections.len())).await;
                client.socket.connect(addr).await.unwrap();
                self.connections.insert(addr, client.clone());
                return client;
            } else {
                println!("Connections contains {addr}");

                let entry = match self.connections.entry(addr) {
                    Entry::Occupied(occupied) => {occupied}
                    Entry::Vacant(_) => {continue;}
                };
                let client = entry.get();
                client.send_buf(&buf).await.unwrap();


                continue;
            }
        }
    }
}

#[derive(Clone, Debug)]
struct UnetClient {
    name: String,
    socket: Arc<UdpSocket>,
}

impl UnetClient {
    async fn connect<T: ToSocketAddrs>(target: T, name: String) -> Self {
        let socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();
        socket.connect(target).await.unwrap();
        println!(
            "new UnetClient [{name}] | local: {} | peer: {}",
            socket.local_addr().unwrap(),
            socket.peer_addr().unwrap()
        );
        // let target = socket.peer_addr().unwrap();
        // let target = lookup_host(target).await.unwrap().next().unwrap();
        Self { name, socket: Arc::new(socket) }
    }

    async fn send(&self, frame: Frame) -> io::Result<usize> {
        let mut buf: [u8; 64] = [0; 64];
        buf[0] = frame.0;
        self.socket.send(&buf).await
    }

    async fn send_buf(&self, buf: &[u8]) -> io::Result<usize> {
        println!("[{}] send_buf: {:?}", self.name, buf);
        self.socket.send(&buf).await
        // Ok(0)
    }

    async fn receive(&self) -> io::Result<Frame> {
        let mut buf: [u8; 2] = [0; 2];
        println!("before UnetConnection::receive() | buf: {buf:#?}");
        let n= self.socket.recv(&mut buf).await?;
        panic!("yolo");
        println!("after UnetConnection::receive() | buf: {buf:#?}");
        Ok(Frame(0))
    }
}

struct Frame(u8);

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use crate::{Frame, UnetClient, UnetServer};

    async fn handle_connection(connection: UnetClient) {
        println!("Got connection!!!: {connection:#?}");
        loop {
            println!("Before receive");
            let frame = connection.receive().await.unwrap();
            println!("After receive");
        }

    }

    #[tokio::test]
    async fn create_server() {
        let addr = "127.0.0.1:10001";
        let handle1 = tokio::spawn(async move {
            let mut server = UnetServer::bind(addr, "Main Server".to_string()).await.unwrap();
            loop {
                let connection = server.accept().await;
                tokio::spawn(async { handle_connection(connection).await });
            }
        });
        let handle2 = tokio::spawn(async move {
            let unet = UnetClient::connect(addr, "Client 1".to_string()).await;
            let n = unet.send(Frame(11)).await.unwrap();
            tokio::time::sleep(Duration::from_millis(8000)).await;
            let n = unet.send(Frame(23)).await.unwrap();
        });

        let (r1, r2) = tokio::join!(handle1, handle2);
    }
}
