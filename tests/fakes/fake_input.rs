use ::ipnetwork::IpNetwork;
use ::pnet::datalink::DataLinkReceiver;
use ::pnet::datalink::NetworkInterface;
use ::std::net::IpAddr;
use ::std::{thread, time};
use ::termion::event::Event;

use ::netstat::*;

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

fn create_fake_socket(
    associated_pids: Vec<u32>,
    local_ip: IpAddr,
    remote_ip: IpAddr,
    local_port: u16,
    remote_port: u16,
) -> SocketInfo {
    let protocol_socket_info = TcpSocketInfo {
        local_addr: local_ip,
        remote_addr: remote_ip,
        local_port: local_port,
        remote_port: remote_port,
        state: TcpState::Listen,
    };
    SocketInfo {
        protocol_socket_info: ProtocolSocketInfo::Tcp(protocol_socket_info),
        associated_pids: associated_pids,
        inode: 2,
    }
}

pub fn get_open_sockets() -> Vec<SocketInfo> {
    vec![
        create_fake_socket(
            vec![1, 2, 3],
            IpAddr::V4("10.0.0.2".parse().unwrap()),
            IpAddr::V4("1.1.1.1".parse().unwrap()),
            443,
            12345,
        ),
        create_fake_socket(
            vec![4],
            IpAddr::V4("10.0.0.2".parse().unwrap()),
            IpAddr::V4("2.2.2.2".parse().unwrap()),
            443,
            54321,
        ),
        create_fake_socket(
            vec![1],
            IpAddr::V4("10.0.0.2".parse().unwrap()),
            IpAddr::V4("3.3.3.3".parse().unwrap()),
            443,
            1337,
        ),
        create_fake_socket(
            vec![4],
            IpAddr::V4("10.0.0.2".parse().unwrap()),
            IpAddr::V4("4.4.4.4".parse().unwrap()),
            443,
            1337,
        ),
        create_fake_socket(
            vec![1, 2, 3],
            IpAddr::V4("10.0.0.2".parse().unwrap()),
            IpAddr::V4("1.1.1.1".parse().unwrap()),
            443,
            12346,
        ),
    ]
}

pub fn get_process_name(id: i32) -> Option<String> {
    Some(id.to_string())
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
