mod traffic;
mod store;
mod display;

use traffic::Sniffer;
use display::display_loop;
use store::{CurrentConnections, NetworkUtilization};

use ::std::{thread, time};
use ::std::sync::{Arc, Mutex};
use ::std::sync::atomic::{AtomicBool, Ordering};
use ::tui::Terminal;
use ::tui::backend::Backend;
use ::termion::event::{Key, Event};
use ::pnet::datalink::{DataLinkReceiver, NetworkInterface};
use ::netstat::SocketInfo;

pub struct OsInput {
    pub network_interface: NetworkInterface,
    pub network_frames: Box<DataLinkReceiver>,
    pub get_process_name: fn(i32) -> Option<String>,
    pub get_open_sockets: fn() -> Vec<SocketInfo>,
    pub keyboard_events: Box<Iterator<Item = Event> + Send + Sync + 'static>
}

pub fn start <B> (terminal_backend: B, os_input: OsInput)
    where B: Backend + Send + 'static
{
    let r = Arc::new(AtomicBool::new(true));
    let displaying = r.clone();
    let running = r.clone();

    let keyboard_events = os_input.keyboard_events; // TODO: as methods in os_interface
    let get_process_name = os_input.get_process_name;
    let get_open_sockets = os_input.get_open_sockets;

    let stdin_handler = thread::spawn(move || {
        for evt in keyboard_events{
            match evt {
                Event::Key(Key::Ctrl('c')) => {
                    // TODO: exit faster
                    r.store(false, Ordering::SeqCst);
                    break
                },
                Event::Key(Key::Char('q')) => {
                    r.store(false, Ordering::SeqCst);
                    break
                },
                _ => ()
            };
        };
    });

    let mut sniffer = Sniffer::new(os_input.network_interface, os_input.network_frames);
    let network_utilization = Arc::new(Mutex::new(NetworkUtilization::new()));

    let display_handler = thread::spawn({
        let network_utilization = network_utilization.clone();
        move || {
            let mut terminal = Terminal::new(terminal_backend).unwrap();
            terminal.clear().unwrap();
            terminal.hide_cursor().unwrap();
            while displaying.load(Ordering::SeqCst) {
                let current_connections = CurrentConnections::new(&get_process_name, &get_open_sockets);
                display_loop(&network_utilization, &mut terminal, current_connections);
                thread::sleep(time::Duration::from_secs(1));
            }
            terminal.clear().unwrap();
            terminal.show_cursor().unwrap();
        }
    });

    let sniffing_handler = thread::spawn(move || {
        while running.load(Ordering::SeqCst) {
            match sniffer.next() {
                Some(segment) => {
                    network_utilization.lock().unwrap().update(&segment)
                },
                None => ()
            }
        }
    });
    display_handler.join().unwrap();
    sniffing_handler.join().unwrap();
    stdin_handler.join().unwrap();
}
