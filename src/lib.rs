use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::ops::Index;
use std::sync::Arc;
use tokio::net::{lookup_host, ToSocketAddrs, UdpSocket};

enum Packet {
    Reliable,
    Unreliable
}

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

    async fn accept(&mut self) -> (UnetClient, SocketAddr) {
        loop {
            let (connection, addr) = self.handshake().await;

            // let client = UnetClient::connect(addr, format!("Accept | {}", self.connections.len())).await;
            // client.socket.connect(addr).await.unwrap();
                
            continue;
        }
    }
    
    async fn handshake(&self) -> (UnetClient, SocketAddr) {
       todo!() 
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
        Self { name, socket: Arc::new(socket) }
    }

    async fn send(&self, packet: Packet) -> io::Result<usize> {
        todo!()
    }

    async fn receive(&self) -> io::Result<Packet> {
        todo!()
    }
}

struct Frame(u8);

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use crate::{Frame, Packet, UnetClient, UnetServer};

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
                let (connection, _) = server.accept().await;
                tokio::spawn(async { handle_connection(connection).await });
            }
        });
        let handle2 = tokio::spawn(async move {
            let unet = UnetClient::connect(addr, "Client 1".to_string()).await;
            let n = unet.send(Packet::Reliable).await.unwrap();
            tokio::time::sleep(Duration::from_millis(8000)).await;
            let n = unet.send(Packet::Reliable).await.unwrap();
        });

        let (r1, r2) = tokio::join!(handle1, handle2);
    }
}
