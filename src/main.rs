mod display;
mod network;
mod os;
#[cfg(test)]
mod tests;

use display::Ui;
use network::{Connection, DnsQueue, Sniffer, Utilization};

use ::std::net::IpAddr;

use ::pnet::datalink::{DataLinkReceiver, NetworkInterface};
use ::std::collections::HashMap;
use ::std::sync::atomic::{AtomicBool, Ordering};
use ::std::sync::{Arc, Mutex};
use ::std::thread::park_timeout;
use ::std::{thread, time};
use ::termion::event::{Event, Key};
use ::tui::backend::Backend;

use std::process;

use ::std::io;
use ::termion::raw::IntoRawMode;
use ::tui::backend::TermionBackend;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "what")]
pub struct Opt {
    #[structopt(short, long)]
    interface: String,
}

fn main() {
    if let Err(err) = try_main() {
        eprintln!("Error: {}", err);
        process::exit(2);
    }
}

fn try_main() -> Result<(), failure::Error> {
    #[cfg(not(target_os = "linux"))]
    compile_error!(
        "Sorry, no implementations for platforms other than linux yet :( - PRs welcome!"
    );

    use os::{get_input};
    let opt = Opt::from_args();
    let os_input = get_input(opt)?;
    let stdout = match io::stdout().into_raw_mode() {
        Ok(stdout) => stdout,
        Err(_) => failure::bail!(
            "Failed to get stdout: 'what' does not (yet) support piping, is it being piped?"
        ),
    };
    let terminal_backend = TermionBackend::new(stdout);
    start(terminal_backend, os_input);
    Ok(())
}

pub struct OsInput {
    pub network_interface: NetworkInterface,
    pub network_frames: Box<DataLinkReceiver>,
    pub get_open_sockets: fn() -> HashMap<Connection, String>,
    pub keyboard_events: Box<Iterator<Item = Event> + Send>,
    pub lookup_addr: Box<Fn(&IpAddr) -> Option<String> + Send>,
    pub on_winch: Box<Fn(Box<Fn()>)+ Send>,
    pub cleanup: Box<Fn() + Send>,
}

pub fn start<B>(terminal_backend: B, os_input: OsInput)
where
    B: Backend + Send + 'static,
{
    let running = Arc::new(AtomicBool::new(true));

    let keyboard_events = os_input.keyboard_events;
    let get_open_sockets = os_input.get_open_sockets;
    let lookup_addr = os_input.lookup_addr;
    let on_winch = os_input.on_winch;
    let cleanup = os_input.cleanup;

    let mut sniffer = Sniffer::new(os_input.network_interface, os_input.network_frames);
    let network_utilization = Arc::new(Mutex::new(Utilization::new()));
    let ui = Arc::new(Mutex::new(Ui::new(terminal_backend)));
    let dns_queue = Arc::new(DnsQueue::new());
    let ip_to_host = Arc::new(Mutex::new(HashMap::new()));

    let dns_handler = thread::spawn({
        let dns_queue = dns_queue.clone();
        let ip_to_host = ip_to_host.clone();
        move || {
            while let Some(ip) = dns_queue.wait_for_job() {
                if let Some(addr) = lookup_addr(&IpAddr::V4(ip)) {
                    ip_to_host.lock().unwrap().insert(ip, addr);
                }
            }
        }
    });

    let resize_handler = thread::spawn({
        let ui = ui.clone();
        move || {
            on_winch({
                Box::new(move || {
                    let mut ui = ui.lock().unwrap();
                    ui.draw();
                })
            });
        }
    });

    let display_handler = thread::spawn({
        let running = running.clone();
        let network_utilization = network_utilization.clone();
        let ip_to_host = ip_to_host.clone();
        let dns_queue = dns_queue.clone();
        let ui = ui.clone();
        move || {
            while running.load(Ordering::Acquire) {
                let connections_to_procs = get_open_sockets();
                let ip_to_host = { ip_to_host.lock().unwrap().clone() };
                let utilization = {
                    let mut network_utilization = network_utilization.lock().unwrap();
                    network_utilization.clone_and_reset()
                };
                dns_queue.find_ips_to_resolve(&connections_to_procs, &ip_to_host);
                {
                    let mut ui = ui.lock().unwrap();
                    ui.update_state(connections_to_procs, utilization, ip_to_host);
                    ui.draw();
                }
                park_timeout(time::Duration::from_secs(1));
            }
            let mut ui = ui.lock().unwrap();
            ui.end();
            dns_queue.end();
        }
    });

    let stdin_handler = thread::spawn({
        let running = running.clone();
        let display_handler = display_handler.thread().clone();
        move || {
            for evt in keyboard_events {
                match evt {
                    Event::Key(Key::Ctrl('c')) | Event::Key(Key::Char('q')) => {
                        running.store(false, Ordering::Release);
                        cleanup();
                        display_handler.unpark();
                        break;
                    }
                    _ => (),
                };
            }
        }
    });

    let sniffing_handler = thread::spawn(move || {
        while running.load(Ordering::Acquire) {
            if let Some(segment) = sniffer.next() {
                network_utilization.lock().unwrap().update(&segment)
            }
        }
    });
    display_handler.join().unwrap();
    sniffing_handler.join().unwrap();
    stdin_handler.join().unwrap();
    dns_handler.join().unwrap();
    resize_handler.join().unwrap();
}
