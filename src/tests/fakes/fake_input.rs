use ::async_trait::async_trait;
use ::crossterm::event::Event;
use ::ipnetwork::IpNetwork;
use ::pnet::datalink::DataLinkReceiver;
use ::pnet::datalink::NetworkInterface;
use ::std::collections::HashMap;
use ::std::net::{IpAddr, Ipv4Addr, SocketAddr};
use ::std::{thread, time};
use ::tokio::runtime::Runtime;

use crate::{
    network::{
        dns::{self, Lookup},
        Connection, Protocol,
    },
    os::OnSigWinch,
    OpenSockets,
};

pub struct KeyboardEvents {
    pub events: Vec<Option<Event>>,
}

impl KeyboardEvents {
    pub fn new(mut events: Vec<Option<Event>>) -> Self {
        events.reverse(); // this is so that we do not have to shift the array
        KeyboardEvents { events }
    }
}
impl Iterator for KeyboardEvents {
    type Item = Event;
    fn next(&mut self) -> Option<Event> {
        match self.events.pop() {
            Some(ev) => match ev {
                Some(ev) => Some(ev),
                None => {
                    thread::sleep(time::Duration::from_millis(900));
                    self.next()
                }
            },
            None => None,
        }
    }
}

pub struct NetworkFrames {
    pub packets: Vec<Option<Vec<u8>>>,
    pub current_index: usize,
}

impl NetworkFrames {
    pub fn new(packets: Vec<Option<Vec<u8>>>) -> Box<Self> {
        Box::new(NetworkFrames {
            packets,
            current_index: 0,
        })
    }
    fn next_packet(&mut self) -> &Option<Vec<u8>> {
        let next_index = self.current_index;
        self.current_index += 1;
        &self.packets[next_index]
    }
}
impl DataLinkReceiver for NetworkFrames {
    fn next(&mut self) -> Result<&[u8], std::io::Error> {
        if self.current_index == 0 {
            // make it less likely to have a race condition with the display loop
            // this is so the tests pass consistently
            thread::sleep(time::Duration::from_millis(500));
        }
        if self.current_index < self.packets.len() {
            let action = self.next_packet();
            match action {
                Some(packet) => Ok(&packet[..]),
                None => {
                    thread::sleep(time::Duration::from_secs(1));
                    Ok(&[][..])
                }
            }
        } else {
            thread::sleep(time::Duration::from_secs(1));
            Ok(&[][..])
        }
    }
}

pub fn get_open_sockets() -> OpenSockets {
    let mut open_sockets = HashMap::new();
    let local_ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
    open_sockets.insert(
        Connection::new(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), 12345),
            local_ip,
            443,
            Protocol::Tcp,
        ),
        String::from("1"),
    );
    open_sockets.insert(
        Connection::new(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(2, 2, 2, 2)), 54321),
            local_ip,
            4434,
            Protocol::Tcp,
        ),
        String::from("4"),
    );
    open_sockets.insert(
        Connection::new(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(3, 3, 3, 3)), 1337),
            local_ip,
            4435,
            Protocol::Tcp,
        ),
        String::from("5"),
    );
    open_sockets.insert(
        Connection::new(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(4, 4, 4, 4)), 1337),
            local_ip,
            4432,
            Protocol::Tcp,
        ),
        String::from("2"),
    );
    open_sockets.insert(
        Connection::new(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), 12346),
            local_ip,
            443,
            Protocol::Tcp,
        ),
        String::from("1"),
    );
    let mut local_socket_to_procs = HashMap::new();
    let mut connections = std::vec::Vec::new();
    for (connection, process_name) in open_sockets {
        local_socket_to_procs.insert(connection.local_socket, process_name);
        connections.push(connection);
    }

    OpenSockets {
        sockets_to_procs: local_socket_to_procs,
    }
}

pub fn get_interfaces() -> Vec<NetworkInterface> {
    vec![NetworkInterface {
        name: String::from("interface_name"),
        index: 42,
        mac: None,
        ips: vec![IpNetwork::V4("10.0.0.2".parse().unwrap())],
        // It's important that the IFF_LOOPBACK bit is set to 0.
        // Otherwise sniffer will attempt to start parse packets
        // at offset 14
        flags: 0,
    }]
}

pub fn create_fake_on_winch(should_send_winch_event: bool) -> Box<OnSigWinch> {
    Box::new(move |cb| {
        if should_send_winch_event {
            thread::sleep(time::Duration::from_millis(900));
            cb()
        }
    })
}

pub fn create_fake_dns_client(ips_to_hosts: HashMap<IpAddr, String>) -> Option<dns::Client> {
    let runtime = Runtime::new().unwrap();
    let dns_client = dns::Client::new(FakeResolver(ips_to_hosts), runtime).unwrap();
    Some(dns_client)
}

struct FakeResolver(HashMap<IpAddr, String>);

#[async_trait]
impl Lookup for FakeResolver {
    async fn lookup(&self, ip: IpAddr) -> Option<String> {
        self.0.get(&ip).cloned()
    }
}
