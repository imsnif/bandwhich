use ::std::boxed::Box;

use ::pnet::datalink::{DataLinkReceiver, NetworkInterface};
use ::pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use ::pnet::packet::ip::{IpNextHeaderProtocol, IpNextHeaderProtocols};
use ::pnet::packet::ipv4::Ipv4Packet;
use ::pnet::packet::ipv6::Ipv6Packet;
use ::pnet::packet::tcp::TcpPacket;
use ::pnet::packet::udp::UdpPacket;
use ::pnet::packet::Packet;

use ::ipnetwork::IpNetwork;
use ::std::io::{self, Result};
use ::std::net::{IpAddr, SocketAddr};
use ::std::thread::park_timeout;

use crate::network::{Connection, Protocol};
use crate::os::shared::get_datalink_channel;

const PACKET_WAIT_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(10);
const CHANNEL_RESET_DELAY: std::time::Duration = std::time::Duration::from_millis(1000);

#[derive(Debug)]
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
    pub fn new(network_interface_ips: &[IpNetwork], source: IpAddr) -> Self {
        if network_interface_ips
            .iter()
            .any(|ip_network| ip_network.ip() == source)
        {
            Direction::Upload
        } else {
            Direction::Download
        }
    }
}

trait NextLevelProtocol {
    fn get_next_level_protocol(&self) -> IpNextHeaderProtocol;
}

impl NextLevelProtocol for Ipv6Packet<'_> {
    fn get_next_level_protocol(&self) -> IpNextHeaderProtocol {
        self.get_next_header()
    }
}

macro_rules! extract_transport_protocol {
    (  $ip_packet: ident ) => {{
        match $ip_packet.get_next_level_protocol() {
            IpNextHeaderProtocols::Tcp => {
                let message = TcpPacket::new($ip_packet.payload())?;
                (
                    Protocol::Tcp,
                    message.get_source(),
                    message.get_destination(),
                    $ip_packet.payload().len() as u128,
                )
            }
            IpNextHeaderProtocols::Udp => {
                let datagram = UdpPacket::new($ip_packet.payload())?;
                (
                    Protocol::Udp,
                    datagram.get_source(),
                    datagram.get_destination(),
                    $ip_packet.payload().len() as u128,
                )
            }
            _ => return None,
        }
    }};
}

pub struct Sniffer {
    network_interface: NetworkInterface,
    network_frames: Box<dyn DataLinkReceiver>,
    dns_shown: bool,
    payload_offset: usize,
}

impl Sniffer {
    pub fn new(
        network_interface: NetworkInterface,
        network_frames: Box<dyn DataLinkReceiver>,
        dns_shown: bool,
        with_payload_offset: bool,
    ) -> Self {
        let payload_offset = {
            // See https://github.com/libpnet/libpnet/blob/master/examples/packetdump.rs
            // The pnet code for BPF loopback adds a zero'd out Ethernet header
            if cfg!(target_os = "macos")
                && (network_interface.is_loopback()
                    // utun interfaces shouldn't need the offset, but user can add with a flag
                    || (with_payload_offset && network_interface.is_point_to_point()))
            {
                14
            } else {
                0
            }
        };
        Sniffer {
            network_interface,
            network_frames,
            dns_shown,
            payload_offset,
        }
    }
    pub fn next(&mut self) -> Option<Segment> {
        let bytes = match self.network_frames.next() {
            Ok(bytes) => bytes,
            Err(err) => match err.kind() {
                std::io::ErrorKind::TimedOut => {
                    park_timeout(PACKET_WAIT_TIMEOUT);
                    return None;
                }
                _ => {
                    park_timeout(CHANNEL_RESET_DELAY);
                    self.reset_channel().ok();
                    return None;
                }
            },
        };

        let ip_packet = Ipv4Packet::new(&bytes[self.payload_offset..])?;
        let version = ip_packet.get_version();

        match version {
            4 => Self::handle_v4(ip_packet, &self.network_interface, self.dns_shown),
            6 => Self::handle_v6(
                Ipv6Packet::new(&bytes[self.payload_offset..])?,
                &self.network_interface,
            ),
            _ => {
                let pkg = EthernetPacket::new(bytes)?;
                match pkg.get_ethertype() {
                    EtherTypes::Ipv4 => Self::handle_v4(
                        Ipv4Packet::new(pkg.payload())?,
                        &self.network_interface,
                        self.dns_shown,
                    ),
                    EtherTypes::Ipv6 => {
                        Self::handle_v6(Ipv6Packet::new(pkg.payload())?, &self.network_interface)
                    }
                    _ => None,
                }
            }
        }
    }
    pub fn reset_channel(&mut self) -> Result<()> {
        self.network_frames = get_datalink_channel(&self.network_interface)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Interface not available"))?;
        Ok(())
    }
    fn handle_v6(ip_packet: Ipv6Packet, network_interface: &NetworkInterface) -> Option<Segment> {
        let (protocol, source_port, destination_port, data_length) =
            extract_transport_protocol!(ip_packet);

        let interface_name = network_interface.name.clone();
        let direction = Direction::new(&network_interface.ips, ip_packet.get_source().into());
        let from = SocketAddr::new(ip_packet.get_source().into(), source_port);
        let to = SocketAddr::new(ip_packet.get_destination().into(), destination_port);

        let connection = match direction {
            Direction::Download => Connection::new(from, to.ip(), destination_port, protocol),
            Direction::Upload => Connection::new(to, from.ip(), source_port, protocol),
        };
        Some(Segment {
            interface_name,
            connection,
            data_length,
            direction,
        })
    }
    fn handle_v4(
        ip_packet: Ipv4Packet,
        network_interface: &NetworkInterface,
        show_dns: bool,
    ) -> Option<Segment> {
        let (protocol, source_port, destination_port, data_length) =
            extract_transport_protocol!(ip_packet);

        let interface_name = network_interface.name.clone();
        let direction = Direction::new(&network_interface.ips, ip_packet.get_source().into());
        let from = SocketAddr::new(ip_packet.get_source().into(), source_port);
        let to = SocketAddr::new(ip_packet.get_destination().into(), destination_port);

        let connection = match direction {
            Direction::Download => Connection::new(from, to.ip(), destination_port, protocol),
            Direction::Upload => Connection::new(to, from.ip(), source_port, protocol),
        };

        if !show_dns && connection.remote_socket.port == 53 {
            return None;
        }
        Some(Segment {
            interface_name,
            connection,
            data_length,
            direction,
        })
    }
}
