use ::pnet::datalink::Channel::Ethernet;
use ::pnet::datalink::DataLinkReceiver;
use ::pnet::datalink::{self, Config, NetworkInterface};
use ::std::io::{self, stdin, Write};
use ::termion::event::Event;
use ::termion::input::TermRead;

use ::std::collections::HashMap;
use ::std::net::IpAddr;

use std::process::Command;
use regex::{Regex};

use signal_hook::iterator::Signals;

use crate::network::{Connection, Protocol};
use crate::OsInputOutput;

use std::net::{SocketAddr};

struct KeyboardEvents;

impl Iterator for KeyboardEvents {
    type Item = Event;
    fn next(&mut self) -> Option<Event> {
        match stdin().events().next() {
            Some(Ok(ev)) => Some(ev),
            _ => None,
        }
    }
}

fn get_datalink_channel(
    interface: &NetworkInterface,
) -> Result<Box<dyn DataLinkReceiver>, failure::Error> {
    match datalink::channel(interface, Config::default()) {
        Ok(Ethernet(_tx, rx)) => Ok(rx),
        Ok(_) => failure::bail!("Unknown interface type"),
        Err(e) => failure::bail!("Failed to listen to network interface: {}", e),
    }
}

fn get_interface(interface_name: &str) -> Option<NetworkInterface> {
    datalink::interfaces()
        .into_iter()
        .find(|iface| iface.name == interface_name)
}

#[derive(Debug)]
struct RawConnection {
    ip: String,
    local_port: String,
    remote_port: String,
    protocol: String,
    process_name: String,
}

fn get_open_sockets() -> HashMap<Connection, String> {
    let mut open_sockets = HashMap::new();

    let output = Command::new("lsof")
            .args(&["-n","-P", "-i4"])//"4tcp"
            .output()
            .expect("failed to execute process");

    let regex = Regex::new(r"([^\s]+).*(TCP|UDP).*:(.*)->(.*):(\d*)(\s|$)").unwrap();

    let output_string = String::from_utf8(output.stdout).unwrap();
    let lines = output_string.lines();

    for line in lines {
        let raw_connection_iter = regex.captures_iter(line).filter_map(|cap| {
            let process_name = String::from(cap.get(1).unwrap().as_str());
            let protocol = String::from(cap.get(2).unwrap().as_str());
            let local_port = String::from(cap.get(3).unwrap().as_str());
            let ip = String::from(cap.get(4).unwrap().as_str());
            let remote_port = String::from(cap.get(5).unwrap().as_str());
            let connection = RawConnection{process_name, ip,local_port, remote_port, protocol};
            Some(connection)
        });

        let raw_connection_vec = raw_connection_iter.map(|m| m).collect::<Vec<_>>();

        if let Some(raw_connection) = raw_connection_vec.first() {
            let protocol = Protocol::from_string(&raw_connection.protocol).unwrap();
            let ip_address = IpAddr::V4(raw_connection.ip.parse().unwrap());
            let remote_port = raw_connection.remote_port.parse::<u16>().unwrap();
            let local_port = raw_connection.local_port.parse::<u16>().unwrap();

            let socket_addr = SocketAddr::new(ip_address, remote_port);
            let connection = Connection::new(socket_addr, local_port, protocol).unwrap();

            open_sockets.insert(connection, raw_connection.process_name.clone());
        }
    }

    return open_sockets;
}

fn lookup_addr(ip: &IpAddr) -> Option<String> {
    ::dns_lookup::lookup_addr(ip).ok()
}

fn sigwinch() -> (Box<dyn Fn(Box<dyn Fn()>) + Send>, Box<dyn Fn() + Send>) {
    let signals = Signals::new(&[signal_hook::SIGWINCH]).unwrap();
    let on_winch = {
        let signals = signals.clone();
        move |cb: Box<dyn Fn()>| {
            for signal in signals.forever() {
                match signal {
                    signal_hook::SIGWINCH => cb(),
                    _ => unreachable!(),
                }
            }
        }
    };
    let cleanup = move || {
        signals.close();
    };
    (Box::new(on_winch), Box::new(cleanup))
}

pub fn create_write_to_stdout() -> Box<dyn FnMut(String) + Send> {
    Box::new({
        let mut stdout = io::stdout();
        move |output: String| {
            writeln!(stdout, "{}", output).unwrap();
        }
    })
}

pub fn get_input(interface_name: &str) -> Result<OsInputOutput, failure::Error> {
    let keyboard_events = Box::new(KeyboardEvents);
    let network_interface = match get_interface(interface_name) {
        Some(interface) => interface,
        None => {
            failure::bail!("Cannot find interface {}", interface_name);
        }
    };
    let network_frames = get_datalink_channel(&network_interface)?;
    let lookup_addr = Box::new(lookup_addr);
    let write_to_stdout = create_write_to_stdout();
    let (on_winch, cleanup) = sigwinch();

    Ok(OsInputOutput {
        network_interface,
        network_frames,
        get_open_sockets,
        keyboard_events,
        lookup_addr,
        on_winch,
        cleanup,
        write_to_stdout,
    })
}
