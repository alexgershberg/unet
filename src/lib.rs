use std::collections::HashSet;
use std::io;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

struct UnetServer {
    socket: UdpSocket,
}

impl UnetServer {
    fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let socket = UdpSocket::bind(addr)?;
        Ok(Self { socket })
    }

    fn accept(&self) -> UnetConnection {
        let mut connections: HashSet<SocketAddr> = HashSet::new();

        loop {
            let mut buf: [u8; 64] = [0; 64];
            let (n, addr) = match self.socket.recv_from(&mut buf) {
                Ok((n, addr)) => (n, addr),

                Err(_) => todo!(),
            };

            if !connections.contains(&addr) {
                connections.insert(addr);
                return UnetConnection::connect(addr);
            } else {
                continue;
            }
        }
    }
}

struct UnetConnection {
    socket: UdpSocket,
    target: SocketAddr,
}

impl UnetConnection {
    fn connect<T: ToSocketAddrs>(target: T) -> Self {
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        let mut iter = target.to_socket_addrs().unwrap();
        Self {
            socket,
            target: iter.next().unwrap(),
        }
    }

    fn send(&self, frame: Frame) -> io::Result<usize> {
        let buf: [u8; 64] = [0; 64];
        self.socket.send_to(&buf, self.target)
    }

    fn receive(&self) -> io::Result<Frame> {
        loop {
            let mut buf: [u8; 64] = [0; 64];
            let (n, addr) = self.socket.recv_from(&mut buf)?;
            if addr != self.target {
                eprintln!("addr != self.target: {addr} | {}", self.target);
                continue;
            }
            Ok(Frame(0));
        }
    }
}

struct Frame(i32);

#[cfg(test)]
mod tests {
    use crate::{Frame, UnetServer};
    use std::net::TcpListener;

    fn test() {
        let l = TcpListener::bind("127.0.0.1:10001").unwrap();
        l.accept();
    }

    #[test]
    fn test1() {
        let server = UnetServer::bind("127.0.0.1:10001").unwrap();
        loop {
            let connection = server.accept();
        }
    }
}
