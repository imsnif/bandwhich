#![deny(clippy::all)]

mod display;
mod network;
mod os;
#[cfg(test)]
mod tests;

use display::{RawTerminalBackend, Ui};
use network::{
    dns::{self, IpTable},
    Connection, LocalSocket, Sniffer, Utilization,
};
use os::OnSigWinch;

use ::pnet_bandwhich_fork::datalink::{DataLinkReceiver, NetworkInterface};
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

const DISPLAY_DELTA: time::Duration = time::Duration::from_millis(1000);

#[derive(StructOpt, Debug)]
#[structopt(name = "bandwhich")]
pub struct Opt {
    #[structopt(short, long)]
    /// The network interface to listen on, eg. eth0
    interface: Option<String>,
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
    #[cfg(target_os = "windows")]
    compile_error!("Sorry, no implementations for Windows yet :( - PRs welcome!");

    use os::get_input;
    let opts = Opt::from_args();
    let os_input = get_input(&opts.interface, !opts.no_resolve)?;
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
                "Failed to get stdout: if you are trying to pipe 'bandwhich' you should use the --raw flag"
            ),
        }
    }
    Ok(())
}

pub struct OpenSockets {
    sockets_to_procs: HashMap<LocalSocket, String>,
    connections: Vec<Connection>,
}

pub struct OsInputOutput {
    pub network_interfaces: Vec<NetworkInterface>,
    pub network_frames: Vec<Box<dyn DataLinkReceiver>>,
    pub get_open_sockets: fn() -> OpenSockets,
    pub keyboard_events: Box<dyn Iterator<Item = Event> + Send>,
    pub dns_client: Option<dns::Client>,
    pub on_winch: Box<OnSigWinch>,
    pub cleanup: Box<dyn Fn() + Send>,
    pub write_to_stdout: Box<dyn FnMut(String) + Send>,
}

pub fn start<B>(terminal_backend: B, os_input: OsInputOutput, opts: Opt)
where
    B: Backend + Send + 'static,
{
    let running = Arc::new(AtomicBool::new(true));
    let paused = Arc::new(AtomicBool::new(false));

    let mut active_threads = vec![];

    let keyboard_events = os_input.keyboard_events;
    let get_open_sockets = os_input.get_open_sockets;
    let mut write_to_stdout = os_input.write_to_stdout;
    let mut dns_client = os_input.dns_client;
    let on_winch = os_input.on_winch;
    let cleanup = os_input.cleanup;

    let raw_mode = opts.raw;

    let network_utilization = Arc::new(Mutex::new(Utilization::new()));
    let ui = Arc::new(Mutex::new(Ui::new(terminal_backend, opts.interface)));

    if !raw_mode {
        active_threads.push(
            thread::Builder::new()
                .name("resize_handler".to_string())
                .spawn({
                    let ui = ui.clone();
                    let paused = paused.clone();
                    move || {
                        on_winch({
                            Box::new(move || {
                                let mut ui = ui.lock().unwrap();
                                ui.draw(paused.load(Ordering::SeqCst));
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
            let paused = paused.clone();
            let network_utilization = network_utilization.clone();
            move || {
                while running.load(Ordering::Acquire) {
                    let render_start_time = Instant::now();
                    let utilization = { network_utilization.lock().unwrap().clone_and_reset() };
                    let OpenSockets {
                        sockets_to_procs,
                        connections,
                    } = get_open_sockets();
                    let mut ip_to_host = IpTable::new();
                    if let Some(dns_client) = dns_client.as_mut() {
                        ip_to_host = dns_client.cache();
                        let unresolved_ips = connections
                            .iter()
                            .filter(|conn| !ip_to_host.contains_key(&conn.remote_socket.ip))
                            .map(|conn| conn.remote_socket.ip)
                            .collect::<Vec<_>>();

                        dns_client.resolve(unresolved_ips);
                    }
                    {
                        let mut ui = ui.lock().unwrap();
                        let paused = paused.load(Ordering::SeqCst);
                        if !paused {
                            ui.update_state(sockets_to_procs, utilization, ip_to_host);
                        }
                        if raw_mode {
                            ui.output_text(&mut write_to_stdout);
                        } else {
                            ui.draw(paused);
                        }
                    }
                    let render_duration = render_start_time.elapsed();
                    if render_duration < DISPLAY_DELTA {
                        park_timeout(DISPLAY_DELTA - render_duration);
                    }
                }
                if !raw_mode {
                    let mut ui = ui.lock().unwrap();
                    ui.end();
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
                            Event::Key(Key::Char(' ')) => {
                                paused.fetch_xor(true, Ordering::SeqCst);
                                display_handler.unpark();
                            }
                            _ => (),
                        };
                    }
                }
            })
            .unwrap(),
    );
    active_threads.push(display_handler);

    let sniffer_threads = os_input
        .network_interfaces
        .into_iter()
        .zip(os_input.network_frames.into_iter())
        .map(|(iface, frames)| {
            let name = format!("sniffing_handler_{}", iface.name);
            let running = running.clone();
            let network_utilization = network_utilization.clone();

            thread::Builder::new()
                .name(name)
                .spawn(move || {
                    let mut sniffer = Sniffer::new(iface, frames);

                    while running.load(Ordering::Acquire) {
                        if let Some(segment) = sniffer.next() {
                            network_utilization.lock().unwrap().update(segment);
                        }
                    }
                })
                .unwrap()
        })
        .collect::<Vec<_>>();
    active_threads.extend(sniffer_threads);

    for thread_handler in active_threads {
        thread_handler.join().unwrap()
    }
}
