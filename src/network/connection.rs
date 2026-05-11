use std::{
    collections::HashMap,
    fmt,
    hash::{Hash, Hasher},
    net::{IpAddr, SocketAddr},
};

#[derive(PartialEq, Hash, Eq, Clone, PartialOrd, Ord, Debug, Copy)]
pub enum Protocol {
    Tcp,
    Udp,
}

impl Protocol {
    #[allow(dead_code)]
    pub fn from_str(string: &str) -> Option<Self> {
        match string {
            "TCP" => Some(Protocol::Tcp),
            "UDP" => Some(Protocol::Udp),
            _ => None,
        }
    }
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Protocol::Tcp => write!(f, "tcp"),
            Protocol::Udp => write!(f, "udp"),
        }
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq, Hash, Copy)]
pub struct Socket {
    pub ip: IpAddr,
    pub port: u16,
}

impl fmt::Debug for Socket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Socket { ip, port } = self;
        match ip {
            IpAddr::V4(v4) => write!(f, "{v4}:{port}"),
            IpAddr::V6(v6) => write!(f, "[{v6}]:{port}"),
        }
    }
}

#[derive(PartialEq, Hash, Eq, Clone, PartialOrd, Ord, Copy)]
pub struct LocalSocket {
    pub ip: IpAddr,
    pub port: u16,
    pub protocol: Protocol,
}

impl fmt::Debug for LocalSocket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let LocalSocket { ip, port, protocol } = self;
        match ip {
            IpAddr::V4(v4) => write!(f, "{protocol}://{v4}:{port}"),
            IpAddr::V6(v6) => write!(f, "{protocol}://[{v6}]:{port}"),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Connection {
    pub remote_socket: Socket,
    pub local_socket: LocalSocket,
}

impl Connection {
    pub fn normalized_key(&self) -> (Protocol, Socket, Socket) {
        let local_socket = Socket {
            ip: self.local_socket.ip,
            port: self.local_socket.port,
        };

        if local_socket <= self.remote_socket {
            (self.local_socket.protocol, local_socket, self.remote_socket)
        } else {
            (self.local_socket.protocol, self.remote_socket, local_socket)
        }
    }
}

impl PartialEq for Connection {
    fn eq(&self, other: &Self) -> bool {
        self.normalized_key() == other.normalized_key()
    }
}

impl Eq for Connection {}

impl Hash for Connection {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.normalized_key().hash(state);
    }
}

impl PartialOrd for Connection {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Connection {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.normalized_key().cmp(&other.normalized_key())
    }
}

impl fmt::Debug for Connection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Connection {
            remote_socket,
            local_socket,
        } = self;
        write!(f, "{local_socket:?} => {remote_socket:?}")
    }
}

pub fn display_ip_or_host(ip: IpAddr, ip_to_host: &HashMap<IpAddr, String>) -> String {
    match ip_to_host.get(&ip) {
        Some(host) => host.clone(),
        None => ip.to_string(),
    }
}

pub fn display_connection_string(
    connection: &Connection,
    ip_to_host: &HashMap<IpAddr, String>,
    interface_name: &str,
) -> String {
    format!(
        "<{interface_name}>:{} => {}:{} ({})",
        connection.local_socket.port,
        display_ip_or_host(connection.remote_socket.ip, ip_to_host),
        connection.remote_socket.port,
        connection.local_socket.protocol,
    )
}

impl Connection {
    pub fn new(
        remote_socket: SocketAddr,
        local_ip: IpAddr,
        local_port: u16,
        protocol: Protocol,
    ) -> Self {
        Connection {
            remote_socket: Socket {
                ip: remote_socket.ip(),
                port: remote_socket.port(),
            },
            local_socket: LocalSocket {
                ip: local_ip,
                port: local_port,
                protocol,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::HashMap, net::Ipv4Addr};

    #[test]
    fn connection_matches_packets_in_either_direction() {
        let outbound = Connection::new(
            SocketAddr::from((Ipv4Addr::new(203, 0, 113, 1), 5201)),
            IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1)),
            49152,
            Protocol::Tcp,
        );
        let inbound = Connection::new(
            SocketAddr::from((Ipv4Addr::new(192, 0, 2, 1), 49152)),
            IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1)),
            5201,
            Protocol::Tcp,
        );

        let mut connections = HashMap::new();
        connections.insert(outbound, "iperf3");

        assert_eq!(connections.get(&inbound), Some(&"iperf3"));
    }

    #[test]
    fn connection_key_keeps_protocols_separate() {
        let tcp = Connection::new(
            SocketAddr::from((Ipv4Addr::new(203, 0, 113, 1), 5201)),
            IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1)),
            49152,
            Protocol::Tcp,
        );
        let udp = Connection::new(
            SocketAddr::from((Ipv4Addr::new(192, 0, 2, 1), 49152)),
            IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1)),
            5201,
            Protocol::Udp,
        );

        assert_ne!(tcp, udp);
        assert_ne!(tcp.normalized_key(), udp.normalized_key());
    }
}
