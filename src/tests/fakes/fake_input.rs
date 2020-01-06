use ::async_trait::async_trait;
use ::ipnetwork::IpNetwork;
use ::pnet::datalink::DataLinkReceiver;
use ::pnet::datalink::NetworkInterface;
use ::std::collections::HashMap;
use ::std::future::Future;
use ::std::net::{IpAddr, Ipv4Addr, SocketAddr};
use ::std::pin::Pin;
use ::std::task::{Context, Poll};
use ::std::{thread, time};
use ::termion::event::Event;

use crate::{
    network::{
        dns::{self, Lookup},
        Connection, Protocol, LocalSocket
    },
    os::OnSigWinch,
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

pub fn get_open_sockets() -> (HashMap<LocalSocket, String>, std::vec::Vec<Connection>) {
    let mut open_sockets = HashMap::new();
    let local_ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
    open_sockets.insert(
        Connection::new(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), 12345),
            local_ip,
            443,
            Protocol::Tcp,
        )
        .unwrap(),
        String::from("1"),
    );
    open_sockets.insert(
        Connection::new(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(2, 2, 2, 2)), 54321),
            local_ip,
            4434,
            Protocol::Tcp,
        )
        .unwrap(),
        String::from("4"),
    );
    open_sockets.insert(
        Connection::new(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(3, 3, 3, 3)), 1337),
            local_ip,
            4435,
            Protocol::Tcp,
        )
        .unwrap(),
        String::from("5"),
    );
    open_sockets.insert(
        Connection::new(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(4, 4, 4, 4)), 1337),
            local_ip,
            4432,
            Protocol::Tcp,
        )
        .unwrap(),
        String::from("2"),
    );
    open_sockets.insert(
        Connection::new(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), 12346),
            local_ip,
            443,
            Protocol::Tcp,
        )
        .unwrap(),
        String::from("1"),
    );
    let mut local_socket_to_procs = HashMap::new();
    let mut connections = std::vec::Vec::new();
    for (connection, process_name) in open_sockets {
        local_socket_to_procs.insert(connection.local_socket, process_name);
        connections.push(connection);
    }

    (local_socket_to_procs, connections)
}

pub fn get_interfaces() -> Vec<NetworkInterface> {
    vec![NetworkInterface {
        name: String::from("interface_name"),
        index: 42,
        mac: None,
        ips: vec![IpNetwork::V4("10.0.0.2".parse().unwrap())],
        flags: 42,
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
    let dns_client = dns::Client::new(FakeResolver(ips_to_hosts), FakeBackground {}).unwrap();
    Some(dns_client)
}

struct FakeResolver(HashMap<IpAddr, String>);

#[async_trait]
impl Lookup for FakeResolver {
    async fn lookup(&self, ip: Ipv4Addr) -> Option<String> {
        let ip = IpAddr::from(ip);
        self.0.get(&ip).cloned()
    }
}

struct FakeBackground {}

impl Future for FakeBackground {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(())
    }
}
