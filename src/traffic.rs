use ::std::boxed::Box;
use ::std::fmt;
use ::std::net::Ipv4Addr;

use ::pnet::datalink::{DataLinkReceiver, NetworkInterface};
use ::pnet::packet::ethernet::{EtherType, EthernetPacket};
use ::pnet::packet::ip::IpNextHeaderProtocol;
use ::pnet::packet::ipv4::Ipv4Packet;
use ::pnet::packet::tcp::TcpPacket;
use ::pnet::packet::udp::UdpPacket;
use ::pnet::packet::Packet;

use ::ipnetwork::IpNetwork;

#[derive(PartialEq, Hash, Eq, Debug, Clone, PartialOrd, Ord)]
pub struct Socket {
    pub ip: Ipv4Addr,
    pub port: u16,
}

impl fmt::Display for Socket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.ip, self.port)
    }
}

pub struct Sniffer {
    network_interface: NetworkInterface,
    network_frames: Box<DataLinkReceiver>,
}

#[derive(PartialEq, Hash, Eq, Debug, Clone, PartialOrd, Ord)]
pub struct Connection {
    pub local_socket: Socket,
    pub remote_socket: Socket,
    pub protocol: Protocol,
}

impl fmt::Display for Connection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} => {} ({})",
            self.local_socket, self.remote_socket, self.protocol
        )
    }
}

pub struct Segment {
    pub connection: Connection,
    pub direction: Direction,
    pub ip_length: u128,
}

#[derive(PartialEq, Hash, Eq, Debug, Clone, PartialOrd, Ord)]
pub enum Protocol {
    Tcp,
    Udp,
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Protocol::Tcp => write!(f, "tcp"),
            Protocol::Udp => write!(f, "udp"),
        }
    }
}

#[derive(PartialEq, Hash, Eq, Debug, Clone, PartialOrd)]
pub enum Direction {
    Download,
    Upload,
}

impl Direction {
    pub fn make_connection(&self, from: Socket, to: Socket, protocol: Protocol) -> Connection {
        match self {
            Direction::Upload => Connection {
                local_socket: from,
                remote_socket: to,
                protocol,
            },
            Direction::Download => Connection {
                local_socket: to,
                remote_socket: from,
                protocol,
            },
        }
    }
}

fn find_direction(network_interface_ips: &Vec<IpNetwork>, ip_packet: &Ipv4Packet) -> Direction {
    match network_interface_ips
        .iter()
        .any(|ip_network| ip_network.ip() == ip_packet.get_source())
    {
        true => Direction::Upload,
        false => Direction::Download,
    }
}

impl Sniffer {
    pub fn new(network_interface: NetworkInterface, network_frames: Box<DataLinkReceiver>) -> Self {
        Sniffer {
            network_interface,
            network_frames,
        }
    }
    pub fn next(&mut self) -> Option<Segment> {
        // TODO: https://github.com/libpnet/libpnet/issues/343
        // make this non-blocking for faster exits
        let bytes = self.network_frames.next().unwrap_or_else(|e| {
            panic!("An error occurred while reading: {}", e);
        });
        let packet = EthernetPacket::new(bytes)?;
        match packet.get_ethertype() {
            EtherType(2048) => {
                let ip_packet = Ipv4Packet::new(packet.payload())?;
                let (protocol, source_port, destination_port) =
                    match ip_packet.get_next_level_protocol() {
                        IpNextHeaderProtocol(6) => {
                            let message = TcpPacket::new(ip_packet.payload())?;
                            (
                                Protocol::Tcp,
                                message.get_source(),
                                message.get_destination(),
                            )
                        }
                        IpNextHeaderProtocol(17) => {
                            let datagram = UdpPacket::new(ip_packet.payload())?;
                            (
                                Protocol::Udp,
                                datagram.get_source(),
                                datagram.get_destination(),
                            )
                        }
                        _ => return None,
                    };
                let direction = find_direction(&self.network_interface.ips, &ip_packet);
                let from = Socket {
                    ip: ip_packet.get_source(),
                    port: source_port,
                };
                let to = Socket {
                    ip: ip_packet.get_destination(),
                    port: destination_port,
                };
                let connection = direction.make_connection(from, to, protocol);
                let ip_length = ip_packet.get_total_length() as u128;
                Some(Segment {
                    connection,
                    ip_length,
                    direction,
                })
            }
            _ => None,
        }
    }
}
