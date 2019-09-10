use ::std::fmt;
use ::std::net::Ipv4Addr;
use ::std::boxed::Box;

use ::pnet::datalink::{NetworkInterface, DataLinkReceiver};
use ::pnet::packet::Packet;
use ::pnet::packet::ethernet::{EtherType, EthernetPacket};
use ::pnet::packet::ipv4::Ipv4Packet;
use ::pnet::packet::ip::IpNextHeaderProtocol;
use ::pnet::packet::tcp::TcpPacket;
use ::pnet::packet::udp::UdpPacket;
use ::num_bigint::{BigUint, ToBigUint};

pub struct Sniffer {
    network_interface: NetworkInterface,
    network_frames: Box<DataLinkReceiver>,
}

#[derive(PartialEq, Hash, Eq, Debug, Clone, PartialOrd)]
pub struct Connection {
    pub local_ip: Ipv4Addr,
    pub remote_ip: Ipv4Addr,
    pub local_port: u16,
    pub remote_port: u16,
    pub protocol: Protocol,
}

impl fmt::Display for Connection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{} => {}:{} ({})",
            self.local_ip,
            self.local_port,
            self.remote_ip,
            self.remote_port,
            self.protocol
        )
    }
}

pub struct Segment {
    pub connection: Connection,
    pub direction: Direction,
    pub ip_length: BigUint
}

#[derive(PartialEq, Hash, Eq, Debug, Clone, PartialOrd)]
pub enum Protocol {
    Tcp,
    Udp
}

#[derive(PartialEq, Hash, Eq, Debug, Clone, PartialOrd)]
pub enum Direction {
    Download,
    Upload
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       match *self {
           Protocol::Tcp => write!(f, "tcp"),
           Protocol::Udp => write!(f, "udp")
       }
    }
}

macro_rules! find_direction {
    ($a:expr, $b:expr) => {
        match $a.iter().any(|ip_network| ip_network.ip() == $b.get_source()) {
            true => Direction::Upload,
            false => Direction::Download
        };
    }
}

macro_rules! build_connection {
    ($a:expr, $b:expr, $c:expr, $d:expr) => {
        match $a {
            Direction::Upload => {
                let local_ip = $b.get_source();
                let remote_ip = $b.get_destination();
                let local_port = $c.get_source();
                let remote_port = $c.get_destination();
                Connection { local_ip, remote_ip, local_port, remote_port, protocol: $d }
            },
            Direction::Download => {
                let local_ip = $b.get_destination();
                let remote_ip = $b.get_source();
                let local_port = $c.get_destination();
                let remote_port = $c.get_source();
                Connection { local_ip, remote_ip, local_port, remote_port, protocol: $d }
            }
        };
    }
}

impl Sniffer {
    pub fn new (network_interface: NetworkInterface, network_frames: Box<DataLinkReceiver>) -> Self {
        Sniffer { network_interface, network_frames }
    }
    pub fn next(&mut self) -> Option<Segment> {
        // TODO: https://github.com/libpnet/libpnet/issues/343
        // make this non-blocking for faster exits
        match self.network_frames.next() {
            Ok(bytes) => {
                match EthernetPacket::new(bytes) {
                    Some(packet) => {
                        match packet.get_ethertype() { // TODO: better way through the module?
                            EtherType(2048) => {
                                let ip_packet = Ipv4Packet::new(packet.payload()).unwrap();
                                match ip_packet.get_next_level_protocol() {
                                    IpNextHeaderProtocol(6) => { // tcp
                                        let message = TcpPacket::new(ip_packet.payload()).unwrap();
                                        let protocol = Protocol::Tcp;
                                        let direction = find_direction!(self.network_interface.ips, ip_packet);
                                        let connection = build_connection!(direction, ip_packet, message, protocol);
                                        let ip_length = ip_packet.get_total_length().to_biguint().unwrap();
                                        Some(Segment { connection, ip_length, direction })
                                    },
                                    IpNextHeaderProtocol(17) => { // udp
                                        let datagram = UdpPacket::new(ip_packet.payload()).unwrap();
                                        let protocol = Protocol::Udp;
                                        let direction = find_direction!(self.network_interface.ips, ip_packet);
                                        let connection = build_connection!(direction, ip_packet, datagram, protocol);
                                        let ip_length = ip_packet.get_total_length().to_biguint().unwrap();
                                        Some(Segment { connection, ip_length, direction })
                                    },
                                    _ => {
                                        None
                                    }
                                }
                            },
                            _ => {
                                None
                            }
                        }
                    },
                    None => None
                }
            },
            Err(e) => {
                // If an error occurs, we can handle it here
                panic!("An error occurred while reading: {}", e);
            }
        }
    }
}
