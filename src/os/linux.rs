use ::std::env;
use ::std::io::stdin;
use ::pnet::datalink::{self, NetworkInterface};
use ::pnet::datalink::DataLinkReceiver;
use ::pnet::datalink::Channel::Ethernet;
use ::termion::event::Event;
use ::termion::input::{TermRead};
use ::netstat::{SocketInfo, AddressFamilyFlags, ProtocolFlags, get_sockets_info};
use ::procfs::Process;

pub struct KeyboardEvents {}

impl Iterator for KeyboardEvents {
    type Item = Event;
    fn next(&mut self) -> Option<Event> {
        let stdin = stdin();
        let mut events = stdin.events(); // TODO: not every time?
        match events.next() {
            Some(res) => {
                match res {
                    Ok(ev) => Some(ev),
                    Err(_) => None
                }
            },
            None => None
        }
    }
}

pub fn get_datalink_channel (interface: &NetworkInterface) -> Box<DataLinkReceiver> {
    let (_tx, rx) = match datalink::channel(interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!("An error occurred when creating the datalink channel: {}", e)
    };
    rx
}

pub fn get_interface () -> NetworkInterface {
    let interface_name = env::args().nth(1).unwrap(); // TODO: figure this out without arg
    let interface_names_match =
        |iface: &NetworkInterface| iface.name == interface_name;
    let interfaces = datalink::interfaces();
    let interface = interfaces.into_iter()
                              .filter(interface_names_match)
                              .next()
                              .unwrap();
    interface
}

pub fn get_process_name (id: i32) -> Option<String> {
    match Process::new(id) {
        Ok(process) => Some(process.stat.comm.to_string()),
        Err(_) => None
    }
}

pub fn get_open_sockets () -> Vec<SocketInfo> {
    let af_flags = AddressFamilyFlags::IPV4;
    let proto_flags = ProtocolFlags::TCP | ProtocolFlags::UDP;
    match get_sockets_info(af_flags, proto_flags) {
        Ok(sockets_info) => sockets_info,
        Err(_) => vec![]
    }
}
