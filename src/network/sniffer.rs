use ::std::boxed::Box;

use ::pnet_bandwhich_fork::datalink::{DataLinkReceiver, NetworkInterface};
use ::pnet_bandwhich_fork::packet::ethernet::{EtherType, EthernetPacket};
use ::pnet_bandwhich_fork::packet::ip::IpNextHeaderProtocol;
use ::pnet_bandwhich_fork::packet::ipv4::Ipv4Packet;
use ::pnet_bandwhich_fork::packet::tcp::TcpPacket;
use ::pnet_bandwhich_fork::packet::udp::UdpPacket;
use ::pnet_bandwhich_fork::packet::Packet;

use ::ipnetwork::IpNetwork;
use ::std::net::{IpAddr, SocketAddr};

use crate::network::{Connection, Protocol};

pub struct Segment {
    pub interface_name: String,
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
        let ip_packet = Ipv4Packet::new(&bytes)?;
        let version = ip_packet.get_version();

        match version {
            4 => Self::handle_v4(ip_packet, &self.network_interface),
            6 => None, // FIXME v6 support!
            _ => {
                let pkg = EthernetPacket::new(bytes)?;
                match pkg.get_ethertype() {
                    EtherType(2048) => Self::handle_v4(Ipv4Packet::new(pkg.payload())?, &self.network_interface),
                    _ => None,
                }
            }
        }
    }
    fn handle_v4(ip_packet: Ipv4Packet, network_interface: &NetworkInterface) -> Option<Segment> {
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

        let interface_name = network_interface.name.clone();
        let direction = Direction::new(&network_interface.ips, &ip_packet);
        let from = SocketAddr::new(IpAddr::V4(ip_packet.get_source()), source_port);
        let to = SocketAddr::new(IpAddr::V4(ip_packet.get_destination()), destination_port);

        let connection = match direction {
            Direction::Download => {
                Connection::new(from, to.ip(), destination_port, protocol)?
            },
            Direction::Upload => Connection::new(to, from.ip(), source_port, protocol)?,
        };
        Some(Segment {
            interface_name,
            connection,
            data_length,
            direction,
        })
    }
}
