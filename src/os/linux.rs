use ::pnet::datalink::Channel::Ethernet;
use ::pnet::datalink::DataLinkReceiver;
use ::pnet::datalink::{self, Config, NetworkInterface};
use ::std::io::stdin;
use ::termion::event::Event;
use ::termion::input::TermRead;

use ::std::collections::HashMap;
use ::std::net::IpAddr;
use ::std::sync::Arc;
use ::std::time;

use ::std::sync::atomic::AtomicBool;

use ::signal_hook;

use ::procfs::FDTarget;

use crate::network::{Connection, Protocol};

pub struct KeyboardEvents;

impl Iterator for KeyboardEvents {
    type Item = Event;
    fn next(&mut self) -> Option<Event> {
        match stdin().events().next() {
            Some(Ok(ev)) => Some(ev),
            _ => None,
        }
    }
}

pub fn get_datalink_channel(interface: &NetworkInterface) -> Box<DataLinkReceiver> {
    let mut config = Config::default();
    config.read_timeout = Some(time::Duration::new(0, 1));
    match datalink::channel(interface, config) {
        Ok(Ethernet(_tx, rx)) => rx,
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!(
            "An error occurred when creating the datalink channel: {}",
            e
        ),
    }
}

pub fn get_interface(interface_name: &str) -> Option<NetworkInterface> {
    datalink::interfaces()
        .into_iter()
        .find(|iface| iface.name == interface_name)
}

pub fn get_open_sockets() -> HashMap<Connection, String> {
    let mut open_sockets = HashMap::new(); // TODO: better
    let all_procs = procfs::all_processes();

    let mut inode_to_procname = HashMap::new();
    for process in all_procs {
        if let Ok(fds) = process.fd() {
            let procname = process.stat.comm;
            for fd in fds {
                if let FDTarget::Socket(inode) = fd.target {
                    inode_to_procname.insert(inode, procname.clone());
                }
            }
        }
    }

    let tcp = ::procfs::tcp().unwrap();
    for entry in tcp.into_iter() {
        let local_port = entry.local_address.port();
        if let (Some(connection), Some(procname)) = (
            Connection::new(entry.remote_address, local_port, Protocol::Tcp),
            inode_to_procname.get(&entry.inode),
        ) {
            open_sockets.insert(connection, procname.clone());
        };
    }

    let udp = ::procfs::udp().unwrap();
    for entry in udp.into_iter() {
        let local_port = entry.local_address.port();
        if let (Some(connection), Some(procname)) = (
            Connection::new(entry.remote_address, local_port, Protocol::Udp),
            inode_to_procname.get(&entry.inode),
        ) {
            open_sockets.insert(connection, procname.clone());
        };
    }
    open_sockets
}

pub fn lookup_addr(ip: &IpAddr) -> Option<String> {
    ::dns_lookup::lookup_addr(ip).ok()
}

pub fn receive_winch(winch: &Arc<AtomicBool>) {
    ::signal_hook::flag::register(signal_hook::SIGWINCH, winch.clone())
        .expect("Failed to register SIGWINCH");
}
