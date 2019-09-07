use ::std::io;
use ::std::env;
use ::std::io::{Write, stdout, stdin, Stdout};
use ::termion::raw::IntoRawMode;
use ::tui::backend::TermionBackend;
use ::pnet::datalink::{self, NetworkInterface};
use ::pnet::datalink::{DataLinkReceiver, Channel};
use ::pnet::datalink::Channel::Ethernet;

use ::termion::event::{Key, Event, MouseEvent};
use ::termion::input::{TermRead};

use ::netstat::*;

use ::procfs::Process;

#[derive(Debug)]
struct GenericProcess {
    proc: Process
}

struct InputEvents {
    // events: termion::input::Events<String>
}

impl Iterator for InputEvents {
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

impl what::display::IsProcess for GenericProcess {
    fn get_name (&self) -> String {
        self.proc.stat.comm.to_string()
    }
}

fn get_channel (interface: &NetworkInterface) -> Box<DataLinkReceiver> {
    let (_tx, rx) = match datalink::channel(interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!("An error occurred when creating the datalink channel: {}", e)
    };
    rx
}

fn get_interface () -> NetworkInterface {
    let interface_name = env::args().nth(1).unwrap(); // TODO: figure this out without arg
    let interface_names_match =
        |iface: &NetworkInterface| iface.name == interface_name;
    // Find the network interface with the provided name
    let interfaces = datalink::interfaces();
    let interface = interfaces.into_iter()
                              .filter(interface_names_match)
                              .next()
                              .unwrap();
    interface
}

fn create_process (id: i32) -> Result<GenericProcess, Box<std::error::Error>> {
    let proc = Process::new(id)?;
    Ok(GenericProcess {proc})
}

fn main () {
    let stdin_events = InputEvents {};
    let stdout = io::stdout().into_raw_mode().unwrap();
    let backend = TermionBackend::new(stdout);
    let interface = get_interface();
    let channel = get_channel(&interface);
    what::start(backend, &create_process, &get_sockets_info, interface, channel, stdin_events)
}
