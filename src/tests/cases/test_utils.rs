use crate::tests::fakes::{
    create_fake_dns_client, get_interfaces, get_open_sockets, NetworkFrames, TerminalEvent,
    TerminalEvents, TestBackend,
};
use std::iter;

use crate::network::dns::Client;
use crate::{Opt, OsInputOutput, RenderOpts};
use ::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use packet_builder::*;
use pnet::datalink::DataLinkReceiver;
use std::collections::HashMap;
use std::io::Write;
use std::sync::{Arc, Mutex};

use pnet::packet::Packet;
use pnet_base::MacAddr;

pub fn sleep_and_quit_events(sleep_num: usize) -> Box<TerminalEvents> {
    let mut events: Vec<Option<Event>> = iter::repeat(None).take(sleep_num).collect();
    events.push(Some(Event::Key(KeyEvent::new(
        KeyCode::Char('c'),
        KeyModifiers::CONTROL,
    ))));
    Box::new(TerminalEvents::new(events))
}

pub fn sleep_resize_and_quit_events(sleep_num: usize) -> Box<TerminalEvents> {
    let mut events: Vec<Option<Event>> = iter::repeat(None).take(sleep_num).collect();
    events.push(Some(Event::Resize(100, 100)));
    events.push(Some(Event::Key(KeyEvent::new(
        KeyCode::Char('c'),
        KeyModifiers::CONTROL,
    ))));
    Box::new(TerminalEvents::new(events))
}

pub fn build_tcp_packet(
    source_ip: &str,
    destination_ip: &str,
    source_port: u16,
    destination_port: u16,
    payload: &'static [u8],
) -> Vec<u8> {
    let mut pkt_buf = [0u8; 1500];
    let pkt = packet_builder!(
         pkt_buf,
         ether({set_destination => MacAddr(0,0,0,0,0,0), set_source => MacAddr(0,0,0,0,0,0)}) /
         ipv4({set_source => ipv4addr!(source_ip), set_destination => ipv4addr!(destination_ip) }) /
         tcp({set_source => source_port, set_destination => destination_port }) /
         payload(payload)
    );
    pkt.packet().to_vec()
}

pub fn sample_frames() -> Vec<Box<dyn DataLinkReceiver>> {
    vec![NetworkFrames::new(vec![
        Some(build_tcp_packet(
            "10.0.0.2",
            "1.1.1.1",
            443,
            12345,
            b"I am a fake tcp upload packet",
        )),
        Some(build_tcp_packet(
            "1.1.1.1",
            "10.0.0.2",
            12345,
            443,
            b"I am a fake tcp download packet",
        )),
        Some(build_tcp_packet(
            "10.0.0.2",
            "1.1.1.1",
            54321,
            53,
            b"I am a fake DNS query packet",
        )),
    ]) as Box<dyn DataLinkReceiver>]
}

pub fn os_input_output(
    network_frames: Vec<Box<dyn DataLinkReceiver>>,
    sleep_num: usize,
) -> OsInputOutput {
    os_input_output_factory(
        network_frames,
        None,
        create_fake_dns_client(HashMap::new()),
        sleep_and_quit_events(sleep_num),
    )
}
pub fn os_input_output_stdout(
    network_frames: Vec<Box<dyn DataLinkReceiver>>,
    sleep_num: usize,
    stdout: Option<Arc<Mutex<Vec<u8>>>>,
) -> OsInputOutput {
    os_input_output_factory(
        network_frames,
        stdout,
        create_fake_dns_client(HashMap::new()),
        sleep_and_quit_events(sleep_num),
    )
}

pub fn os_input_output_dns(
    network_frames: Vec<Box<dyn DataLinkReceiver>>,
    sleep_num: usize,
    stdout: Option<Arc<Mutex<Vec<u8>>>>,
    dns_client: Option<Client>,
) -> OsInputOutput {
    os_input_output_factory(
        network_frames,
        stdout,
        dns_client,
        sleep_and_quit_events(sleep_num),
    )
}

pub fn os_input_output_factory(
    network_frames: Vec<Box<dyn DataLinkReceiver>>,
    stdout: Option<Arc<Mutex<Vec<u8>>>>,
    dns_client: Option<Client>,
    keyboard_events: Box<dyn Iterator<Item = Event> + Send>,
) -> OsInputOutput {
    let write_to_stdout: Box<dyn FnMut(String) + Send> = match stdout {
        Some(stdout) => Box::new({
            move |output: String| {
                let mut stdout = stdout.lock().unwrap();
                writeln!(&mut stdout, "{}", output).unwrap();
            }
        }),
        None => Box::new(move |_output: String| {}),
    };

    OsInputOutput {
        network_interfaces: get_interfaces(),
        network_frames,
        get_open_sockets,
        terminal_events: keyboard_events,
        dns_client,
        write_to_stdout,
    }
}

pub fn opts_raw() -> Opt {
    opts_factory(true)
}

pub fn opts_ui() -> Opt {
    opts_factory(false)
}

fn opts_factory(raw: bool) -> Opt {
    Opt {
        interface: Some(String::from("interface_name")),
        raw,
        no_resolve: false,
        show_dns: false,
        dns_server: None,
        render_opts: RenderOpts {
            addresses: false,
            connections: false,
            processes: false,
            total_utilization: false,
        },
    }
}
type BackendWithStreams = (
    Arc<Mutex<Vec<TerminalEvent>>>,
    Arc<Mutex<Vec<String>>>,
    TestBackend,
);
pub fn test_backend_factory(w: u16, h: u16) -> BackendWithStreams {
    let terminal_events: Arc<Mutex<Vec<TerminalEvent>>> = Arc::new(Mutex::new(Vec::new()));
    let terminal_draw_events: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    let backend = TestBackend::new(
        terminal_events.clone(),
        terminal_draw_events.clone(),
        Arc::new(Mutex::new(w)),
        Arc::new(Mutex::new(h)),
    );
    (terminal_events, terminal_draw_events, backend)
}
