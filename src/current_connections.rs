use ::std::collections::HashMap;
use ::std::fmt;
use ::std::net::Ipv4Addr;
use ::std::net::IpAddr;

// #[cfg(not(feature = "test"))]
// use ::procfs::Process;

use ::num_bigint::BigUint;
use ::num_traits::{Zero, One};

use ::netstat::*;
use ::pnet::datalink::NetworkInterface;

use crate::traffic::{Segment, Connection, Protocol, Direction};
use crate::display::IsProcess;

// #[cfg(feature = "test")]
// pub struct Stat {
//     pub comm: String
// }
// 
// #[cfg(feature = "test")]
// pub struct Process {
//     pub stat: Stat
// }
// #[cfg(feature = "test")]
// impl IsProcess for Process {
// //    pub fn new (&self, id: i32) -> Result<Self, Error> {
// //        Ok(Process {stat: Stat { comm: String::from("foo")}})
// //    }
// }


pub struct CurrentConnections <T>
where T: std::fmt::Debug
{
    // pub connections: HashMap<Connection, Vec<Process>>
    pub connections: HashMap<Connection, Vec<T>>
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
                    let local_addr = match tcp_si.local_addr {
                        IpAddr::V4(local_addr) => Some(local_addr),
                        IpAddr::V6(_) => None
                    };
                    let remote_addr = match tcp_si.remote_addr {
                        IpAddr::V4(remote_addr) => Some(remote_addr),
                        IpAddr::V6(_) => None
                    };
                
                    match (local_addr, remote_addr) {
                        (Some(local_addr), Some(remote_addr)) => {
                            connections.insert(
                                Connection {
                                    local_ip: local_addr,
                                    local_port: tcp_si.local_port,
                                    remote_ip: remote_addr,
                                    remote_port: tcp_si.remote_port,
                                    protocol: Protocol::Tcp
                                    // tcp_si.state
                                },
                                // si.associated_pids.iter().map(|pid| Process::new(*pid as i32).unwrap()).collect()
                                si.associated_pids.iter().map(|pid| create_process(*pid as i32).unwrap()).collect()
                            );
                        },
                        (_, _) => () 
                    }
                },
                ProtocolSocketInfo::Udp(udp_si) => {
                    let local_addr = match udp_si.local_addr {
                        IpAddr::V4(local_addr) => Some(local_addr),
                        IpAddr::V6(_) => None
                    };
                    let remote_addr = match udp_si.remote_addr {
                        IpAddr::V4(remote_addr) => Some(remote_addr),
                        IpAddr::V6(_) => None
                    };
                
                    match (local_addr, remote_addr) {
                        (Some(local_addr), Some(remote_addr)) => {
                            connections.insert(
                                Connection {
                                    local_ip: local_addr,
                                    local_port: udp_si.local_port,
                                    remote_ip: remote_addr,
                                    remote_port: udp_si.remote_port,
                                    protocol: Protocol::Udp
                                    // tcp_si.state
                                },
                                // si.associated_pids.iter().map(|pid| Process::new(*pid as i32).unwrap()).collect()
                                si.associated_pids.iter().map(|pid| create_process(*pid as i32).unwrap()).collect()
                            );
                        },
                        (_, _) => () 
                    }
                }
            }
        };
        CurrentConnections {connections}
    }
}

impl <T> fmt::Debug for CurrentConnections <T>
where T: std::fmt::Debug
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", self.connections)
    }
}

// #[cfg(feature = "test")]
// impl fmt::Debug for Process {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "{:#?}", self.stat.comm)
//     }
// }
