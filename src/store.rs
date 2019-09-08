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
pub struct TotalBandwidth {
    pub total_bytes_downloaded: BigUint,
    pub total_bytes_uploaded: BigUint,
}

impl TotalBandwidth {
    pub fn increment_bytes_downloaded (&mut self, ip_length: &BigUint) {
        self.total_bytes_downloaded += ip_length;
    }
    pub fn increment_bytes_uploaded (&mut self, ip_length: &BigUint) {
        self.total_bytes_uploaded += ip_length;
    }
}

pub struct NetworkUtilization {
    pub connections: HashMap<Connection, TotalBandwidth>
}

impl NetworkUtilization {
    pub fn new() -> Self {
        let connections = HashMap::new();
        NetworkUtilization { connections }
    }
    pub fn reset (&mut self) {
        self.connections.clear();
    }
    pub fn update(&mut self, seg: &Segment) {
        let total_bandwidth = self.connections.entry(seg.connection.clone()).or_insert(TotalBandwidth {
            total_bytes_downloaded: Zero::zero(),
            total_bytes_uploaded: Zero::zero()
        });
        match seg.direction {
            Direction::Download => {
                total_bandwidth.increment_bytes_downloaded(&seg.ip_length)
            },
            Direction::Upload => {
                total_bandwidth.increment_bytes_uploaded(&seg.ip_length)
            }

        }
    }
}
