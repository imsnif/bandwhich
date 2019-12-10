use ::std::boxed::Box;

use ::pnet::datalink::{DataLinkReceiver, NetworkInterface};
use ::pnet::packet::ethernet::{EtherType, EthernetPacket};
use ::pnet::packet::ip::IpNextHeaderProtocol;
use ::pnet::packet::ipv4::Ipv4Packet;
use ::pnet::packet::tcp::TcpPacket;
use ::pnet::packet::udp::UdpPacket;
use ::pnet::packet::Packet;

use ::ipnetwork::IpNetwork;
use ::std::net::{IpAddr, SocketAddr};

use crate::network::{Connection, Protocol};

pub struct Segment {
    pub interface: String,
    pub connection: Connection,
    pub direction: Direction,
    pub data_length: u128,
}

#[derive(PartialEq, Hash, Eq, Debug, Clone, PartialOrd)]
pub enum Direction {
    Download,
    Upload,
}

impl Direction {
    pub fn new(network_interface_ips: &[IpNetwork], ip_packet: &Ipv4Packet) -> Self {
        if network_interface_ips
            .iter()
            .any(|ip_network| ip_network.ip() == ip_packet.get_source())
        {
            Direction::Upload
        } else {
            Direction::Download
        }
    }
}

pub struct Sniffer {
    network_interface: NetworkInterface,
    network_frames: Box<dyn DataLinkReceiver>,
}

impl Sniffer {
    pub fn new(
        network_interface: NetworkInterface,
        network_frames: Box<dyn DataLinkReceiver>,
    ) -> Self {
        Sniffer {
            network_interface,
            network_frames,
        }
    }
    pub fn next(&mut self) -> Option<Segment> {
        let bytes = self.network_frames.next().ok()?;
        let packet = EthernetPacket::new(bytes)?;
        match packet.get_ethertype() {
            EtherType(2048) => {
                let ip_packet = Ipv4Packet::new(packet.payload())?;
                let (protocol, source_port, destination_port, data_length) =
                    match ip_packet.get_next_level_protocol() {
                        IpNextHeaderProtocol(6) => {
                            let message = TcpPacket::new(ip_packet.payload())?;
                            (
                                Protocol::Tcp,
                                message.get_source(),
                                message.get_destination(),
                                ip_packet.payload().len() as u128,
                            )
                        }
                        IpNextHeaderProtocol(17) => {
                            let datagram = UdpPacket::new(ip_packet.payload())?;
                            (
                                Protocol::Udp,
                                datagram.get_source(),
                                datagram.get_destination(),
                                ip_packet.payload().len() as u128,
                            )
                        }
                        _ => return None,
                    };
                let interface = self.network_interface.name.clone();
                let direction = Direction::new(&self.network_interface.ips, &ip_packet);
                let from = SocketAddr::new(IpAddr::V4(ip_packet.get_source()), source_port);
                let to = SocketAddr::new(IpAddr::V4(ip_packet.get_destination()), destination_port);

                let connection = match direction {
                    Direction::Download => Connection::new(from, destination_port, protocol)?,
                    Direction::Upload => Connection::new(to, source_port, protocol)?,
                };
                Some(Segment {
                    interface,
                    connection,
                    data_length,
                    direction,
                })
            }
            _ => None,
        }
    }
}
