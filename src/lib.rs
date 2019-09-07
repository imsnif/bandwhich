#![ allow( dead_code, unused_imports ) ]
mod traffic;
mod store; // TODO: change name to network_utilization
mod current_connections;
pub mod display;

use ::pnet::datalink::{self, NetworkInterface};
use ::pnet::datalink::{DataLinkReceiver, Channel};

use ::human_size::{Size};
use ::std::{thread, time};
use ::std::sync::{Arc, Mutex};
use ::std::sync::atomic::{AtomicBool, Ordering};
use ::std::collections::HashMap;

use traffic::{Sniffer, Segment, Connection};
use store::{NetworkUtilization, ConnectionData};
use display::{IsProcess, display_loop};

use ::num_bigint::{BigUint, ToBigUint, ParseBigIntError};
use ::num_traits::{Zero, One};
use ::num_traits::cast::{ToPrimitive};

use ::std::io;
use ::std::io::{Write, stdout, stdin, Stdout};
use ::tui::Terminal;
use ::tui::backend::{Backend, TermionBackend};
use ::tui::widgets::{Widget, Block, Borders, Table, Row};
use ::tui::layout::{Layout, Constraint, Direction, Rect};
use ::tui::style::{Style, Color};
use ::tui::buffer::Cell;
use ::tui::style;
use ::termion::raw::IntoRawMode;
use ::termion::event::{Key, Event, MouseEvent};
use ::termion::input::{TermRead};

use ::std::fmt::Debug;
use ::netstat::*;

pub fn start <P, Q, B, T, S, Z> (backend: B, create_process: &'static P, get_sockets_info: &'static Q, interface: NetworkInterface, channel: Box<DataLinkReceiver>, stdin_events: S, ) where
    P: Fn(i32) -> Result<T, Box<std::error::Error>>+std::marker::Sync,
    B: Backend + Send + 'static,
    T: IsProcess + Send + Sync + Debug + 'static,
    S: Iterator<Item = Event> + Send + Sync + 'static,
    Q: Fn(AddressFamilyFlags, ProtocolFlags) -> Result<Vec<SocketInfo>, Z>+std::marker::Sync,
    Z: std::fmt::Debug
{
    let r = Arc::new(AtomicBool::new(true));
    let displaying = r.clone();
    let running = r.clone();
    let stdin_handler = thread::spawn(move || {
        for evt in stdin_events {
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

    let mut sniffer = Sniffer::new(interface, channel);
    let network_utilization = Arc::new(Mutex::new(NetworkUtilization::new()));
    let mirror_utilization = Arc::clone(&network_utilization); // TODO: better name
    let display_handler = thread::spawn(move || {
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.clear().unwrap();
        terminal.hide_cursor().unwrap();
        while displaying.load(Ordering::SeqCst) {
            display_loop(&mirror_utilization, &mut terminal, &create_process, &get_sockets_info);
            thread::sleep(time::Duration::from_secs(1));
        }
        terminal.clear().unwrap();
        terminal.show_cursor().unwrap();
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
