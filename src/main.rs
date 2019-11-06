mod display;
mod network;
mod os;
#[cfg(test)]
mod tests;

use display::{RawTerminalBackend, Ui};
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
use ::std::time::Instant;
use ::termion::raw::IntoRawMode;
use ::tui::backend::TermionBackend;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "what")]
pub struct Opt {
    #[structopt(short, long)]
    /// The network interface to listen on, eg. eth0
    interface: String,
    #[structopt(short, long)]
    /// Machine friendlier output
    raw: bool,
    #[structopt(short, long)]
    /// Do not attempt to resolve IPs to their hostnames
    no_resolve: bool,
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

    use os::get_input;
    let opts = Opt::from_args();
    let os_input = get_input(&opts.interface)?;
    let raw_mode = opts.raw;
    if raw_mode {
        let terminal_backend = RawTerminalBackend {};
        start(terminal_backend, os_input, opts);
    } else {
        match io::stdout().into_raw_mode() {
            Ok(stdout) => {
                let terminal_backend = TermionBackend::new(stdout);
                start(terminal_backend, os_input, opts);
            }
            Err(_) => failure::bail!(
                "Failed to get stdout: 'what' does not (yet) support piping, is it being piped?"
            ),
        }
    };
    Ok(())
}

pub struct OsInputOutput {
    pub network_interface: NetworkInterface,
    pub network_frames: Box<dyn DataLinkReceiver>,
    pub get_open_sockets: fn() -> HashMap<Connection, String>,
    pub keyboard_events: Box<dyn Iterator<Item = Event> + Send>,
    pub lookup_addr: Box<dyn Fn(&IpAddr) -> Option<String> + Send>,
    pub on_winch: Box<dyn Fn(Box<dyn Fn()>) + Send>,
    pub cleanup: Box<dyn Fn() + Send>,
    pub write_to_stdout: Box<dyn FnMut(String) + Send>,
}

pub fn start<'a, B>(terminal_backend: B, os_input: OsInputOutput, opts: Opt)
where
    B: Backend + Send + 'static,
{
    let running = Arc::new(AtomicBool::new(true));

    let mut active_threads = vec![];

    let keyboard_events = os_input.keyboard_events;
    let get_open_sockets = os_input.get_open_sockets;
    let lookup_addr = os_input.lookup_addr;
    let mut write_to_stdout = os_input.write_to_stdout;
    let on_winch = os_input.on_winch;
    let cleanup = os_input.cleanup;

    let raw_mode = opts.raw;
    let no_resolve = opts.no_resolve;

    let mut sniffer = Sniffer::new(os_input.network_interface, os_input.network_frames);
    let network_utilization = Arc::new(Mutex::new(Utilization::new()));
    let ui = Arc::new(Mutex::new(Ui::new(terminal_backend)));

    let dns_queue = if no_resolve {
        Arc::new(None)
    } else {
        Arc::new(Some(DnsQueue::new()))
    };
    let ip_to_host = Arc::new(Mutex::new(HashMap::new()));

    if !no_resolve {
        active_threads.push(
            thread::Builder::new()
                .name("dns_resolver".to_string())
                .spawn({
                    let dns_queue = dns_queue.clone();
                    let ip_to_host = ip_to_host.clone();
                    move || {
                        if let Some(dns_queue) = Option::as_ref(&dns_queue) {
                            while let Some(ip) = dns_queue.wait_for_job() {
                                if let Some(addr) = lookup_addr(&IpAddr::V4(ip)) {
                                    ip_to_host.lock().unwrap().insert(ip, addr);
                                }
                            }
                        }
                    }
                })
                .unwrap(),
        );
    }

    if !raw_mode {
        active_threads.push(
            thread::Builder::new()
                .name("resize_handler".to_string())
                .spawn({
                    let ui = ui.clone();
                    move || {
                        on_winch({
                            Box::new(move || {
                                let mut ui = ui.lock().unwrap();
                                ui.draw();
                            })
                        });
                    }
                })
                .unwrap(),
        );
    }

    let display_handler = thread::Builder::new()
        .name("display_handler".to_string())
        .spawn({
            let running = running.clone();
            let network_utilization = network_utilization.clone();
            let ip_to_host = ip_to_host.clone();
            let dns_queue = dns_queue.clone();
            let ui = ui.clone();
            move || {
                while running.load(Ordering::Acquire) {
                    let render_start_time = Instant::now();
                    let utilization = { network_utilization.lock().unwrap().clone_and_reset() };
                    let connections_to_procs = get_open_sockets();
                    let ip_to_host = { ip_to_host.lock().unwrap().clone() };
                    let mut unresolved_ips = Vec::new();
                    for connection in connections_to_procs.keys() {
                        if !ip_to_host.contains_key(&connection.remote_socket.ip) {
                            unresolved_ips.push(connection.remote_socket.ip);
                        }
                    }
                    if let Some(dns_queue) = Option::as_ref(&dns_queue) {
                        if !unresolved_ips.is_empty() {
                            dns_queue.resolve_ips(unresolved_ips);
                        }
                    }
                    {
                        let mut ui = ui.lock().unwrap();
                        ui.update_state(connections_to_procs, utilization, ip_to_host);
                        if raw_mode {
                            ui.output_text(&mut write_to_stdout);
                        } else {
                            ui.draw();
                        }
                    }
                    let render_duration = render_start_time.elapsed();
                    park_timeout(time::Duration::from_millis(1000) - render_duration);
                }
                if !raw_mode {
                    let mut ui = ui.lock().unwrap();
                    ui.end();
                }
                if let Some(dns_queue) = Option::as_ref(&dns_queue) {
                    dns_queue.end();
                }
            }
        })
        .unwrap();

    active_threads.push(
        thread::Builder::new()
            .name("stdin_handler".to_string())
            .spawn({
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
            })
            .unwrap(),
    );
    active_threads.push(display_handler);

    active_threads.push(
        thread::Builder::new()
            .name("sniffing_handler".to_string())
            .spawn(move || {
                while running.load(Ordering::Acquire) {
                    if let Some(segment) = sniffer.next() {
                        network_utilization.lock().unwrap().update(&segment)
                    }
                }
            })
            .unwrap(),
    );
    for thread_handler in active_threads {
        thread_handler.join().unwrap()
    }
}
