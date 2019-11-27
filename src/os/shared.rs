use ::pnet::datalink::Channel::Ethernet;
use ::pnet::datalink::DataLinkReceiver;
use ::pnet::datalink::{self, Config, NetworkInterface};
use ::std::io::{self, stdin, Write};
use ::termion::event::Event;
use ::termion::input::TermRead;

use ::std::net::IpAddr;
use ::std::time;

use signal_hook::iterator::Signals;

use crate::OsInputOutput;
#[cfg(target_os = "linux")]
use crate::os::linux::get_open_sockets;
#[cfg(target_os = "macos")]
use crate::os::macos::get_open_sockets;

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

fn get_datalink_channel(
    interface: &NetworkInterface,
) -> Result<Box<dyn DataLinkReceiver>, failure::Error> {
    let mut config = Config::default();
    config.read_timeout = Some(time::Duration::new(0, 1));
    match datalink::channel(interface, config) {
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

fn create_write_to_stdout() -> Box<dyn FnMut(String) + Send> {
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
