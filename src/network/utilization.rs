use std::collections::HashMap;

use crate::network::{Connection, Direction, Segment};

#[derive(Clone)]
pub struct ConnectionInfo {
    pub interface_name: String,
    pub total_bytes_downloaded: u128,
    pub total_bytes_uploaded: u128,
}

#[derive(Clone)]
pub struct Utilization {
    pub connections: HashMap<Connection, ConnectionInfo>,
}

impl Utilization {
    pub fn new() -> Self {
        let connections = HashMap::new();
        Utilization { connections }
    }
    pub fn clone_and_reset(&mut self) -> Self {
        let clone = self.clone();
        self.connections.clear();
        clone
    }
    pub fn ingest(&mut self, seg: Segment) {
        let total_bandwidth = self
            .connections
            .entry(seg.connection)
            .or_insert(ConnectionInfo {
                interface_name: seg.interface_name,
                total_bytes_downloaded: 0,
                total_bytes_uploaded: 0,
            });
        match seg.direction {
            Direction::Download => {
                total_bandwidth.total_bytes_downloaded += seg.data_length;
            }
            Direction::Upload => {
                total_bandwidth.total_bytes_uploaded += seg.data_length;
            }
        }
    }
}
