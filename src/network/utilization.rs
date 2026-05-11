use std::collections::HashMap;

use crate::network::{Connection, Direction, Segment};

#[derive(Clone)]
pub struct ConnectionInfo {
    pub interface_name: String,
    pub total_bytes_downloaded: u128,
    pub total_bytes_uploaded: u128,
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    use crate::network::{Connection, Direction, Protocol, Segment};

    use super::Utilization;

    fn connection(source_port: u16, destination_port: u16) -> Connection {
        Connection {
            source: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), source_port),
            destination: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1)), destination_port),
            protocol: Protocol::Tcp,
        }
    }

    fn segment(connection: Connection, direction: Direction, data_length: u128) -> Segment {
        Segment {
            interface_name: "eth0".to_string(),
            connection,
            direction,
            data_length,
        }
    }

    #[test]
    fn aggregates_upload_and_download_on_the_same_connection() {
        let mut utilization = Utilization::new();

        utilization.ingest(segment(connection(12345, 5201), Direction::Upload, 100));
        utilization.ingest(segment(connection(5201, 12345), Direction::Download, 250));

        assert_eq!(utilization.connections.len(), 1);
        let connection_info = utilization.connections.values().next().unwrap();
        assert_eq!(connection_info.total_bytes_uploaded, 100);
        assert_eq!(connection_info.total_bytes_downloaded, 250);
    }

    #[test]
    fn keeps_direction_counters_separate_when_connection_is_normalized() {
        let mut utilization = Utilization::new();

        utilization.ingest(segment(connection(12345, 5201), Direction::Upload, 100));
        utilization.ingest(segment(connection(5201, 12345), Direction::Download, 250));
        utilization.ingest(segment(connection(12345, 5201), Direction::Upload, 50));

        let connection_info = utilization.connections.values().next().unwrap();
        assert_eq!(connection_info.total_bytes_uploaded, 150);
        assert_eq!(connection_info.total_bytes_downloaded, 250);
    }
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
        let direction = seg.direction;
        let total_bandwidth = self
            .connections
            .entry(seg.connection.normalized(direction))
            .or_insert(ConnectionInfo {
                interface_name: seg.interface_name,
                total_bytes_downloaded: 0,
                total_bytes_uploaded: 0,
            });
        match direction {
            Direction::Download => {
                total_bandwidth.total_bytes_downloaded += seg.data_length;
            }
            Direction::Upload => {
                total_bandwidth.total_bytes_uploaded += seg.data_length;
            }
        }
    }
}
