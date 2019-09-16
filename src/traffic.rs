use ::std::fmt;
use ::std::net::{Ipv4Addr, SocketAddrV4};
use ::std::boxed::Box;

use ::pnet::datalink::{NetworkInterface, DataLinkReceiver};
use ::pnet::packet::Packet;
use ::pnet::packet::ethernet::{EtherType, EthernetPacket};
use ::pnet::packet::ipv4::Ipv4Packet;
use ::pnet::packet::ip::IpNextHeaderProtocol;
use ::pnet::packet::tcp::TcpPacket;
use ::pnet::packet::udp::UdpPacket;

use ::ipnetwork::IpNetwork;

pub struct Sniffer {
    network_interface: NetworkInterface,
    network_frames: Box<DataLinkReceiver>,
}

#[derive(PartialEq, Hash, Eq, Debug, Clone, PartialOrd, Ord)]
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
    pub ip_length: u128
}

#[derive(PartialEq, Hash, Eq, Debug, Clone, PartialOrd, Ord)]
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

fn find_direction (network_interface_ips: &Vec<IpNetwork>, ip_packet: &Ipv4Packet) -> Direction {
    match network_interface_ips.iter().any(|ip_network| ip_network.ip() == ip_packet.get_source()) {
        true => Direction::Upload,
        false => Direction::Download
    }
}

impl Direction {
    pub fn make_connection (&self, from: SocketAddrV4, to: SocketAddrV4, protocol: Protocol) -> Connection {
        match self {
            Direction::Upload => {
                Connection {
                    local_ip: *from.ip(),
                    remote_ip: *to.ip(),
                    local_port: from.port(),
                    remote_port: to.port(),
                    protocol
                }
            },
            Direction::Download => {
                Connection {
                    local_ip: *to.ip(),
                    remote_ip: *from.ip(),
                    local_port: to.port(),
                    remote_port: from.port(),
                    protocol
                }
            }
        }
    }
}

impl Sniffer {
    pub fn new (network_interface: NetworkInterface, network_frames: Box<DataLinkReceiver>) -> Self {
        Sniffer { network_interface, network_frames }
    }
    pub fn next(&mut self) -> Option<Segment> {
        // TODO: https://github.com/libpnet/libpnet/issues/343
        // make this non-blocking for faster exits
        let bytes = self.network_frames.next().unwrap_or_else(|e| {
            panic!("An error occurred while reading: {}", e);
        });
        let packet = EthernetPacket::new(bytes)?;
        match packet.get_ethertype() { // TODO: better way through the module?
            EtherType(2048) => {
                let ip_packet = Ipv4Packet::new(packet.payload())?;
                let (protocol, source_port, destination_port) = match ip_packet.get_next_level_protocol() {
                    IpNextHeaderProtocol(6) => { // tcp
                        let message = TcpPacket::new(ip_packet.payload())?;
                        (
                            Protocol::Tcp,
                            message.get_source(),
                            message.get_destination()
                        )
                    },
                    IpNextHeaderProtocol(17) => { // udp
                        let datagram = UdpPacket::new(ip_packet.payload())?;
                        (
                            Protocol::Udp,
                            datagram.get_source(),
                            datagram.get_destination()
                        )
                    },
                    _ => return None
                };
                let direction = find_direction(&self.network_interface.ips, &ip_packet);
                let from = SocketAddrV4::new(ip_packet.get_source(), source_port);
                let to = SocketAddrV4::new(ip_packet.get_destination(), destination_port);
                let connection = direction.make_connection(from, to, protocol);
                let ip_length = ip_packet.get_total_length() as u128;
                Some(Segment { connection, ip_length, direction })
            },
            _ => {
                None
            }
        }
    }
}
