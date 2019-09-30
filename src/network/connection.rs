use ::std::fmt;
use ::std::net::Ipv4Addr;

use ::std::mem::swap;
use ::std::net::SocketAddr;

use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

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

#[derive(Clone)]
pub struct Socket {
    pub ip: Ipv4Addr,
    pub port: u16,
    host_addr: Option<String>,
}

impl Socket {
    pub fn clone_host_or_ip(&self) -> String {
        match &self.host_addr {
            Some(host_addr) => host_addr.clone(),
            None => self.ip.to_string(),
        }
    }
}

impl Ord for Socket {
    fn cmp(&self, other: &Self) -> Ordering {
        let ip_eq = self.ip.cmp(&other.ip); // TODO: also port
        match ip_eq {
            Ordering::Equal => self.port.cmp(&other.port),
            _ => ip_eq,
        }
    }
}

impl PartialOrd for Socket {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Socket {
    fn eq(&self, other: &Self) -> bool {
        self.ip == other.ip && self.port == other.port
    }
}

impl Eq for Socket {}

impl Hash for Socket {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ip.hash(state);
        self.port.hash(state);
    }
}

impl fmt::Display for Socket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.host_addr {
            Some(host_addr) => write!(f, "{}:{}", host_addr, self.port),
            None => write!(f, "{}:{}", self.ip, self.port),
        }
    }
}

#[derive(PartialEq, Hash, Eq, Clone, PartialOrd, Ord)]
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
                local_socket: Socket {
                    ip: *local_socket.ip(),
                    port: local_socket.port(),
                    host_addr: None,
                },
                remote_socket: Socket {
                    ip: *remote_socket.ip(),
                    port: remote_socket.port(),
                    host_addr: None,
                },
                protocol,
            }),
            (_, _) => None,
        }
    }
    pub fn swap_direction(&mut self) {
        swap(&mut self.local_socket, &mut self.remote_socket);
    }
    pub fn set_local_host_addr(&mut self, addr: &str) {
        self.local_socket.host_addr = Some(String::from(addr));
    }
    pub fn set_remote_host_addr(&mut self, addr: &str) {
        self.remote_socket.host_addr = Some(String::from(addr));
    }
}
