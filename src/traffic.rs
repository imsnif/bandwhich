use ::std::fmt;
use ::std::env;
use ::std::net::Ipv4Addr;
use ::std::error::Error;
use ::std::boxed::Box;

use ::pnet::datalink::{self, NetworkInterface};
use ::pnet::datalink::{DataLinkReceiver, Channel};
use ::pnet::datalink::Channel::Ethernet;
use ::pnet::packet::{Packet, MutablePacket};
use ::pnet::packet::ethernet::{EtherType, EthernetPacket, MutableEthernetPacket};
use ::pnet::packet::ipv4::Ipv4Packet;
use ::pnet::packet::ip::IpNextHeaderProtocol;
use ::pnet::packet::tcp::TcpPacket;
use ::pnet::packet::udp::UdpPacket;

use ::num_bigint::{BigUint, ToBigUint};
use ::num_traits::{Zero, One};

use ::ipnetwork::IpNetwork;


pub struct Sniffer {
    interface: NetworkInterface,
    channel: Box<DataLinkReceiver>,
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
        write!(f, "{}:{} => {}:{} ({})", self.local_ip, self.local_port, self.remote_ip, self.remote_port, self.protocol)
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

pub struct FakeReceiver {
}
impl DataLinkReceiver for FakeReceiver {
    fn next (&mut self) -> Result<&[u8], std::io::Error > {
        Ok(&[11])
    }
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       match *self {
           Protocol::Tcp => write!(f, "tcp"),
           Protocol::Udp => write!(f, "udp")
       }
    }
}

impl Segment {
    pub fn from_tcp_message (ip_packet: &Ipv4Packet, self_ip_addresses: &Vec<IpNetwork>, message: TcpPacket) -> Self {
        let protocol = Protocol::Tcp;
        let direction = match self_ip_addresses.iter().any(|ip_network| ip_network.ip() == ip_packet.get_source()) {
            true => Direction::Upload,
            false => Direction::Download
        };
        let connection = match direction {
            Direction::Upload => {
                let local_ip = ip_packet.get_source();
                let remote_ip = ip_packet.get_destination();
                let local_port = message.get_source();
                let remote_port = message.get_destination();
                Connection { local_ip, remote_ip, local_port, remote_port, protocol }
            },
            Direction::Download => {
                let local_ip = ip_packet.get_destination();
                let remote_ip = ip_packet.get_source();
                let local_port = message.get_destination();
                let remote_port = message.get_source();
                Connection { local_ip, remote_ip, local_port, remote_port, protocol }
            }
        };
        let ip_length = ip_packet.get_total_length().to_biguint().unwrap();
        Segment { connection, ip_length, direction }
    }
    pub fn from_udp_datagram (ip_packet: &Ipv4Packet, self_ip_addresses: &Vec<IpNetwork>, datagram: UdpPacket) -> Self {
        // TODO: only leave the datagram parts here and merge the rest to a third generic method
        let protocol = Protocol::Udp;
        let direction = match self_ip_addresses.iter().any(|ip_network| ip_network.ip() == ip_packet.get_source()) {
            true => Direction::Upload,
            false => Direction::Download
        };
        let connection = match direction {
            Direction::Upload => {
                let local_ip = ip_packet.get_source();
                let remote_ip = ip_packet.get_destination();
                let local_port = datagram.get_source();
                let remote_port = datagram.get_destination();
                Connection { local_ip, remote_ip, local_port, remote_port, protocol }
            },
            Direction::Download => {
                let local_ip = ip_packet.get_destination();
                let remote_ip = ip_packet.get_source();
                let local_port = datagram.get_destination();
                let remote_port = datagram.get_source();
                Connection { local_ip, remote_ip, local_port, remote_port, protocol }
            }
        };
        let ip_length = ip_packet.get_total_length().to_biguint().unwrap();
        Segment { connection, ip_length, direction }
    }
}

// #[cfg(not(feature = "test"))]
// fn get_interface_name_from_arg() -> String {
//     env::args().nth(1).unwrap() // TODO: figure this out without arg
// }
// 
// #[cfg(feature = "test")]
// fn get_interface_name_from_arg() -> String {
//     String::from("foo")
//     // env::args().nth(1).unwrap() // TODO: figure this out without arg
// }
// 
// #[cfg(not(feature = "test"))]
// fn get_interface () -> NetworkInterface {
//     let interface_name = get_interface_name_from_arg();
//     let interface_names_match =
//         |iface: &NetworkInterface| iface.name == interface_name;
//     // Find the network interface with the provided name
//     let interfaces = datalink::interfaces();
//     let interface = interfaces.into_iter()
//                               .filter(interface_names_match)
//                               .next()
//                               .unwrap();
//     interface
// }

// #[cfg(feature = "test")]
// fn get_interface () -> NetworkInterface {
//     let interface = NetworkInterface {
//         name: String::from("foo"),
//         index: 42,
//         mac: None,
//         ips: vec!(IpNetwork::V4("1.1.1.1".parse().unwrap())),
//         flags: 42
//     };
//     interface
// }
// 
// #[cfg(not(feature = "test"))]
// fn get_channel (interface: &NetworkInterface) -> Box<DataLinkReceiver> {
//     let (_tx, rx) = match datalink::channel(interface, Default::default()) {
//         Ok(Ethernet(tx, rx)) => (tx, rx),
//         Ok(_) => panic!("Unhandled channel type"),
//         Err(e) => panic!("An error occurred when creating the datalink channel: {}", e)
//     };
//     rx
// }
// 
// 
// #[cfg(feature = "test")]
// fn get_channel (interface: &NetworkInterface) -> Box<DataLinkReceiver> {
//     Box::new(FakeReceiver {})
// }

impl Sniffer {
    // pub fn new <I, C> (get_interface: &I, get_channel: &C) -> Self where
    pub fn new (interface: NetworkInterface, channel: Box<DataLinkReceiver>) -> Self where
//        I: &NetworkInterface,
//        C: DataLinkReceiver
//        I: Fn() -> NetworkInterface,
//        C: Fn(&NetworkInterface) -> Box<DataLinkReceiver>
    {
//        let interface = get_interface();
//        let rx = get_channel(&interface);

        Sniffer { interface, channel }
        // Sniffer { interface, channel: rx }
    }
    pub fn next(&mut self) -> Option<Segment> {
        // TODO: https://github.com/libpnet/libpnet/issues/343
        // make this non-blocking for faster exits
        match self.channel.next() {
            // Ok(packet) => {
            Ok(bytes) => {
                // let packet = EthernetPacket::new(packet).unwrap();
//                println!("********************************** bytes ***********************");
//                println!("{:?}", bytes);
//                println!("********************************** bytes ***********************");
                match EthernetPacket::new(bytes) {
                    Some(packet) => {
                        match packet.get_ethertype() { // TODO: better way through the module?
                            EtherType(2048) => {
                                let ip_packet = Ipv4Packet::new(packet.payload()).unwrap();
                                match ip_packet.get_next_level_protocol() {
                                    IpNextHeaderProtocol(6) => { // tcp
                                        // Some(ip_packet.get_total_length())
                                        let message = TcpPacket::new(ip_packet.payload()).unwrap();
                                        let segment = Segment::from_tcp_message(&ip_packet, &self.interface.ips, message);
                                        Some(segment)

                                    },
                                    IpNextHeaderProtocol(17) => { // udp
                                        // Some(ip_packet.get_total_length())
                                        let datagram = UdpPacket::new(ip_packet.payload()).unwrap();
                                        let segment = Segment::from_udp_datagram(&ip_packet, &self.interface.ips, datagram);
                                        Some(segment)
                                    },
                                    _ => {
                                        None
                                        // self.next()
                                    }
                                }
                            },
                            _ => {
                                None
                                // self.next()
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
    pub fn get_interface(&self) -> &NetworkInterface {
        &self.interface
    }

}

// impl fmt::Display for IncomingTraffic {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         // TBD
//         // write!(f, "({}, {})", self.x, self.y)
//     }
// }
