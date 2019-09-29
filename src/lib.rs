mod display;
pub mod network;

use display::display_loop;
use network::{Connection, Sniffer, Utilization, DnsQueue, resolve_connections};

use ::std::net::IpAddr;

use ::pnet::datalink::{DataLinkReceiver, NetworkInterface};
use ::std::collections::HashMap;
use ::std::sync::atomic::{AtomicBool, Ordering};
use ::std::sync::{Arc, Mutex};
use ::std::{thread, time};
use ::termion::event::{Event, Key};
use ::tui::backend::Backend;
use ::tui::Terminal;

pub struct OsInput {
    pub network_interface: NetworkInterface,
    pub network_frames: Box<DataLinkReceiver>,
    pub get_open_sockets: fn() -> HashMap<Connection, String>,
    pub keyboard_events: Box<Iterator<Item = Event> + Send + Sync + 'static>,
    pub lookup_addr: Box<Fn(&IpAddr) -> Option<String> + Send + Sync + 'static>
}

pub fn start<B>(terminal_backend: B, os_input: OsInput)
where
    B: Backend + Send + 'static,
{
    let running = Arc::new(AtomicBool::new(true));

    let keyboard_events = os_input.keyboard_events; // TODO: as methods in os_interface
    let get_open_sockets = os_input.get_open_sockets;
    let lookup_addr = os_input.lookup_addr;

    let stdin_handler = thread::spawn({
        let running = running.clone();
        move || {
            for evt in keyboard_events {
                match evt {
                    Event::Key(Key::Ctrl('c')) | Event::Key(Key::Char('q')) => {
                        // TODO: exit faster
                        running.store(false, Ordering::Relaxed);
                        break;
                    }
                    _ => (),
                };
            }
        }
    });

    let mut sniffer = Sniffer::new(os_input.network_interface, os_input.network_frames);
    let network_utilization = Arc::new(Mutex::new(Utilization::new()));

    let dns_queue = Arc::new(DnsQueue::new());
    let ip_to_host = Arc::new(Mutex::new(HashMap::new()));

    let dns_handler = thread::spawn({
        let running = running.clone();
        let dns_queue = dns_queue.clone();
        let ip_to_host = ip_to_host.clone();
        move || {
            while running.load(Ordering::Relaxed) {
                let jobs = dns_queue.wait_for_jobs();
                for ip in jobs {
                    if let Some(addr) = lookup_addr(&IpAddr::V4(ip.clone())) {
                        ip_to_host.lock().unwrap().insert(ip, addr);
                    }
                }
            }
        }
    });

    let display_handler = thread::spawn({
        let running = running.clone();
        let network_utilization = network_utilization.clone();
        let ip_to_host = ip_to_host.clone();
        let dns_queue = dns_queue.clone();
        move || {
            let mut terminal = Terminal::new(terminal_backend).unwrap();
            terminal.clear().unwrap();
            terminal.hide_cursor().unwrap();
            while running.load(Ordering::Relaxed) {
                let connections_to_procs = {
                    let open_sockets = get_open_sockets();
                    let ip_to_host = ip_to_host.lock().unwrap();
                    let (unresolved_ips, connections_to_procs) = resolve_connections(open_sockets, &ip_to_host);
                    dns_queue.add_ips_to_resolve(unresolved_ips);
                    connections_to_procs
                };
                {
                    let mut network_utilization = network_utilization.lock().unwrap();
                    let utilization = network_utilization.clone_and_reset();
                    display_loop(&utilization, &mut terminal, connections_to_procs);
                }
                thread::sleep(time::Duration::from_secs(1));
            }
            terminal.clear().unwrap();
            terminal.show_cursor().unwrap();
            dns_queue.end();
        }
    });

    let sniffing_handler = thread::spawn(move || {
        while running.load(Ordering::Relaxed) {
            if let Some(segment) = sniffer.next() {
                network_utilization.lock().unwrap().update(&segment)
            }
        }
    });
    display_handler.join().unwrap();
    sniffing_handler.join().unwrap();
    stdin_handler.join().unwrap();
    dns_handler.join().unwrap();
}
