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

use crate::network::{Connection, Protocol};
use crate::os::shared::get_datalink_channel;

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
}

impl Sniffer {
    pub fn new(
        network_interface: NetworkInterface,
        network_frames: Box<dyn DataLinkReceiver>,
        dns_shown: bool,
    ) -> Self {
        Sniffer {
            network_interface,
            network_frames,
            dns_shown,
        }
    }
    pub fn next(&mut self) -> Result<Option<Segment>> {
        let bytes = self.network_frames.next()?;
        // See https://github.com/libpnet/libpnet/blob/master/examples/packetdump.rs
        // VPN interfaces (such as utun0, utun1, etc) have POINT_TO_POINT bit set to 1
        let payload_offset = if (self.network_interface.is_loopback()
            || self.network_interface.is_point_to_point())
            && cfg!(target_os = "macos")
        {
            // The pnet code for BPF loopback adds a zero'd out Ethernet header
            14
        } else {
            0
        };
        if let Some(ip) = Ipv4Packet::new(&bytes[payload_offset..]) {
            let payload = &bytes[payload_offset..];
            match ip.get_version() {
                4 => {
                    return Ok(Self::handle_v4(
                        payload,
                        &self.network_interface,
                        self.dns_shown,
                    ))
                }
                6 => return Ok(Self::handle_v6(payload, &self.network_interface)),
                _ => (),
            }
        }
        if let Some(pkt) = EthernetPacket::new(&bytes[payload_offset..]) {
            match pkt.get_ethertype() {
                EtherTypes::Ipv4 => {
                    return Ok(Self::handle_v4(
                        &pkt.payload(),
                        &self.network_interface,
                        self.dns_shown,
                    ))
                }
                EtherTypes::Ipv6 => {
                    return Ok(Self::handle_v6(&pkt.payload(), &self.network_interface))
                }
                _ => (),
            }
        }
        Ok(None)
    }
    pub fn reset_channel(&mut self) -> Result<()> {
        self.network_frames = get_datalink_channel(&self.network_interface)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Interface not available"))?;
        Ok(())
    }
    fn handle_v6(bytes: &[u8], network_interface: &NetworkInterface) -> Option<Segment> {
        let ip_packet = Ipv6Packet::new(bytes)?;
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
        bytes: &[u8],
        network_interface: &NetworkInterface,
        show_dns: bool,
    ) -> Option<Segment> {
        let ip_packet = Ipv4Packet::new(bytes)?;
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
