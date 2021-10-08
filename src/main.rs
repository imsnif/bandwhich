#![deny(clippy::all)]

mod display;
mod network;
mod os;
#[cfg(test)]
mod tests;

use display::{elapsed_time, RawTerminalBackend, Ui};
use network::{
    dns::{self, IpTable},
    LocalSocket, Sniffer, Utilization,
};

use ::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ::crossterm::terminal;
use ::pnet::datalink::{DataLinkReceiver, NetworkInterface};
use ::std::collections::HashMap;
use ::std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use ::std::sync::{Arc, Mutex};
use ::std::thread;
use ::std::thread::park_timeout;
use ::tui::backend::Backend;

use std::process;

use ::std::net::Ipv4Addr;
use ::std::time::{Duration, Instant};
use ::tui::backend::CrosstermBackend;
use std::sync::RwLock;
use structopt::StructOpt;

const DISPLAY_DELTA: Duration = Duration::from_millis(1000);

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
    #[structopt(flatten)]
    render_opts: RenderOpts,
    #[structopt(short, long)]
    /// Show DNS queries
    show_dns: bool,
    #[structopt(short, long)]
    /// A dns server ip to use instead of the system default
    dns_server: Option<Ipv4Addr>,
    #[cfg(target_os = "macos")]
    #[structopt(short = "o", long)]
    /// Remove default payload offset for Point-to-Point interfaces
    no_payload_offset: bool,
}

#[derive(StructOpt, Debug, Copy, Clone)]
pub struct RenderOpts {
    #[structopt(short, long)]
    /// Show processes table only
    processes: bool,
    #[structopt(short, long)]
    /// Show connections table only
    connections: bool,
    #[structopt(short, long)]
    /// Show remote addresses table only
    addresses: bool,
    #[structopt(short, long)]
    /// Show total (cumulative) usages
    total_utilization: bool,
}

fn main() {
    if let Err(err) = try_main() {
        eprintln!("Error: {}", err);
        process::exit(2);
    }
}

fn try_main() -> Result<(), failure::Error> {
    use os::get_input;
    let opts = Opt::from_args();
    let os_input = get_input(&opts.interface, !opts.no_resolve, &opts.dns_server)?;
    let raw_mode = opts.raw;
    if raw_mode {
        let terminal_backend = RawTerminalBackend {};
        start(terminal_backend, os_input, opts);
    } else {
        match terminal::enable_raw_mode() {
            Ok(()) => {
                let stdout = std::io::stdout();
                let terminal_backend = CrosstermBackend::new(stdout);
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
}

pub struct OsInputOutput {
    pub network_interfaces: Vec<NetworkInterface>,
    pub network_frames: Vec<Box<dyn DataLinkReceiver>>,
    pub get_open_sockets: fn() -> OpenSockets,
    pub terminal_events: Box<dyn Iterator<Item = Event> + Send>,
    pub dns_client: Option<dns::Client>,
    pub write_to_stdout: Box<dyn FnMut(String) + Send>,
}

pub fn start<B>(terminal_backend: B, os_input: OsInputOutput, opts: Opt)
where
    B: Backend + Send + 'static,
{
    let running = Arc::new(AtomicBool::new(true));
    let paused = Arc::new(AtomicBool::new(false));
    let last_start_time = Arc::new(RwLock::new(Instant::now()));
    let cumulative_time = Arc::new(RwLock::new(Duration::new(0, 0)));
    let ui_offset = Arc::new(AtomicUsize::new(0));
    let dns_shown = opts.show_dns;

    let mut active_threads = vec![];

    let terminal_events = os_input.terminal_events;
    let get_open_sockets = os_input.get_open_sockets;
    let mut write_to_stdout = os_input.write_to_stdout;
    let mut dns_client = os_input.dns_client;

    let raw_mode = opts.raw;

    let network_utilization = Arc::new(Mutex::new(Utilization::new()));
    let ui = Arc::new(Mutex::new(Ui::new(terminal_backend, opts.render_opts)));

    let display_handler = thread::Builder::new()
        .name("display_handler".to_string())
        .spawn({
            let running = running.clone();
            let paused = paused.clone();
            let ui_offset = ui_offset.clone();

            let network_utilization = network_utilization.clone();
            let last_start_time = last_start_time.clone();
            let cumulative_time = cumulative_time.clone();
            let ui = ui.clone();

            move || {
                while running.load(Ordering::Acquire) {
                    let render_start_time = Instant::now();
                    let utilization = { network_utilization.lock().unwrap().clone_and_reset() };
                    let OpenSockets { sockets_to_procs } = get_open_sockets();
                    let mut ip_to_host = IpTable::new();
                    if let Some(dns_client) = dns_client.as_mut() {
                        ip_to_host = dns_client.cache();
                        let unresolved_ips = utilization
                            .connections
                            .keys()
                            .filter(|conn| !ip_to_host.contains_key(&conn.remote_socket.ip))
                            .map(|conn| conn.remote_socket.ip)
                            .collect::<Vec<_>>();
                        dns_client.resolve(unresolved_ips);
                    }
                    {
                        let mut ui = ui.lock().unwrap();
                        let paused = paused.load(Ordering::SeqCst);
                        let ui_offset = ui_offset.load(Ordering::SeqCst);
                        if !paused {
                            ui.update_state(sockets_to_procs, utilization, ip_to_host);
                        }
                        let elapsed_time = elapsed_time(
                            *last_start_time.read().unwrap(),
                            *cumulative_time.read().unwrap(),
                            paused,
                        );

                        if raw_mode {
                            ui.output_text(&mut write_to_stdout);
                        } else {
                            ui.draw(paused, dns_shown, elapsed_time, ui_offset);
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
            .name("terminal_events_handler".to_string())
            .spawn({
                let running = running.clone();
                let display_handler = display_handler.thread().clone();

                move || {
                    for evt in terminal_events {
                        let mut ui = ui.lock().unwrap();

                        match evt {
                            Event::Resize(_x, _y) => {
                                if !raw_mode {
                                    let paused = paused.load(Ordering::SeqCst);
                                    ui.draw(
                                        paused,
                                        dns_shown,
                                        elapsed_time(
                                            *last_start_time.read().unwrap(),
                                            *cumulative_time.read().unwrap(),
                                            paused,
                                        ),
                                        ui_offset.load(Ordering::SeqCst),
                                    );
                                };
                            }
                            Event::Key(KeyEvent {
                                modifiers: KeyModifiers::CONTROL,
                                code: KeyCode::Char('c'),
                            })
                            | Event::Key(KeyEvent {
                                modifiers: KeyModifiers::NONE,
                                code: KeyCode::Char('q'),
                            }) => {
                                running.store(false, Ordering::Release);
                                display_handler.unpark();
                                match terminal::disable_raw_mode() {
                                    Ok(_) => {}
                                    Err(_) => println!("Error could not disable raw input"),
                                }
                                break;
                            }
                            Event::Key(KeyEvent {
                                modifiers: KeyModifiers::NONE,
                                code: KeyCode::Char(' '),
                            }) => {
                                let restarting = paused.fetch_xor(true, Ordering::SeqCst);
                                if restarting {
                                    *last_start_time.write().unwrap() = Instant::now();
                                } else {
                                    let last_start_time_copy = *last_start_time.read().unwrap();
                                    let current_cumulative_time_copy =
                                        *cumulative_time.read().unwrap();
                                    let new_cumulative_time = current_cumulative_time_copy
                                        + last_start_time_copy.elapsed();
                                    *cumulative_time.write().unwrap() = new_cumulative_time;
                                }

                                display_handler.unpark();
                            }
                            Event::Key(KeyEvent {
                                modifiers: KeyModifiers::NONE,
                                code: KeyCode::Tab,
                            }) => {
                                let paused = paused.load(Ordering::SeqCst);
                                let elapsed_time = elapsed_time(
                                    *last_start_time.read().unwrap(),
                                    *cumulative_time.read().unwrap(),
                                    paused,
                                );
                                let table_count = ui.get_table_count();
                                let new = ui_offset.load(Ordering::SeqCst) + 1 % table_count;
                                ui_offset.store(new, Ordering::SeqCst);
                                ui.draw(paused, dns_shown, elapsed_time, new);
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
            let show_dns = opts.show_dns;
            #[cfg(target_os = "macos")]
            let with_payload_offset = !opts.no_payload_offset;
            #[cfg(not(target_os = "macos"))]
            let with_payload_offset = false;
            let network_utilization = network_utilization.clone();

            thread::Builder::new()
                .name(name)
                .spawn(move || {
                    let mut sniffer = Sniffer::new(iface, frames, show_dns, with_payload_offset);

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
