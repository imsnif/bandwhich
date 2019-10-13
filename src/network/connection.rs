use ::std::fmt;
use ::std::net::Ipv4Addr;

use ::std::net::SocketAddr;

#[derive(PartialEq, Hash, Eq, Clone, PartialOrd, Ord)]
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

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq, Hash)]
pub struct Socket {
    pub ip: Ipv4Addr,
    pub port: u16,
}

#[derive(PartialEq, Hash, Eq, Clone, PartialOrd, Ord)]
pub struct Connection {
    pub remote_socket: Socket,
    pub protocol: Protocol,
    pub local_port: u16,
}

impl Connection {
    pub fn new(remote_socket: SocketAddr, local_port: u16, protocol: Protocol) -> Option<Self> {
        match remote_socket {
            SocketAddr::V4(remote_socket) => Some(Connection {
                remote_socket: Socket {
                    ip: *remote_socket.ip(),
                    port: remote_socket.port(),
                },
                protocol,
                local_port,
            }),
            _ => None,
        }
    }
}
