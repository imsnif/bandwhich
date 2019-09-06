use ::std::collections::HashMap;
use ::std::fmt;
use ::std::net::Ipv4Addr;
use ::std::net::IpAddr;

use ::procfs::Process;

use ::num_bigint::BigUint;
use ::num_traits::{Zero, One};

use ::netstat::*;
use ::pnet::datalink::NetworkInterface;

use crate::traffic::{Segment, Connection, Protocol, Direction};

#[derive(Debug, Clone)]
pub struct ConnectionData {
    // pub processes: Vec<Process>, // TODO: actual type
    pub total_bytes_downloaded: BigUint,
    pub total_bytes_uploaded: BigUint,
}

// impl fmt::Display for ConnectionData {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         let processes: String = self.processes.clone()
//             .into_iter()
//             .map(|process| process.stat.comm.to_string())
//             .collect();
//         write!(f, "({}/{})", processes, self.total_bytes)
//     }
// }

impl ConnectionData {
    pub fn increment_bytes_downloaded (&mut self, ip_length: &BigUint) {
        self.total_bytes_downloaded += ip_length;
    }
    pub fn increment_bytes_uploaded (&mut self, ip_length: &BigUint) {
        self.total_bytes_uploaded += ip_length;
    }
}

pub struct NetworkUtilization {
    pub connections: HashMap<Connection, ConnectionData>
}

impl NetworkUtilization {
    pub fn new() -> Self {
        let connections = HashMap::new();
        NetworkUtilization { connections }
    }
    // pub fn update(&mut self, seg: &Segment, proc: &Proc) {
    pub fn reset (&mut self) {
        self.connections.clear();
    }
    pub fn update(&mut self, seg: &Segment) {
        let connection_data = self.connections.entry(seg.connection.clone()).or_insert(ConnectionData {
            total_bytes_downloaded: Zero::zero(),
            total_bytes_uploaded: Zero::zero()
        });
        match seg.direction {
            Direction::Download => {
                connection_data.increment_bytes_downloaded(&seg.ip_length)
            },
            Direction::Upload => {
                connection_data.increment_bytes_uploaded(&seg.ip_length)
            }

        }
    }
}

impl fmt::Debug for NetworkUtilization {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", self.connections)
    }
}
