use ::std::fmt;
use ::std::net::Ipv4Addr;

use ::std::mem::swap;
use ::std::net::SocketAddr;

#[derive(PartialEq, Hash, Eq, Debug, Clone, PartialOrd, Ord)]
pub enum Protocol {
    Tcp,
    Udp,
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Protocol::Tcp => write!(f, "tcp"),
            Protocol::Udp => write!(f, "udp"),
        }
    }
}

#[derive(PartialEq, Hash, Eq, Debug, Clone, PartialOrd, Ord)]
pub struct Socket {
    pub ip: Ipv4Addr,
    pub port: u16,
}

impl fmt::Display for Socket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.ip, self.port)
    }
}

#[derive(PartialEq, Hash, Eq, Debug, Clone, PartialOrd, Ord)]
pub struct Connection {
    pub local_socket: Socket,
    pub remote_socket: Socket,
    pub protocol: Protocol,
}

impl fmt::Display for Connection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} => {} ({})",
            self.local_socket, self.remote_socket, self.protocol
        )
    }
}

impl Connection {
    pub fn new(
        local_socket: SocketAddr,
        remote_socket: SocketAddr,
        protocol: Protocol,
    ) -> Option<Self> {
        match (local_socket, remote_socket) {
            (SocketAddr::V4(local_socket), SocketAddr::V4(remote_socket)) => Some(Connection {
                // we use our own Socket here because SocketAddr is not sorable
                local_socket: Socket {
                    ip: *local_socket.ip(),
                    port: local_socket.port(),
                },
                remote_socket: Socket {
                    ip: *remote_socket.ip(),
                    port: remote_socket.port(),
                },
                protocol,
            }),
            (_, _) => None,
        }
    }
    pub fn swap_direction(&mut self) {
        swap(&mut self.local_socket, &mut self.remote_socket);
    }
}
