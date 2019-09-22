use crate::network::{Connection, Direction, Segment};

use ::std::collections::HashMap;
use ::std::time::SystemTime;

#[derive(Clone)]
pub struct TotalBandwidth {
    pub total_bytes_downloaded: u128,
    pub total_bytes_uploaded: u128,
}

impl TotalBandwidth {
    pub fn increment_bytes_downloaded(&mut self, data_length: u128, reset_time: &SystemTime) {
        if let Ok(elapsed) = reset_time.elapsed() {
            if elapsed.as_millis() < 1000 {
                self.total_bytes_downloaded += data_length;
            }
        }
    }
    pub fn increment_bytes_uploaded(&mut self, data_length: u128, reset_time: &SystemTime) {
        if let Ok(elapsed) = reset_time.elapsed() {
            if elapsed.as_millis() < 1000 {
                self.total_bytes_uploaded += data_length;
            }
        }
    }
}

#[derive(Clone)]
pub struct Utilization {
    pub connections: HashMap<Connection, TotalBandwidth>,
    reset_time: SystemTime,
}

impl Utilization {
    pub fn new() -> Self {
        let connections = HashMap::new();
        Utilization {
            connections,
            reset_time: SystemTime::now(),
        }
    }
    pub fn clone_and_reset(&mut self) -> Self {
        let clone = self.clone();
        self.reset_time = SystemTime::now();
        self.connections.clear();
        clone
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
            Direction::Download => {
                total_bandwidth.increment_bytes_downloaded(seg.data_length, &self.reset_time)
            }
            Direction::Upload => {
                total_bandwidth.increment_bytes_uploaded(seg.data_length, &self.reset_time)
            }
        }
    }
}
