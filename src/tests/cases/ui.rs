use std::{collections::HashMap, net::IpAddr};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use insta::{assert_debug_snapshot, assert_snapshot};
use itertools::Itertools;
use pnet::datalink::DataLinkReceiver;
use rstest::rstest;

use crate::{
    cli::RenderOpts,
    start,
    tests::{
        cases::test_utils::{
            build_tcp_packet, opts_ui, os_input_output, os_input_output_factory,
            sample_frames_short, sample_frames_sustained_long,
            sample_frames_sustained_multiple_processes, sample_frames_sustained_one_process,
            sleep_and_quit_events, sleep_resize_and_quit_events, test_backend_factory,
        },
        fakes::{
            create_fake_dns_client, get_interfaces_with_frames, get_open_sockets, NetworkFrames,
            TerminalEvents,
        },
    },
    Opt, OsInputOutput,
};

const SNAPSHOT_SECTION_SEPARATOR: &str = "\n--- SECTION SEPARATOR ---\n";

#[test]
fn basic_startup() {
    let network_frames = vec![NetworkFrames::new(vec![
        None, // sleep
    ]) as Box<dyn DataLinkReceiver>];

    let (terminal_events, terminal_draw_events, backend) = test_backend_factory(190, 50);
    let os_input = os_input_output(network_frames, 1);
    let opts = opts_ui();
    start(backend, os_input, opts);

    assert_snapshot!(terminal_draw_events
        .lock()
        .unwrap()
        .join(SNAPSHOT_SECTION_SEPARATOR));
    assert_debug_snapshot!(terminal_events.lock().unwrap().as_slice());
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

    let events = [
        None,
        Some(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE)),
        None,
        None,
        Some(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE)),
        Some(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
    ]
    .into_iter()
    .map(|ke| ke.map(Event::Key))
    .collect_vec();

    let events = Box::new(TerminalEvents::new(events));
    let os_input = os_input_output_factory(network_frames, None, None, events);
    let (terminal_events, terminal_draw_events, backend) = test_backend_factory(190, 50);
    let opts = opts_ui();
    start(backend, os_input, opts);

    assert_snapshot!(terminal_draw_events
        .lock()
        .unwrap()
        .join(SNAPSHOT_SECTION_SEPARATOR));
    assert_debug_snapshot!(terminal_events.lock().unwrap().as_slice());
}

#[test]
fn rearranged_by_tab() {
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

    let events = [
        None,
        None,
        Some(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)),
        None,
        None,
        Some(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
    ]
    .into_iter()
    .map(|ke| ke.map(Event::Key))
    .collect_vec();

    let events = Box::new(TerminalEvents::new(events));
    let os_input = os_input_output_factory(network_frames, None, None, events);
    let (terminal_events, terminal_draw_events, backend) = test_backend_factory(190, 50);
    let opts = opts_ui();
    start(backend, os_input, opts);

    assert_snapshot!(terminal_draw_events
        .lock()
        .unwrap()
        .join(SNAPSHOT_SECTION_SEPARATOR));
    assert_debug_snapshot!(terminal_events.lock().unwrap().as_slice());
}

#[test]
fn basic_only_processes() {
    let network_frames = vec![NetworkFrames::new(vec![
        None, // sleep
    ]) as Box<dyn DataLinkReceiver>];

    let (_, terminal_draw_events, backend) = test_backend_factory(190, 50);
    let os_input = os_input_output(network_frames, 1);
    let opts = Opt {
        render_opts: RenderOpts {
            processes: true,
            ..Default::default()
        },
        ..opts_ui()
    };
    start(backend, os_input, opts);

    assert_snapshot!(terminal_draw_events
        .lock()
        .unwrap()
        .join(SNAPSHOT_SECTION_SEPARATOR));
}

#[test]
fn basic_processes_with_dns_queries() {
    let network_frames = vec![NetworkFrames::new(vec![
        None, // sleep
    ]) as Box<dyn DataLinkReceiver>];

    let (_, terminal_draw_events, backend) = test_backend_factory(190, 50);
    let os_input = os_input_output(network_frames, 1);
    let opts = Opt {
        show_dns: true,
        render_opts: RenderOpts {
            processes: true,
            ..Default::default()
        },
        ..opts_ui()
    };
    start(backend, os_input, opts);

    assert_snapshot!(terminal_draw_events
        .lock()
        .unwrap()
        .join(SNAPSHOT_SECTION_SEPARATOR));
}

#[test]
fn basic_only_connections() {
    let network_frames = vec![NetworkFrames::new(vec![
        None, // sleep
    ]) as Box<dyn DataLinkReceiver>];

    let (_, terminal_draw_events, backend) = test_backend_factory(190, 50);
    let os_input = os_input_output(network_frames, 1);
    let opts = Opt {
        render_opts: RenderOpts {
            connections: true,
            ..Default::default()
        },
        ..opts_ui()
    };
    start(backend, os_input, opts);

    assert_snapshot!(terminal_draw_events
        .lock()
        .unwrap()
        .join(SNAPSHOT_SECTION_SEPARATOR));
}

#[test]
fn basic_only_addresses() {
    let network_frames = vec![NetworkFrames::new(vec![
        None, // sleep
    ]) as Box<dyn DataLinkReceiver>];

    let (_, terminal_draw_events, backend) = test_backend_factory(190, 50);
    let os_input = os_input_output(network_frames, 1);
    let opts = Opt {
        render_opts: RenderOpts {
            addresses: true,
            ..Default::default()
        },
        ..opts_ui()
    };
    start(backend, os_input, opts);

    assert_snapshot!(terminal_draw_events
        .lock()
        .unwrap()
        .join(SNAPSHOT_SECTION_SEPARATOR));
}

#[rstest(sample_frames_short as frames)]
fn two_packets_only_processes(frames: Vec<Box<dyn DataLinkReceiver>>) {
    let (_, terminal_draw_events, backend) = test_backend_factory(190, 50);
    let os_input = os_input_output(frames, 2);
    let opts = Opt {
        render_opts: RenderOpts {
            processes: true,
            ..Default::default()
        },
        ..opts_ui()
    };
    start(backend, os_input, opts);

    assert_snapshot!(terminal_draw_events
        .lock()
        .unwrap()
        .join(SNAPSHOT_SECTION_SEPARATOR));
}

#[rstest(sample_frames_short as frames)]
fn two_packets_only_connections(frames: Vec<Box<dyn DataLinkReceiver>>) {
    let (_, terminal_draw_events, backend) = test_backend_factory(190, 50);
    let os_input = os_input_output(frames, 2);
    let opts = Opt {
        render_opts: RenderOpts {
            connections: true,
            ..Default::default()
        },
        ..opts_ui()
    };
    start(backend, os_input, opts);

    assert_snapshot!(terminal_draw_events
        .lock()
        .unwrap()
        .join(SNAPSHOT_SECTION_SEPARATOR));
}

#[rstest(sample_frames_short as frames)]
fn two_packets_only_addresses(frames: Vec<Box<dyn DataLinkReceiver>>) {
    let (_, terminal_draw_events, backend) = test_backend_factory(190, 50);
    let os_input = os_input_output(frames, 2);
    let opts = Opt {
        render_opts: RenderOpts {
            addresses: true,
            ..Default::default()
        },
        ..opts_ui()
    };
    start(backend, os_input, opts);

    assert_snapshot!(terminal_draw_events
        .lock()
        .unwrap()
        .join(SNAPSHOT_SECTION_SEPARATOR));
}

#[test]
fn two_windows_split_horizontally() {
    let network_frames = vec![NetworkFrames::new(vec![
        None, // sleep
    ]) as Box<dyn DataLinkReceiver>];

    let (_, terminal_draw_events, backend) = test_backend_factory(60, 50);
    let os_input = os_input_output(network_frames, 2);
    let opts = Opt {
        render_opts: RenderOpts {
            addresses: true,
            connections: true,
            ..Default::default()
        },
        ..opts_ui()
    };
    start(backend, os_input, opts);

    assert_snapshot!(terminal_draw_events
        .lock()
        .unwrap()
        .join(SNAPSHOT_SECTION_SEPARATOR));
}

#[test]
fn two_windows_split_vertically() {
    let network_frames = vec![NetworkFrames::new(vec![
        None, // sleep
    ]) as Box<dyn DataLinkReceiver>];

    let (_, terminal_draw_events, backend) = test_backend_factory(190, 50);
    let os_input = os_input_output(network_frames, 1);
    let opts = Opt {
        render_opts: RenderOpts {
            addresses: true,
            connections: true,
            ..Default::default()
        },
        ..opts_ui()
    };
    start(backend, os_input, opts);

    assert_snapshot!(terminal_draw_events
        .lock()
        .unwrap()
        .join(SNAPSHOT_SECTION_SEPARATOR));
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

    assert_snapshot!(terminal_draw_events
        .lock()
        .unwrap()
        .join(SNAPSHOT_SECTION_SEPARATOR));
    assert_debug_snapshot!(terminal_events.lock().unwrap().as_slice());
}

#[rstest(sample_frames_short as frames)]
fn bi_directional_traffic(frames: Vec<Box<dyn DataLinkReceiver>>) {
    let (terminal_events, terminal_draw_events, backend) = test_backend_factory(190, 50);
    let os_input = os_input_output(frames, 2);
    let opts = opts_ui();
    start(backend, os_input, opts);

    assert_snapshot!(terminal_draw_events
        .lock()
        .unwrap()
        .join(SNAPSHOT_SECTION_SEPARATOR));
    assert_debug_snapshot!(terminal_events.lock().unwrap().as_slice());
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

    assert_snapshot!(terminal_draw_events
        .lock()
        .unwrap()
        .join(SNAPSHOT_SECTION_SEPARATOR));
    assert_debug_snapshot!(terminal_events.lock().unwrap().as_slice());
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

    assert_snapshot!(terminal_draw_events
        .lock()
        .unwrap()
        .join(SNAPSHOT_SECTION_SEPARATOR));
    assert_debug_snapshot!(terminal_events.lock().unwrap().as_slice());
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

    assert_snapshot!(terminal_draw_events
        .lock()
        .unwrap()
        .join(SNAPSHOT_SECTION_SEPARATOR));
    assert_debug_snapshot!(terminal_events.lock().unwrap().as_slice());
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
            b"Greetings traveller, I'm from 3.3.3.3",
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

    assert_snapshot!(terminal_draw_events
        .lock()
        .unwrap()
        .join(SNAPSHOT_SECTION_SEPARATOR));
    assert_debug_snapshot!(terminal_events.lock().unwrap().as_slice());
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

    assert_snapshot!(terminal_draw_events
        .lock()
        .unwrap()
        .join(SNAPSHOT_SECTION_SEPARATOR));
    assert_debug_snapshot!(terminal_events.lock().unwrap().as_slice());
}

#[rstest(sample_frames_sustained_one_process as frames)]
fn sustained_traffic_from_one_process(frames: Vec<Box<dyn DataLinkReceiver>>) {
    let (terminal_events, terminal_draw_events, backend) = test_backend_factory(190, 50);

    let os_input = os_input_output(frames, 3);
    let opts = opts_ui();
    start(backend, os_input, opts);

    assert_snapshot!(terminal_draw_events
        .lock()
        .unwrap()
        .join(SNAPSHOT_SECTION_SEPARATOR));
    assert_debug_snapshot!(terminal_events.lock().unwrap().as_slice());
}

#[rstest(sample_frames_sustained_one_process as frames)]
fn sustained_traffic_from_one_process_total(frames: Vec<Box<dyn DataLinkReceiver>>) {
    let (terminal_events, terminal_draw_events, backend) = test_backend_factory(190, 50);

    let os_input = os_input_output(frames, 3);
    let mut opts = opts_ui();
    opts.render_opts.total_utilization = true;
    start(backend, os_input, opts);

    assert_snapshot!(terminal_draw_events
        .lock()
        .unwrap()
        .join(SNAPSHOT_SECTION_SEPARATOR));
    assert_debug_snapshot!(terminal_events.lock().unwrap().as_slice());
}

#[rstest(sample_frames_sustained_multiple_processes as frames)]
fn sustained_traffic_from_multiple_processes(frames: Vec<Box<dyn DataLinkReceiver>>) {
    let (terminal_events, terminal_draw_events, backend) = test_backend_factory(190, 50);

    let os_input = os_input_output(frames, 3);
    let opts = opts_ui();
    start(backend, os_input, opts);

    assert_snapshot!(terminal_draw_events
        .lock()
        .unwrap()
        .join(SNAPSHOT_SECTION_SEPARATOR));
    assert_debug_snapshot!(terminal_events.lock().unwrap().as_slice());
}

#[rstest(sample_frames_sustained_multiple_processes as frames)]
fn sustained_traffic_from_multiple_processes_total(frames: Vec<Box<dyn DataLinkReceiver>>) {
    let (terminal_events, terminal_draw_events, backend) = test_backend_factory(190, 50);

    let os_input = os_input_output(frames, 3);
    let mut opts = opts_ui();
    opts.render_opts.total_utilization = true;
    start(backend, os_input, opts);

    assert_snapshot!(terminal_draw_events
        .lock()
        .unwrap()
        .join(SNAPSHOT_SECTION_SEPARATOR));
    assert_debug_snapshot!(terminal_events.lock().unwrap().as_slice());
}

#[rstest(sample_frames_sustained_long as frames)]
fn sustained_traffic_from_multiple_processes_bi_directional(
    frames: Vec<Box<dyn DataLinkReceiver>>,
) {
    let (terminal_events, terminal_draw_events, backend) = test_backend_factory(190, 50);

    let os_input = os_input_output(frames, 3);
    let opts = opts_ui();
    start(backend, os_input, opts);

    assert_snapshot!(terminal_draw_events
        .lock()
        .unwrap()
        .join(SNAPSHOT_SECTION_SEPARATOR));
    assert_debug_snapshot!(terminal_events.lock().unwrap().as_slice());
}

#[rstest(sample_frames_sustained_long as frames)]
fn sustained_traffic_from_multiple_processes_bi_directional_total(
    frames: Vec<Box<dyn DataLinkReceiver>>,
) {
    let (terminal_events, terminal_draw_events, backend) = test_backend_factory(190, 50);

    let os_input = os_input_output(frames, 3);
    let mut opts = opts_ui();
    opts.render_opts.total_utilization = true;
    start(backend, os_input, opts);

    assert_snapshot!(terminal_draw_events
        .lock()
        .unwrap()
        .join(SNAPSHOT_SECTION_SEPARATOR));
    assert_debug_snapshot!(terminal_events.lock().unwrap().as_slice());
}

#[rstest(sample_frames_sustained_long as network_frames)]
fn traffic_with_host_names(network_frames: Vec<Box<dyn DataLinkReceiver>>) {
    let (terminal_events, terminal_draw_events, backend) = test_backend_factory(190, 50);

    let interfaces_with_frames = get_interfaces_with_frames(network_frames);

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
    let write_to_stdout = Box::new(|_output: &_| {});

    let os_input = OsInputOutput {
        interfaces_with_frames,
        get_open_sockets,
        terminal_events: sleep_and_quit_events(3),
        dns_client,
        write_to_stdout,
    };
    let opts = opts_ui();
    start(backend, os_input, opts);

    assert_snapshot!(terminal_draw_events
        .lock()
        .unwrap()
        .join(SNAPSHOT_SECTION_SEPARATOR));
    assert_debug_snapshot!(terminal_events.lock().unwrap().as_slice());
}

#[rstest(sample_frames_sustained_long as network_frames)]
fn truncate_long_hostnames(network_frames: Vec<Box<dyn DataLinkReceiver>>) {
    let (terminal_events, terminal_draw_events, backend) = test_backend_factory(190, 50);

    let interfaces_with_frames = get_interfaces_with_frames(network_frames);

    let mut ips_to_hostnames = HashMap::new();
    ips_to_hostnames.insert(
        IpAddr::V4("1.1.1.1".parse().unwrap()),
        String::from("i.am.not.too.long"),
    );
    ips_to_hostnames.insert(
        IpAddr::V4("3.3.3.3".parse().unwrap()),
        String::from("i.am.an.obnoxiosuly.long.hostname.why.would.anyone.do.this.really.i.ask"),
    );
    ips_to_hostnames.insert(
        IpAddr::V4("10.0.0.2".parse().unwrap()),
        String::from("i-like-cheese.com"),
    );
    let dns_client = create_fake_dns_client(ips_to_hostnames);
    let write_to_stdout = Box::new(|_output: &_| {});

    let os_input = OsInputOutput {
        interfaces_with_frames,
        get_open_sockets,
        terminal_events: sleep_and_quit_events(3),
        dns_client,
        write_to_stdout,
    };
    let opts = opts_ui();
    start(backend, os_input, opts);

    assert_snapshot!(terminal_draw_events
        .lock()
        .unwrap()
        .join(SNAPSHOT_SECTION_SEPARATOR));
    assert_debug_snapshot!(terminal_events.lock().unwrap().as_slice());
}

#[rstest(sample_frames_sustained_long as network_frames)]
fn no_resolve_mode(network_frames: Vec<Box<dyn DataLinkReceiver>>) {
    let (terminal_events, terminal_draw_events, backend) = test_backend_factory(190, 50);

    let interfaces_with_frames = get_interfaces_with_frames(network_frames);

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
    let write_to_stdout = Box::new(|_output: &_| {});

    let os_input = OsInputOutput {
        interfaces_with_frames,
        get_open_sockets,
        terminal_events: sleep_and_quit_events(3),
        dns_client,
        write_to_stdout,
    };
    let opts = opts_ui();
    start(backend, os_input, opts);

    assert_snapshot!(terminal_draw_events
        .lock()
        .unwrap()
        .join(SNAPSHOT_SECTION_SEPARATOR));
    assert_debug_snapshot!(terminal_events.lock().unwrap().as_slice());
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
    let interfaces_with_frames = get_interfaces_with_frames(network_frames);

    let (terminal_events, terminal_draw_events, backend) = test_backend_factory(190, 50);

    let dns_client = create_fake_dns_client(HashMap::new());
    let write_to_stdout = Box::new(|_output: &_| {});

    let os_input = OsInputOutput {
        interfaces_with_frames,
        get_open_sockets,
        terminal_events: sleep_resize_and_quit_events(2),
        dns_client,
        write_to_stdout,
    };
    let opts = opts_ui();
    start(backend, os_input, opts);

    assert_snapshot!(terminal_draw_events
        .lock()
        .unwrap()
        .join(SNAPSHOT_SECTION_SEPARATOR));
    assert_debug_snapshot!(terminal_events.lock().unwrap().as_slice());
}

#[rstest]
#[case("full-width-under-30-height", 190, 29)]
#[case("under-120-width-full-height", 119, 50)]
#[case("under-120-width-under-30-height", 119, 29)]
#[case("under-50-width-under-50-height", 50, 50)]
#[case("under-70-width-under-30-height", 69, 29)]
fn layout(#[case] name: &str, #[case] width: u16, #[case] height: u16) {
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
            b"Greetings traveller, I'm from 3.3.3.3",
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

    let (terminal_events, terminal_draw_events, backend) = test_backend_factory(width, height);

    let os_input = os_input_output(network_frames, 2);
    let opts = opts_ui();
    start(backend, os_input, opts);

    assert_snapshot!(
        format!("layout-{name}-draw_events"),
        terminal_draw_events
            .lock()
            .unwrap()
            .join(SNAPSHOT_SECTION_SEPARATOR)
    );
    assert_debug_snapshot!(
        format!("layout-{name}-events"),
        terminal_events.lock().unwrap().as_slice()
    );
}
