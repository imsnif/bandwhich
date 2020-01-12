use crate::tests::fakes::TerminalEvent::*;
use crate::tests::fakes::{
    create_fake_dns_client, create_fake_on_winch, get_interfaces, get_open_sockets, NetworkFrames,
};

use ::insta::assert_snapshot;

use ::std::collections::HashMap;
use ::std::net::IpAddr;

use crate::tests::cases::test_utils::{
    opts_ui, os_input_output, os_input_output_factory, sleep_and_quit_events, test_backend_factory,
};
use ::termion::event::{Event, Key};
use packet_builder::payload::PayloadData;
use packet_builder::*;
use pnet_bandwhich_fork::datalink::DataLinkReceiver;
use pnet_bandwhich_fork::packet::Packet;
use pnet_base::MacAddr;
use std::iter;

use crate::tests::fakes::KeyboardEvents;

use crate::{start, Opt, OsInputOutput};

fn build_tcp_packet(
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

#[test]
fn basic_startup() {
    let network_frames = vec![NetworkFrames::new(vec![
        None, // sleep
    ]) as Box<dyn DataLinkReceiver>];

    let (terminal_events, terminal_draw_events, backend) = test_backend_factory(190, 50);
    let os_input = os_input_output(network_frames, 1);
    let opts = opts_ui();
    start(backend, os_input, opts);
    let terminal_draw_events_mirror = terminal_draw_events.lock().unwrap();

    let expected_terminal_events = vec![Clear, HideCursor, Draw, Flush, Clear, ShowCursor];
    assert_eq!(
        &terminal_events.lock().unwrap()[..],
        &expected_terminal_events[..]
    );

    assert_eq!(terminal_draw_events_mirror.len(), 1);
    assert_snapshot!(&terminal_draw_events_mirror[0]);
}

#[test]
fn pause_by_space() {
    let network_frames = vec![NetworkFrames::new(vec![
        Some(build_tcp_packet(
            "1.1.1.1",
            "10.0.0.2",
            12345,
            443,
            b"I have come from 1.1.1.1",
        )),
        None, // sleep
        None, // sleep
        None, // sleep
        Some(build_tcp_packet(
            "1.1.1.1",
            "10.0.0.2",
            12345,
            443,
            b"Same here, but one second later",
        )),
    ]) as Box<dyn DataLinkReceiver>];

    // sleep for 1s, then press space, sleep for 2s, then quit
    let mut events: Vec<Option<Event>> = iter::repeat(None).take(1).collect();
    events.push(Some(Event::Key(Key::Char(' '))));
    events.push(None);
    events.push(None);
    events.push(Some(Event::Key(Key::Char(' '))));
    events.push(Some(Event::Key(Key::Ctrl('c'))));

    let events = Box::new(KeyboardEvents::new(events));
    let os_input = os_input_output_factory(network_frames, None, None, events);
    let (terminal_events, terminal_draw_events, backend) = test_backend_factory(190, 50);
    let opts = opts_ui();
    start(backend, os_input, opts);
    let terminal_draw_events_mirror = terminal_draw_events.lock().unwrap();
    let expected_terminal_events = vec![
        Clear, HideCursor, Draw, Flush, Draw, Flush, Draw, Flush, Clear, ShowCursor,
    ];
    assert_eq!(
        &terminal_events.lock().unwrap()[..],
        &expected_terminal_events[..]
    );
    assert_eq!(terminal_draw_events_mirror.len(), 3);
    assert_snapshot!(&terminal_draw_events_mirror[0]);
    assert_snapshot!(&terminal_draw_events_mirror[1]);
    assert_snapshot!(&terminal_draw_events_mirror[2]);
}

#[test]
fn basic_only_processes() {
    let network_frames = vec![NetworkFrames::new(vec![
        None, // sleep
    ]) as Box<dyn DataLinkReceiver>];

    let (_, terminal_draw_events, backend) = test_backend_factory(190, 50);
    let os_input = os_input_output(network_frames, 1);
    let opts = Opt {
            interface: Some(String::from("interface_name")),
            raw:false,
            no_resolve: false,
            addresses: false,
            connections: false,
            processes: true,
        };

    start(backend, os_input, opts);
    let terminal_draw_events_mirror = terminal_draw_events.lock().unwrap();
    assert_snapshot!(&terminal_draw_events_mirror[0]);
}
#[test]
fn basic_only_connections() {
    let network_frames = vec![NetworkFrames::new(vec![
        None, // sleep
    ]) as Box<dyn DataLinkReceiver>];

    let (_, terminal_draw_events, backend) = test_backend_factory(190, 50);
    let os_input = os_input_output(network_frames, 1);
    let opts = Opt {
            interface: Some(String::from("interface_name")),
            raw:false,
            no_resolve: false,
            addresses: false,
            connections: true,
            processes: false,
        };

    start(backend, os_input, opts);
    let terminal_draw_events_mirror = terminal_draw_events.lock().unwrap();
    assert_snapshot!(&terminal_draw_events_mirror[0]);
}

#[test]
fn basic_only_addresses() {
    let network_frames = vec![NetworkFrames::new(vec![
        None, // sleep
    ]) as Box<dyn DataLinkReceiver>];

    let (_, terminal_draw_events, backend) = test_backend_factory(190, 50);
    let os_input = os_input_output(network_frames, 1);
    let opts = Opt {
            interface: Some(String::from("interface_name")),
            raw:false,
            no_resolve: false,
            addresses: true,
            connections: false,
            processes: false,
        };

    start(backend, os_input, opts);
    let terminal_draw_events_mirror = terminal_draw_events.lock().unwrap();
    assert_snapshot!(&terminal_draw_events_mirror[0]);
}

#[test]
fn two_windows_split_horizontally() {
    let network_frames = vec![NetworkFrames::new(vec![
        None, // sleep
    ]) as Box<dyn DataLinkReceiver>];

    let (_, terminal_draw_events, backend) = test_backend_factory(60, 50);
    let os_input = os_input_output(network_frames, 1);
    let opts = Opt {
            interface: Some(String::from("interface_name")),
            raw:false,
            no_resolve: false,
            addresses: true,
            connections: true,
            processes: false,
        };

    start(backend, os_input, opts);
    let terminal_draw_events_mirror = terminal_draw_events.lock().unwrap();
    assert_snapshot!(&terminal_draw_events_mirror[0]);
}


#[test]
fn two_windows_split_vertically() {
    let network_frames = vec![NetworkFrames::new(vec![
        None, // sleep
    ]) as Box<dyn DataLinkReceiver>];

    let (_, terminal_draw_events, backend) = test_backend_factory(190, 50);
    let os_input = os_input_output(network_frames, 1);
    let opts = Opt {
            interface: Some(String::from("interface_name")),
            raw:false,
            no_resolve: false,
            addresses: true,
            connections: true,
            processes: false,
        };

    start(backend, os_input, opts);
    let terminal_draw_events_mirror = terminal_draw_events.lock().unwrap();
    assert_snapshot!(&terminal_draw_events_mirror[0]);
}

#[test]
fn one_packet_of_traffic() {
    let network_frames = vec![NetworkFrames::new(vec![Some(build_tcp_packet(
        "10.0.0.2",
        "1.1.1.1",
        443,
        12345,
        b"I am a fake tcp packet",
    ))]) as Box<dyn DataLinkReceiver>];
    let (terminal_events, terminal_draw_events, backend) = test_backend_factory(190, 50);
    let os_input = os_input_output(network_frames, 2);
    let opts = opts_ui();
    start(backend, os_input, opts);
    let terminal_draw_events_mirror = terminal_draw_events.lock().unwrap();

    let expected_terminal_events = vec![
        Clear, HideCursor, Draw, Flush, Draw, Flush, Clear, ShowCursor,
    ];
    assert_eq!(
        &terminal_events.lock().unwrap()[..],
        &expected_terminal_events[..]
    );

    assert_eq!(terminal_draw_events_mirror.len(), 2);
    assert_snapshot!(&terminal_draw_events_mirror[0]);
    assert_snapshot!(&terminal_draw_events_mirror[1]);
}

#[test]
fn bi_directional_traffic() {
    let network_frames = vec![NetworkFrames::new(vec![
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
    ]) as Box<dyn DataLinkReceiver>];

    let (terminal_events, terminal_draw_events, backend) = test_backend_factory(190, 50);
    let os_input = os_input_output(network_frames, 2);
    let opts = opts_ui();
    start(backend, os_input, opts);
    let terminal_draw_events_mirror = terminal_draw_events.lock().unwrap();

    let expected_terminal_events = vec![
        Clear, HideCursor, Draw, Flush, Draw, Flush, Clear, ShowCursor,
    ];
    assert_eq!(
        &terminal_events.lock().unwrap()[..],
        &expected_terminal_events[..]
    );

    assert_eq!(terminal_draw_events_mirror.len(), 2);
    assert_snapshot!(&terminal_draw_events_mirror[0]);
    assert_snapshot!(&terminal_draw_events_mirror[1]);
}

#[test]
fn multiple_packets_of_traffic_from_different_connections() {
    let network_frames = vec![NetworkFrames::new(vec![
        Some(build_tcp_packet(
            "2.2.2.2",
            "10.0.0.2",
            12345,
            443,
            b"I have come from 2.2.2.2",
        )),
        Some(build_tcp_packet(
            "2.2.2.2",
            "10.0.0.2",
            54321,
            4434,
            b"I come from 2.2.2.2",
        )),
    ]) as Box<dyn DataLinkReceiver>];

    let (terminal_events, terminal_draw_events, backend) = test_backend_factory(190, 50);
    let os_input = os_input_output(network_frames, 2);
    let opts = opts_ui();
    start(backend, os_input, opts);
    let terminal_draw_events_mirror = terminal_draw_events.lock().unwrap();

    let expected_terminal_events = vec![
        Clear, HideCursor, Draw, Flush, Draw, Flush, Clear, ShowCursor,
    ];
    assert_eq!(
        &terminal_events.lock().unwrap()[..],
        &expected_terminal_events[..]
    );

    assert_eq!(terminal_draw_events_mirror.len(), 2);
    assert_snapshot!(&terminal_draw_events_mirror[0]);
    assert_snapshot!(&terminal_draw_events_mirror[1]);
}

#[test]
fn multiple_packets_of_traffic_from_single_connection() {
    let network_frames = vec![NetworkFrames::new(vec![
        Some(build_tcp_packet(
            "1.1.1.1",
            "10.0.0.2",
            12345,
            443,
            b"I have come from 1.1.1.1",
        )),
        Some(build_tcp_packet(
            "1.1.1.1",
            "10.0.0.2",
            12345,
            443,
            b"I've come from 1.1.1.1 too!",
        )),
    ]) as Box<dyn DataLinkReceiver>];

    let (terminal_events, terminal_draw_events, backend) = test_backend_factory(190, 50);
    let os_input = os_input_output(network_frames, 2);
    let opts = opts_ui();
    start(backend, os_input, opts);
    let terminal_draw_events_mirror = terminal_draw_events.lock().unwrap();

    let expected_terminal_events = vec![
        Clear, HideCursor, Draw, Flush, Draw, Flush, Clear, ShowCursor,
    ];
    assert_eq!(
        &terminal_events.lock().unwrap()[..],
        &expected_terminal_events[..]
    );

    assert_eq!(terminal_draw_events_mirror.len(), 2);
    assert_snapshot!(&terminal_draw_events_mirror[0]);
    assert_snapshot!(&terminal_draw_events_mirror[1]);
}

#[test]
fn one_process_with_multiple_connections() {
    let network_frames = vec![NetworkFrames::new(vec![
        Some(build_tcp_packet(
            "1.1.1.1",
            "10.0.0.2",
            12345,
            443,
            b"I have come from 1.1.1.1",
        )),
        Some(build_tcp_packet(
            "1.1.1.1",
            "10.0.0.2",
            12346,
            443,
            b"Funny that, I'm from 1.1.1.1",
        )),
    ]) as Box<dyn DataLinkReceiver>];

    let (terminal_events, terminal_draw_events, backend) = test_backend_factory(190, 50);
    let os_input = os_input_output(network_frames, 2);
    let opts = opts_ui();
    start(backend, os_input, opts);
    let terminal_draw_events_mirror = terminal_draw_events.lock().unwrap();

    let expected_terminal_events = vec![
        Clear, HideCursor, Draw, Flush, Draw, Flush, Clear, ShowCursor,
    ];
    assert_eq!(
        &terminal_events.lock().unwrap()[..],
        &expected_terminal_events[..]
    );

    assert_eq!(terminal_draw_events_mirror.len(), 2);
    assert_snapshot!(&terminal_draw_events_mirror[0]);
    assert_snapshot!(&terminal_draw_events_mirror[1]);
}

#[test]
fn multiple_processes_with_multiple_connections() {
    let network_frames = vec![NetworkFrames::new(vec![
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
            b"Awesome, I'm from 3.3.3.3",
        )),
        Some(build_tcp_packet(
            "2.2.2.2",
            "10.0.0.2",
            54321,
            4434,
            b"You know, 2.2.2.2 is really nice!",
        )),
        Some(build_tcp_packet(
            "4.4.4.4",
            "10.0.0.2",
            1337,
            4432,
            b"I'm partial to 4.4.4.4",
        )),
    ]) as Box<dyn DataLinkReceiver>];

    let (terminal_events, terminal_draw_events, backend) = test_backend_factory(190, 50);
    let os_input = os_input_output(network_frames, 2);
    let opts = opts_ui();
    start(backend, os_input, opts);
    let terminal_draw_events_mirror = terminal_draw_events.lock().unwrap();

    let expected_terminal_events = vec![
        Clear, HideCursor, Draw, Flush, Draw, Flush, Clear, ShowCursor,
    ];
    assert_eq!(
        &terminal_events.lock().unwrap()[..],
        &expected_terminal_events[..]
    );

    assert_eq!(terminal_draw_events_mirror.len(), 2);
    assert_snapshot!(&terminal_draw_events_mirror[0]);
    assert_snapshot!(&terminal_draw_events_mirror[1]);
}

#[test]
fn multiple_connections_from_remote_address() {
    let network_frames = vec![NetworkFrames::new(vec![
        Some(build_tcp_packet(
            "1.1.1.1",
            "10.0.0.2",
            12345,
            443,
            b"I have come from 1.1.1.1",
        )),
        Some(build_tcp_packet(
            "1.1.1.1",
            "10.0.0.2",
            12346,
            443,
            b"Me too, but on a different port",
        )),
    ]) as Box<dyn DataLinkReceiver>];

    let (terminal_events, terminal_draw_events, backend) = test_backend_factory(190, 50);
    let os_input = os_input_output(network_frames, 2);
    let opts = opts_ui();
    start(backend, os_input, opts);
    let terminal_draw_events_mirror = terminal_draw_events.lock().unwrap();

    let expected_terminal_events = vec![
        Clear, HideCursor, Draw, Flush, Draw, Flush, Clear, ShowCursor,
    ];
    assert_eq!(
        &terminal_events.lock().unwrap()[..],
        &expected_terminal_events[..]
    );

    assert_eq!(terminal_draw_events_mirror.len(), 2);
    assert_snapshot!(&terminal_draw_events_mirror[0]);
    assert_snapshot!(&terminal_draw_events_mirror[1]);
}

#[test]
fn sustained_traffic_from_one_process() {
    let network_frames = vec![NetworkFrames::new(vec![
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
    ]) as Box<dyn DataLinkReceiver>];

    let (terminal_events, terminal_draw_events, backend) = test_backend_factory(190, 50);

    let os_input = os_input_output(network_frames, 3);
    let opts = opts_ui();
    start(backend, os_input, opts);
    let terminal_draw_events_mirror = terminal_draw_events.lock().unwrap();

    let expected_terminal_events = vec![
        Clear, HideCursor, Draw, Flush, Draw, Flush, Draw, Flush, Clear, ShowCursor,
    ];
    assert_eq!(
        &terminal_events.lock().unwrap()[..],
        &expected_terminal_events[..]
    );

    assert_eq!(terminal_draw_events_mirror.len(), 3);
    assert_snapshot!(&terminal_draw_events_mirror[1]);
    assert_snapshot!(&terminal_draw_events_mirror[2]);
}

#[test]
fn sustained_traffic_from_multiple_processes() {
    let network_frames = vec![NetworkFrames::new(vec![
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
    ]) as Box<dyn DataLinkReceiver>];

    let (terminal_events, terminal_draw_events, backend) = test_backend_factory(190, 50);

    let os_input = os_input_output(network_frames, 3);
    let opts = opts_ui();
    start(backend, os_input, opts);
    let terminal_draw_events_mirror = terminal_draw_events.lock().unwrap();

    let expected_terminal_events = vec![
        Clear, HideCursor, Draw, Flush, Draw, Flush, Draw, Flush, Clear, ShowCursor,
    ];
    assert_eq!(
        &terminal_events.lock().unwrap()[..],
        &expected_terminal_events[..]
    );

    assert_eq!(terminal_draw_events_mirror.len(), 3);
    assert_snapshot!(&terminal_draw_events_mirror[1]);
    assert_snapshot!(&terminal_draw_events_mirror[2]);
}

#[test]
fn sustained_traffic_from_multiple_processes_bi_directional() {
    let network_frames = vec![NetworkFrames::new(vec![
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
    ]) as Box<dyn DataLinkReceiver>];

    let (terminal_events, terminal_draw_events, backend) = test_backend_factory(190, 50);

    let os_input = os_input_output(network_frames, 3);
    let opts = opts_ui();
    start(backend, os_input, opts);
    let terminal_draw_events_mirror = terminal_draw_events.lock().unwrap();

    let expected_terminal_events = vec![
        Clear, HideCursor, Draw, Flush, Draw, Flush, Draw, Flush, Clear, ShowCursor,
    ];
    assert_eq!(
        &terminal_events.lock().unwrap()[..],
        &expected_terminal_events[..]
    );

    assert_eq!(terminal_draw_events_mirror.len(), 3);
    assert_snapshot!(&terminal_draw_events_mirror[1]);
    assert_snapshot!(&terminal_draw_events_mirror[2]);
}

#[test]
fn traffic_with_host_names() {
    let network_frames = vec![NetworkFrames::new(vec![
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
    ]) as Box<dyn DataLinkReceiver>];

    let (terminal_events, terminal_draw_events, backend) = test_backend_factory(190, 50);

    let mut ips_to_hostnames = HashMap::new();
    ips_to_hostnames.insert(
        IpAddr::V4("1.1.1.1".parse().unwrap()),
        String::from("one.one.one.one"),
    );
    ips_to_hostnames.insert(
        IpAddr::V4("3.3.3.3".parse().unwrap()),
        String::from("three.three.three.three"),
    );
    ips_to_hostnames.insert(
        IpAddr::V4("10.0.0.2".parse().unwrap()),
        String::from("i-like-cheese.com"),
    );
    let dns_client = create_fake_dns_client(ips_to_hostnames);
    let on_winch = create_fake_on_winch(false);
    let cleanup = Box::new(|| {});
    let write_to_stdout = Box::new({ move |_output: String| {} });

    let os_input = OsInputOutput {
        network_interfaces: get_interfaces(),
        network_frames,
        get_open_sockets,
        keyboard_events: sleep_and_quit_events(3),
        dns_client,
        on_winch,
        cleanup,
        write_to_stdout,
    };
    let opts = opts_ui();
    start(backend, os_input, opts);
    let terminal_draw_events_mirror = terminal_draw_events.lock().unwrap();

    let expected_terminal_events = vec![
        Clear, HideCursor, Draw, Flush, Draw, Flush, Draw, Flush, Clear, ShowCursor,
    ];
    assert_eq!(
        &terminal_events.lock().unwrap()[..],
        &expected_terminal_events[..]
    );

    assert_eq!(terminal_draw_events_mirror.len(), 3);
    assert_snapshot!(&terminal_draw_events_mirror[1]);
    assert_snapshot!(&terminal_draw_events_mirror[2]);
}

#[test]
fn no_resolve_mode() {
    let network_frames = vec![NetworkFrames::new(vec![
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
    ]) as Box<dyn DataLinkReceiver>];
    let (terminal_events, terminal_draw_events, backend) = test_backend_factory(190, 50);

    let mut ips_to_hostnames = HashMap::new();
    ips_to_hostnames.insert(
        IpAddr::V4("1.1.1.1".parse().unwrap()),
        String::from("one.one.one.one"),
    );
    ips_to_hostnames.insert(
        IpAddr::V4("3.3.3.3".parse().unwrap()),
        String::from("three.three.three.three"),
    );
    ips_to_hostnames.insert(
        IpAddr::V4("10.0.0.2".parse().unwrap()),
        String::from("i-like-cheese.com"),
    );
    let dns_client = None;
    let on_winch = create_fake_on_winch(false);
    let cleanup = Box::new(|| {});
    let write_to_stdout = Box::new({ move |_output: String| {} });

    let os_input = OsInputOutput {
        network_interfaces: get_interfaces(),
        network_frames,
        get_open_sockets,
        keyboard_events: sleep_and_quit_events(3),
        dns_client,
        on_winch,
        cleanup,
        write_to_stdout,
    };
    let opts = Opt {
        interface: Some(String::from("interface_name")),
        raw: false,
        no_resolve: true,
        addresses: false,
        connections: false,
        processes: false,
    };
    start(backend, os_input, opts);
    let terminal_draw_events_mirror = terminal_draw_events.lock().unwrap();

    let expected_terminal_events = vec![
        Clear, HideCursor, Draw, Flush, Draw, Flush, Draw, Flush, Clear, ShowCursor,
    ];
    assert_eq!(
        &terminal_events.lock().unwrap()[..],
        &expected_terminal_events[..]
    );

    assert_eq!(terminal_draw_events_mirror.len(), 3);
    assert_snapshot!(&terminal_draw_events_mirror[1]);
    assert_snapshot!(&terminal_draw_events_mirror[2]);
}

#[test]
fn traffic_with_winch_event() {
    let network_frames = vec![NetworkFrames::new(vec![Some(build_tcp_packet(
        "10.0.0.2",
        "1.1.1.1",
        443,
        12345,
        b"I am a fake tcp packet",
    ))]) as Box<dyn DataLinkReceiver>];

    let (terminal_events, terminal_draw_events, backend) = test_backend_factory(190, 50);

    let dns_client = create_fake_dns_client(HashMap::new());
    let on_winch = create_fake_on_winch(true);
    let cleanup = Box::new(|| {});
    let write_to_stdout = Box::new({ move |_output: String| {} });

    let os_input = OsInputOutput {
        network_interfaces: get_interfaces(),
        network_frames,
        get_open_sockets,
        keyboard_events: sleep_and_quit_events(2),
        dns_client,
        on_winch,
        cleanup,
        write_to_stdout,
    };
    let opts = opts_ui();
    start(backend, os_input, opts);
    let terminal_draw_events_mirror = terminal_draw_events.lock().unwrap();

    let expected_terminal_events = vec![
        Clear, HideCursor, Draw, Flush, Draw, Flush, Draw, Flush, Clear, ShowCursor,
    ];
    assert_eq!(
        &terminal_events.lock().unwrap()[..],
        &expected_terminal_events[..]
    );

    assert_eq!(terminal_draw_events_mirror.len(), 3);
    assert_snapshot!(&terminal_draw_events_mirror[0]);
    assert_snapshot!(&terminal_draw_events_mirror[1]);
    assert_snapshot!(&terminal_draw_events_mirror[2]);
}

#[test]
fn layout_full_width_under_30_height() {
    let network_frames = vec![NetworkFrames::new(vec![
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
            b"Awesome, I'm from 3.3.3.3",
        )),
        Some(build_tcp_packet(
            "2.2.2.2",
            "10.0.0.2",
            54321,
            4434,
            b"You know, 2.2.2.2 is really nice!",
        )),
        Some(build_tcp_packet(
            "4.4.4.4",
            "10.0.0.2",
            1337,
            4432,
            b"I'm partial to 4.4.4.4",
        )),
    ]) as Box<dyn DataLinkReceiver>];

    let (terminal_events, terminal_draw_events, backend) = test_backend_factory(190, 29);

    let os_input = os_input_output(network_frames, 2);
    let opts = opts_ui();
    start(backend, os_input, opts);
    let terminal_draw_events_mirror = terminal_draw_events.lock().unwrap();

    let expected_terminal_events = vec![
        Clear, HideCursor, Draw, Flush, Draw, Flush, Clear, ShowCursor,
    ];
    assert_eq!(
        &terminal_events.lock().unwrap()[..],
        &expected_terminal_events[..]
    );

    assert_eq!(terminal_draw_events_mirror.len(), 2);
    assert_snapshot!(&terminal_draw_events_mirror[0]);
    assert_snapshot!(&terminal_draw_events_mirror[1]);
}

#[test]
fn layout_under_150_width_full_height() {
    let network_frames = vec![NetworkFrames::new(vec![
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
            b"Awesome, I'm from 3.3.3.3",
        )),
        Some(build_tcp_packet(
            "2.2.2.2",
            "10.0.0.2",
            54321,
            4434,
            b"You know, 2.2.2.2 is really nice!",
        )),
        Some(build_tcp_packet(
            "4.4.4.4",
            "10.0.0.2",
            1337,
            4432,
            b"I'm partial to 4.4.4.4",
        )),
    ]) as Box<dyn DataLinkReceiver>];

    let (terminal_events, terminal_draw_events, backend) = test_backend_factory(149, 50);

    let os_input = os_input_output(network_frames, 2);
    let opts = opts_ui();
    start(backend, os_input, opts);
    let terminal_draw_events_mirror = terminal_draw_events.lock().unwrap();

    let expected_terminal_events = vec![
        Clear, HideCursor, Draw, Flush, Draw, Flush, Clear, ShowCursor,
    ];
    assert_eq!(
        &terminal_events.lock().unwrap()[..],
        &expected_terminal_events[..]
    );

    assert_eq!(terminal_draw_events_mirror.len(), 2);
    assert_snapshot!(&terminal_draw_events_mirror[0]);
    assert_snapshot!(&terminal_draw_events_mirror[1]);
}

#[test]
fn layout_under_150_width_under_30_height() {
    let network_frames = vec![NetworkFrames::new(vec![
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
            b"Awesome, I'm from 3.3.3.3",
        )),
        Some(build_tcp_packet(
            "2.2.2.2",
            "10.0.0.2",
            54321,
            4434,
            b"You know, 2.2.2.2 is really nice!",
        )),
        Some(build_tcp_packet(
            "4.4.4.4",
            "10.0.0.2",
            1337,
            4432,
            b"I'm partial to 4.4.4.4",
        )),
    ]) as Box<dyn DataLinkReceiver>];
    let (terminal_events, terminal_draw_events, backend) = test_backend_factory(149, 29);

    let os_input = os_input_output(network_frames, 2);
    let opts = opts_ui();
    start(backend, os_input, opts);
    let terminal_draw_events_mirror = terminal_draw_events.lock().unwrap();

    let expected_terminal_events = vec![
        Clear, HideCursor, Draw, Flush, Draw, Flush, Clear, ShowCursor,
    ];
    assert_eq!(
        &terminal_events.lock().unwrap()[..],
        &expected_terminal_events[..]
    );

    assert_eq!(terminal_draw_events_mirror.len(), 2);
    assert_snapshot!(&terminal_draw_events_mirror[0]);
    assert_snapshot!(&terminal_draw_events_mirror[1]);
}

#[test]
fn layout_under_120_width_full_height() {
    let network_frames = vec![NetworkFrames::new(vec![
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
            b"Awesome, I'm from 3.3.3.3",
        )),
        Some(build_tcp_packet(
            "2.2.2.2",
            "10.0.0.2",
            54321,
            4434,
            b"You know, 2.2.2.2 is really nice!",
        )),
        Some(build_tcp_packet(
            "4.4.4.4",
            "10.0.0.2",
            1337,
            4432,
            b"I'm partial to 4.4.4.4",
        )),
    ]) as Box<dyn DataLinkReceiver>];
    let (terminal_events, terminal_draw_events, backend) = test_backend_factory(119, 50);

    let os_input = os_input_output(network_frames, 2);
    let opts = opts_ui();
    start(backend, os_input, opts);
    let terminal_draw_events_mirror = terminal_draw_events.lock().unwrap();

    let expected_terminal_events = vec![
        Clear, HideCursor, Draw, Flush, Draw, Flush, Clear, ShowCursor,
    ];
    assert_eq!(
        &terminal_events.lock().unwrap()[..],
        &expected_terminal_events[..]
    );

    assert_eq!(terminal_draw_events_mirror.len(), 2);
    assert_snapshot!(&terminal_draw_events_mirror[0]);
    assert_snapshot!(&terminal_draw_events_mirror[1]);
}

#[test]
fn layout_under_120_width_under_30_height() {
    let network_frames = vec![NetworkFrames::new(vec![
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
            b"Awesome, I'm from 3.3.3.3",
        )),
        Some(build_tcp_packet(
            "2.2.2.2",
            "10.0.0.2",
            54321,
            4434,
            b"You know, 2.2.2.2 is really nice!",
        )),
        Some(build_tcp_packet(
            "4.4.4.4",
            "10.0.0.2",
            1337,
            4432,
            b"I'm partial to 4.4.4.4",
        )),
    ]) as Box<dyn DataLinkReceiver>];
    let (terminal_events, terminal_draw_events, backend) = test_backend_factory(119, 29);
    let os_input = os_input_output(network_frames, 2);
    let opts = opts_ui();
    start(backend, os_input, opts);
    let terminal_draw_events_mirror = terminal_draw_events.lock().unwrap();

    let expected_terminal_events = vec![
        Clear, HideCursor, Draw, Flush, Draw, Flush, Clear, ShowCursor,
    ];
    assert_eq!(
        &terminal_events.lock().unwrap()[..],
        &expected_terminal_events[..]
    );

    assert_eq!(terminal_draw_events_mirror.len(), 2);
    assert_snapshot!(&terminal_draw_events_mirror[0]);
    assert_snapshot!(&terminal_draw_events_mirror[1]);
}
