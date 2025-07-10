#![cfg_attr(not(feature = "ui_test"), allow(dead_code))]

use std::{
    collections::HashMap,
    io::Write,
    iter,
    sync::{Arc, Mutex},
};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use packet_builder::*;
use pnet::{datalink::DataLinkReceiver, packet::Packet};
use pnet_base::MacAddr;
use rstest::fixture;

use crate::{
    network::dns::Client,
    tests::fakes::{
        create_fake_dns_client, get_interfaces_with_frames, get_open_sockets, NetworkFrames,
        TerminalEvent, TerminalEvents, TestBackend,
    },
    Opt, OsInputOutput,
};

pub fn sleep_and_quit_events(sleep_num: usize) -> Box<TerminalEvents> {
    let events = iter::repeat_n(None, sleep_num)
        .chain([Some(Event::Key(KeyEvent::new(
            KeyCode::Char('c'),
            KeyModifiers::CONTROL,
        )))])
        .collect();
    Box::new(TerminalEvents::new(events))
}

pub fn sleep_resize_and_quit_events(sleep_num: usize) -> Box<TerminalEvents> {
    let events = iter::repeat_n(None, sleep_num)
        .chain([
            Some(Event::Resize(100, 100)),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Char('c'),
                KeyModifiers::CONTROL,
            ))),
        ])
        .collect();
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

#[fixture]
pub fn sample_frames_short() -> Vec<Box<dyn DataLinkReceiver>> {
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

#[fixture]
pub fn sample_frames_sustained_one_process() -> Vec<Box<dyn DataLinkReceiver>> {
    vec![NetworkFrames::new(vec![
        Some(build_tcp_packet(
            "1.1.1.1",
            "10.0.0.2",
            12345,
            443,
            b"I have come from 1.1.1.1",
        )),
        None, // sleep
        Some(build_tcp_packet(
            "1.1.1.1",
            "10.0.0.2",
            12345,
            443,
            b"Same here, but one second later",
        )),
    ]) as Box<dyn DataLinkReceiver>]
}

#[fixture]
pub fn sample_frames_sustained_multiple_processes() -> Vec<Box<dyn DataLinkReceiver>> {
    vec![NetworkFrames::new(vec![
        Some(build_tcp_packet(
            "1.1.1.1",
            "10.0.0.2",
            12345,
            443,
            b"I have come from 1.1.1.1",
        )),
        Some(build_tcp_packet(
            "3.3.3.3",
            "10.0.0.2",
            1337,
            4435,
            b"I come from 3.3.3.3",
        )),
        None, // sleep
        Some(build_tcp_packet(
            "1.1.1.1",
            "10.0.0.2",
            12345,
            443,
            b"I have come from 1.1.1.1 one second later",
        )),
        Some(build_tcp_packet(
            "3.3.3.3",
            "10.0.0.2",
            1337,
            4435,
            b"I come 3.3.3.3 one second later",
        )),
    ]) as Box<dyn DataLinkReceiver>]
}

#[fixture]
pub fn sample_frames_sustained_long() -> Vec<Box<dyn DataLinkReceiver>> {
    vec![NetworkFrames::new(vec![
        Some(build_tcp_packet(
            "10.0.0.2",
            "3.3.3.3",
            4435,
            1337,
            b"omw to 3.3.3.3",
        )),
        Some(build_tcp_packet(
            "3.3.3.3",
            "10.0.0.2",
            1337,
            4435,
            b"I was just there!",
        )),
        Some(build_tcp_packet(
            "1.1.1.1",
            "10.0.0.2",
            12345,
            443,
            b"Is it nice there? I think 1.1.1.1 is dull",
        )),
        Some(build_tcp_packet(
            "10.0.0.2",
            "1.1.1.1",
            443,
            12345,
            b"Well, I heard 1.1.1.1 is all the rage",
        )),
        None, // sleep
        Some(build_tcp_packet(
            "10.0.0.2",
            "3.3.3.3",
            4435,
            1337,
            b"Wait for me!",
        )),
        Some(build_tcp_packet(
            "3.3.3.3",
            "10.0.0.2",
            1337,
            4435,
            b"They're waiting for you...",
        )),
        Some(build_tcp_packet(
            "1.1.1.1",
            "10.0.0.2",
            12345,
            443,
            b"1.1.1.1 forever!",
        )),
        Some(build_tcp_packet(
            "10.0.0.2",
            "1.1.1.1",
            443,
            12345,
            b"10.0.0.2 forever!",
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
    network_frames: impl IntoIterator<Item = Box<dyn DataLinkReceiver>>,
    stdout: Option<Arc<Mutex<Vec<u8>>>>,
    dns_client: Option<Client>,
    keyboard_events: Box<dyn Iterator<Item = Event> + Send>,
) -> OsInputOutput {
    let interfaces_with_frames = get_interfaces_with_frames(network_frames);

    let write_to_stdout: Box<dyn FnMut(&str) + Send> = match stdout {
        Some(stdout) => Box::new({
            move |output| {
                let mut stdout = stdout.lock().unwrap();
                writeln!(&mut stdout, "{output}").unwrap();
            }
        }),
        None => Box::new(|_output| {}),
    };

    OsInputOutput {
        interfaces_with_frames,
        get_open_sockets,
        terminal_events: keyboard_events,
        dns_client,
        write_to_stdout,
    }
}

pub fn opts_raw() -> Opt {
    Opt {
        interface: Some(String::from("interface_name")),
        raw: true,
        ..Default::default()
    }
}
pub fn opts_ui() -> Opt {
    Opt {
        interface: Some(String::from("interface_name")),
        ..Default::default()
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
