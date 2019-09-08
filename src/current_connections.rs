use ::std::collections::HashMap;
use ::std::fmt;
use ::std::net::Ipv4Addr;
use ::std::net::IpAddr;

use ::num_bigint::BigUint;
use ::num_traits::{Zero, One};

use ::netstat::*;
use ::pnet::datalink::NetworkInterface;

use crate::traffic::{Segment, Connection, Protocol, Direction};
use crate::display::IsProcess;

pub struct CurrentConnections <T>
where T: std::fmt::Debug
{
    pub connections: HashMap<Connection, Vec<T>>
}

macro_rules! get_ipv4_address {
    ($a:expr) => {
        match $a {
            IpAddr::V4(addr) => Some(addr),
            IpAddr::V6(_) => None
        }
    }
}

macro_rules! build_ipv4_connection {
    ($a:expr, $b:expr, $c:expr, $d:expr) => {
        match ($a, $b) {
            (Some(local_ip), Some(remote_ip)) => {
                Some(Connection {
                    local_ip,
                    remote_ip,
                    local_port: $c.local_port,
                    remote_port: $c.remote_port,
                    protocol: $d
                })
            },
            (_, _) => None
        }
    }
}

impl <T> CurrentConnections <T>
where T: std::fmt::Debug + IsProcess
{
    pub fn new<Z>(create_process: &Fn(i32) -> Result<T, Box<std::error::Error>>, get_sockets_info: &Fn(AddressFamilyFlags, ProtocolFlags) -> Result<Vec<SocketInfo>, Z>) -> Self
    where Z: std::fmt::Debug
    {
        let af_flags = AddressFamilyFlags::IPV4;
        let proto_flags = ProtocolFlags::TCP | ProtocolFlags::UDP;
        let sockets_info = get_sockets_info(af_flags, proto_flags).unwrap();
        let mut connections = HashMap::new();
        for si in sockets_info {
            match si.protocol_socket_info {
                ProtocolSocketInfo::Tcp(tcp_si) => {
                    let local_addr = get_ipv4_address!(tcp_si.local_addr);
                    let remote_addr = get_ipv4_address!(tcp_si.remote_addr);
                    let connection = build_ipv4_connection!(local_addr, remote_addr, tcp_si, Protocol::Tcp);
                    match connection {
                        Some(conn) => {
                            connections.insert(conn, si.associated_pids.iter().map(|pid| create_process(*pid as i32).unwrap()).collect());
                        },
                        None => ()
                    }
                },
                ProtocolSocketInfo::Udp(udp_si) => {
                    let local_addr = get_ipv4_address!(udp_si.local_addr);
                    let remote_addr = get_ipv4_address!(udp_si.remote_addr);
                    let connection = build_ipv4_connection!(local_addr, remote_addr, udp_si, Protocol::Udp);
                    match connection {
                        Some(conn) => {
                            connections.insert(conn, si.associated_pids.iter().map(|pid| create_process(*pid as i32).unwrap()).collect());
                        },
                        None => ()
                    }
                }
            }
        };
        CurrentConnections {connections}
    }
}
