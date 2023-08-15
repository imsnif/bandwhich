use crate::tests::fakes::{create_fake_dns_client, NetworkFrames};

use ::insta::assert_snapshot;
use ::std::sync::{Arc, Mutex};

use ::std::collections::HashMap;
use ::std::net::IpAddr;

use packet_builder::*;
use pnet::{datalink::DataLinkReceiver, packet::Packet};

use crate::tests::cases::test_utils::{
    build_tcp_packet, opts_raw, os_input_output_dns, os_input_output_stdout, test_backend_factory,
};

use crate::{start, Opt, RenderOpts};

fn build_ip_tcp_packet(
    source_ip: &str,
    destination_ip: &str,
    source_port: u16,
    destination_port: u16,
    payload: &'static [u8],
) -> Vec<u8> {
    let mut pkt_buf = [0u8; 1500];
    let pkt = packet_builder!(
         pkt_buf,
         ipv4({set_source => ipv4addr!(source_ip), set_destination => ipv4addr!(destination_ip) }) /
         tcp({set_source => source_port, set_destination => destination_port }) /
         payload(payload)
    );
    pkt.packet().to_vec()
}

fn format_raw_output(output: Vec<u8>) -> String {
    let stdout_utf8 = String::from_utf8(output).unwrap();
    use regex::Regex;
    let timestamp = Regex::new(r"<\d+>").unwrap();
    let replaced = timestamp.replace_all(&stdout_utf8, "<TIMESTAMP_REMOVED>");
    format!("{}", replaced)
}

#[test]
fn one_ip_packet_of_traffic() {
    let network_frames = vec![NetworkFrames::new(vec![Some(build_ip_tcp_packet(
        "10.0.0.2",
        "1.1.1.1",
        443,
        12345,
        b"I am a fake tcp packet",
    ))]) as Box<dyn DataLinkReceiver>];
    let (_, _, backend) = test_backend_factory(190, 50);
    let stdout = Arc::new(Mutex::new(Vec::new()));
    let os_input = os_input_output_stdout(network_frames, 2, Some(stdout.clone()));
    let opts = opts_raw();
    start(backend, os_input, opts);
    let stdout = Arc::try_unwrap(stdout).unwrap().into_inner().unwrap();
    let formatted = format_raw_output(stdout);
    assert_snapshot!(formatted);
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
    let (_, _, backend) = test_backend_factory(190, 50);
    let stdout = Arc::new(Mutex::new(Vec::new()));
    let os_input = os_input_output_stdout(network_frames, 2, Some(stdout.clone()));
    let opts = opts_raw();
    start(backend, os_input, opts);
    let stdout = Arc::try_unwrap(stdout).unwrap().into_inner().unwrap();
    let formatted = format_raw_output(stdout);
    assert_snapshot!(formatted);
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
    let (_, _, backend) = test_backend_factory(190, 50);
    let stdout = Arc::new(Mutex::new(Vec::new()));
    let os_input = os_input_output_stdout(network_frames, 2, Some(stdout.clone()));
    let opts = opts_raw();
    start(backend, os_input, opts);
    let stdout = Arc::try_unwrap(stdout).unwrap().into_inner().unwrap();
    let formatted = format_raw_output(stdout);
    assert_snapshot!(formatted);
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
    let (_, _, backend) = test_backend_factory(190, 50);
    let stdout = Arc::new(Mutex::new(Vec::new()));
    let os_input = os_input_output_stdout(network_frames, 2, Some(stdout.clone()));
    let opts = opts_raw();
    start(backend, os_input, opts);
    let stdout = Arc::try_unwrap(stdout).unwrap().into_inner().unwrap();
    let formatted = format_raw_output(stdout);
    assert_snapshot!(formatted);
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
    let (_, _, backend) = test_backend_factory(190, 50);
    let stdout = Arc::new(Mutex::new(Vec::new()));
    let os_input = os_input_output_stdout(network_frames, 2, Some(stdout.clone()));
    let opts = opts_raw();
    start(backend, os_input, opts);
    let stdout = Arc::try_unwrap(stdout).unwrap().into_inner().unwrap();
    let formatted = format_raw_output(stdout);
    assert_snapshot!(formatted);
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
    let (_, _, backend) = test_backend_factory(190, 50);
    let stdout = Arc::new(Mutex::new(Vec::new()));
    let os_input = os_input_output_stdout(network_frames, 2, Some(stdout.clone()));
    let opts = opts_raw();
    start(backend, os_input, opts);
    let stdout = Arc::try_unwrap(stdout).unwrap().into_inner().unwrap();
    let formatted = format_raw_output(stdout);
    assert_snapshot!(formatted);
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
    let (_, _, backend) = test_backend_factory(190, 50);
    let stdout = Arc::new(Mutex::new(Vec::new()));
    let os_input = os_input_output_stdout(network_frames, 2, Some(stdout.clone()));
    let opts = opts_raw();
    start(backend, os_input, opts);
    let stdout = Arc::try_unwrap(stdout).unwrap().into_inner().unwrap();
    let formatted = format_raw_output(stdout);
    assert_snapshot!(formatted);
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
    let (_, _, backend) = test_backend_factory(190, 50);

    let stdout = Arc::new(Mutex::new(Vec::new()));
    let os_input = os_input_output_stdout(network_frames, 2, Some(stdout.clone()));
    let opts = opts_raw();
    start(backend, os_input, opts);
    let stdout = Arc::try_unwrap(stdout).unwrap().into_inner().unwrap();
    let formatted = format_raw_output(stdout);
    assert_snapshot!(formatted);
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
    let (_, _, backend) = test_backend_factory(190, 50);

    let stdout = Arc::new(Mutex::new(Vec::new()));
    let os_input = os_input_output_stdout(network_frames, 3, Some(stdout.clone()));
    let opts = opts_raw();
    start(backend, os_input, opts);
    let stdout = Arc::try_unwrap(stdout).unwrap().into_inner().unwrap();
    let formatted = format_raw_output(stdout);
    assert_snapshot!(formatted);
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
    let (_, _, backend) = test_backend_factory(190, 50);

    let stdout = Arc::new(Mutex::new(Vec::new()));
    let os_input = os_input_output_stdout(network_frames, 3, Some(stdout.clone()));
    let opts = opts_raw();
    start(backend, os_input, opts);
    let stdout = Arc::try_unwrap(stdout).unwrap().into_inner().unwrap();
    let formatted = format_raw_output(stdout);
    assert_snapshot!(formatted);
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
    let (_, _, backend) = test_backend_factory(190, 50);
    let stdout = Arc::new(Mutex::new(Vec::new()));
    let os_input = os_input_output_stdout(network_frames, 3, Some(stdout.clone()));

    let opts = opts_raw();
    start(backend, os_input, opts);
    let stdout = Arc::try_unwrap(stdout).unwrap().into_inner().unwrap();
    let formatted = format_raw_output(stdout);
    assert_snapshot!(formatted);
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
    let (_, _, backend) = test_backend_factory(190, 50);
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
    let stdout = Arc::new(Mutex::new(Vec::new()));
    let os_input = os_input_output_dns(network_frames, 3, Some(stdout.clone()), dns_client);
    let opts = opts_raw();
    start(backend, os_input, opts);
    let stdout = Arc::try_unwrap(stdout).unwrap().into_inner().unwrap();
    let formatted = format_raw_output(stdout);
    assert_snapshot!(formatted);
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
    let (_, _, backend) = test_backend_factory(190, 50);
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

    let stdout = Arc::new(Mutex::new(Vec::new()));
    let os_input = os_input_output_dns(network_frames, 3, Some(stdout.clone()), None);
    let opts = Opt {
        interface: Some(String::from("interface_name")),
        raw: true,
        no_resolve: true,
        show_dns: false,
        dns_server: None,
        render_opts: RenderOpts {
            addresses: false,
            connections: false,
            processes: false,
            total_utilization: false,
        },
    };
    start(backend, os_input, opts);
    let stdout = Arc::try_unwrap(stdout).unwrap().into_inner().unwrap();
    let formatted = format_raw_output(stdout);
    assert_snapshot!(formatted);
}
