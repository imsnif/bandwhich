use crate::network::{Connection, Direction, Segment};

use ::std::collections::HashMap;

pub struct TotalBandwidth {
    pub total_bytes_downloaded: u128,
    pub total_bytes_uploaded: u128,
}

impl TotalBandwidth {
    pub fn increment_bytes_downloaded(&mut self, ip_length: u128) {
        self.total_bytes_downloaded += ip_length;
    }
    pub fn increment_bytes_uploaded(&mut self, ip_length: u128) {
        self.total_bytes_uploaded += ip_length;
    }
}

pub struct Utilization {
    pub connections: HashMap<Connection, TotalBandwidth>,
}

impl Utilization {
    pub fn new() -> Self {
        let connections = HashMap::new();
        Utilization { connections }
    }
    pub fn reset(&mut self) {
        self.connections.clear();
    }
    pub fn update(&mut self, seg: &Segment) {
        let total_bandwidth =
            self.connections
                .entry(seg.connection.clone())
                .or_insert(TotalBandwidth {
                    total_bytes_downloaded: 0,
                    total_bytes_uploaded: 0,
                });
        match seg.direction {
            Direction::Download => total_bandwidth.increment_bytes_downloaded(seg.ip_length),
            Direction::Upload => total_bandwidth.increment_bytes_uploaded(seg.ip_length),
        }
    }
}
