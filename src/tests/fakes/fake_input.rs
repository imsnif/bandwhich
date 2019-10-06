use ::ipnetwork::IpNetwork;
use ::pnet::datalink::DataLinkReceiver;
use ::pnet::datalink::NetworkInterface;
use ::std::collections::HashMap;
use ::std::net::{IpAddr, Ipv4Addr, SocketAddr};
use ::std::{thread, time};
use ::termion::event::Event;

use ::std::sync::atomic::{AtomicBool, Ordering};
use ::std::sync::Arc;

use crate::network::{Connection, Protocol};

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
            Some(ev) => {
                match ev {
                    Some(ev) => Some(ev), // TODO: better
                    None => {
                        thread::sleep(time::Duration::from_secs(1));
                        self.next()
                    }
                }
            }
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
        match self.current_index < self.packets.len() {
            true => {
                let action = self.next_packet();
                match action {
                    Some(packet) => {
                        Ok(&packet[..]) // TODO: better
                    }
                    None => {
                        thread::sleep(time::Duration::from_secs(1));
                        Ok(&[][..])
                    }
                }
            }
            false => {
                thread::sleep(time::Duration::from_secs(1));
                Ok(&[][..])
            }
        }
    }
}

pub fn get_open_sockets() -> HashMap<Connection, String> {
    let mut open_sockets = HashMap::new();
    open_sockets.insert(
        Connection::new(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)), 443),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), 12345),
            Protocol::Tcp,
        )
        .unwrap(),
        String::from("1"),
    );
    open_sockets.insert(
        Connection::new(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)), 443),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(2, 2, 2, 2)), 54321),
            Protocol::Tcp,
        )
        .unwrap(),
        String::from("4"),
    );
    open_sockets.insert(
        Connection::new(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)), 443),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(3, 3, 3, 3)), 1337),
            Protocol::Tcp,
        )
        .unwrap(),
        String::from("5"),
    );
    open_sockets.insert(
        Connection::new(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)), 443),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(4, 4, 4, 4)), 1337),
            Protocol::Tcp,
        )
        .unwrap(),
        String::from("2"),
    );
    open_sockets.insert(
        Connection::new(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)), 443),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), 12346),
            Protocol::Tcp,
        )
        .unwrap(),
        String::from("3"),
    );
    open_sockets
}

pub fn get_interface() -> NetworkInterface {
    let interface = NetworkInterface {
        name: String::from("foo"),
        index: 42,
        mac: None,
        ips: vec![IpNetwork::V4("10.0.0.2".parse().unwrap())],
        flags: 42,
    };
    interface
}

pub fn create_fake_lookup_addr(
    ips_to_hosts: HashMap<IpAddr, String>,
) -> Box<Fn(&IpAddr) -> Option<String> + Send + Sync + 'static> {
    Box::new(move |ip| match ips_to_hosts.get(ip) {
        Some(host) => Some(host.clone()),
        None => None,
    })
}

pub fn create_fake_receive_winch(should_send_winch_event: bool) -> Box<Fn(&Arc<AtomicBool>)> {
    Box::new(move |winch| {
        if should_send_winch_event {
            thread::sleep(time::Duration::from_secs(1));
            winch.store(true, Ordering::Relaxed);
        }
    })
}
