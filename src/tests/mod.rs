mod fakes;

use fakes::TerminalEvent::*;
use fakes::{
    create_fake_lookup_addr, create_fake_on_winch, get_interface, get_open_sockets, KeyboardEvents,
    NetworkFrames, TestBackend,
};

use ::insta::assert_snapshot;
use ::std::sync::{Arc, Mutex};
use ::termion::event::{Event, Key};

use ::std::collections::HashMap;
use ::std::net::IpAddr;

use packet_builder::payload::PayloadData;
use packet_builder::*;
use pnet::packet::Packet;
use pnet_base::MacAddr;

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

struct LogWithMirror<T> {
    pub write: Arc<Mutex<T>>,
    pub mirror: Arc<Mutex<T>>,
}

impl<T> LogWithMirror<T> {
    pub fn new(log: T) -> Self {
        let write = Arc::new(Mutex::new(log));
        let mirror = write.clone();
        LogWithMirror { write, mirror }
    }
}

#[test]
fn basic_startup() {
    let keyboard_events = Box::new(KeyboardEvents::new(vec![
        None, // sleep
        Some(Event::Key(Key::Ctrl('c'))),
    ]));
    let network_frames = NetworkFrames::new(vec![
        None, // sleep
    ]);

    let terminal_width = Arc::new(Mutex::new(190));
    let terminal_height = Arc::new(Mutex::new(50));
    let terminal_events = LogWithMirror::new(Vec::new());
    let terminal_draw_events = LogWithMirror::new(Vec::new());

    let backend = TestBackend::new(
        terminal_events.write,
        terminal_draw_events.write,
        terminal_width,
        terminal_height,
    );
    let network_interface = get_interface();
    let lookup_addr = create_fake_lookup_addr(HashMap::new());
    let on_winch = create_fake_on_winch(false);
    let cleanup = Box::new(|| {});

    let os_input = crate::OsInput {
        network_interface,
        network_frames,
        get_open_sockets,
        keyboard_events,
        lookup_addr,
        on_winch,
        cleanup,
    };
    crate::start(backend, os_input);

    let terminal_events_mirror = terminal_events.mirror.lock().unwrap();
    let terminal_draw_events_mirror = terminal_draw_events.mirror.lock().unwrap();

    let expected_terminal_events = vec![Clear, HideCursor, Draw, Flush, Clear, ShowCursor];
    assert_eq!(&terminal_events_mirror[..], &expected_terminal_events[..]);

    assert_eq!(terminal_draw_events_mirror.len(), 1);
    assert_snapshot!(&terminal_draw_events_mirror[0]);
}

#[test]
fn one_packet_of_traffic() {
    let keyboard_events = Box::new(KeyboardEvents::new(vec![
        None, // sleep
        None, // sleep
        Some(Event::Key(Key::Ctrl('c'))),
    ]));
    let network_frames = NetworkFrames::new(vec![Some(build_tcp_packet(
        "10.0.0.2",
        "1.1.1.1",
        443,
        12345,
        b"I am a fake tcp packet",
    ))]);

    let terminal_width = Arc::new(Mutex::new(190));
    let terminal_height = Arc::new(Mutex::new(50));
    let terminal_events = LogWithMirror::new(Vec::new());
    let terminal_draw_events = LogWithMirror::new(Vec::new());

    let backend = TestBackend::new(
        terminal_events.write,
        terminal_draw_events.write,
        terminal_width,
        terminal_height,
    );
    let network_interface = get_interface();
    let lookup_addr = create_fake_lookup_addr(HashMap::new());
    let on_winch = create_fake_on_winch(false);
    let cleanup = Box::new(|| {});

    let os_input = crate::OsInput {
        network_interface,
        network_frames,
        get_open_sockets,
        keyboard_events,
        lookup_addr,
        on_winch,
        cleanup,
    };
    crate::start(backend, os_input);

    let terminal_events_mirror = terminal_events.mirror.lock().unwrap();
    let terminal_draw_events_mirror = terminal_draw_events.mirror.lock().unwrap();

    let expected_terminal_events = vec![
        Clear, HideCursor, Draw, Flush, Draw, Flush, Clear, ShowCursor,
    ];
    assert_eq!(&terminal_events_mirror[..], &expected_terminal_events[..]);

    assert_eq!(terminal_draw_events_mirror.len(), 2);
    assert_snapshot!(&terminal_draw_events_mirror[0]);
    assert_snapshot!(&terminal_draw_events_mirror[1]);
}

#[test]
fn bi_directional_traffic() {
    let keyboard_events = Box::new(KeyboardEvents::new(vec![
        None, // sleep
        None, // sleep
        Some(Event::Key(Key::Ctrl('c'))),
    ]));
    let network_frames = NetworkFrames::new(vec![
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
    ]);

    let terminal_width = Arc::new(Mutex::new(190));
    let terminal_height = Arc::new(Mutex::new(50));
    let terminal_events = LogWithMirror::new(Vec::new());
    let terminal_draw_events = LogWithMirror::new(Vec::new());

    let backend = TestBackend::new(
        terminal_events.write,
        terminal_draw_events.write,
        terminal_width,
        terminal_height,
    );
    let network_interface = get_interface();
    let lookup_addr = create_fake_lookup_addr(HashMap::new());
    let on_winch = create_fake_on_winch(false);
    let cleanup = Box::new(|| {});

    let os_input = crate::OsInput {
        network_interface,
        network_frames,
        get_open_sockets,
        keyboard_events,
        lookup_addr,
        on_winch,
        cleanup,
    };
    crate::start(backend, os_input);

    let terminal_events_mirror = terminal_events.mirror.lock().unwrap();
    let terminal_draw_events_mirror = terminal_draw_events.mirror.lock().unwrap();

    let expected_terminal_events = vec![
        Clear, HideCursor, Draw, Flush, Draw, Flush, Clear, ShowCursor,
    ];
    assert_eq!(&terminal_events_mirror[..], &expected_terminal_events[..]);

    assert_eq!(terminal_draw_events_mirror.len(), 2);
    assert_snapshot!(&terminal_draw_events_mirror[0]);
    assert_snapshot!(&terminal_draw_events_mirror[1]);
}

#[test]
fn multiple_packets_of_traffic_from_different_connections() {
    let keyboard_events = Box::new(KeyboardEvents::new(vec![
        None, // sleep
        None, // sleep
        Some(Event::Key(Key::Ctrl('c'))),
    ]));
    let network_frames = NetworkFrames::new(vec![
        Some(build_tcp_packet(
            "1.1.1.1",
            "10.0.0.2",
            12345,
            443,
            b"I have come from 1.1.1.1",
        )),
        Some(build_tcp_packet(
            "2.2.2.2",
            "10.0.0.2",
            54321,
            443,
            b"I come from 2.2.2.2",
        )),
    ]);

    let terminal_width = Arc::new(Mutex::new(190));
    let terminal_height = Arc::new(Mutex::new(50));
    let terminal_events = LogWithMirror::new(Vec::new());
    let terminal_draw_events = LogWithMirror::new(Vec::new());

    let backend = TestBackend::new(
        terminal_events.write,
        terminal_draw_events.write,
        terminal_width,
        terminal_height,
    );
    let on_winch = create_fake_on_winch(false);
    let cleanup = Box::new(|| {});
    let network_interface = get_interface();
    let lookup_addr = create_fake_lookup_addr(HashMap::new());

    let os_input = crate::OsInput {
        network_interface,
        network_frames,
        get_open_sockets,
        on_winch,
        cleanup,
        keyboard_events,
        lookup_addr,
    };
    crate::start(backend, os_input);

    let terminal_events_mirror = terminal_events.mirror.lock().unwrap();
    let terminal_draw_events_mirror = terminal_draw_events.mirror.lock().unwrap();

    let expected_terminal_events = vec![
        Clear, HideCursor, Draw, Flush, Draw, Flush, Clear, ShowCursor,
    ];
    assert_eq!(&terminal_events_mirror[..], &expected_terminal_events[..]);

    assert_eq!(terminal_draw_events_mirror.len(), 2);
    assert_snapshot!(&terminal_draw_events_mirror[0]);
    assert_snapshot!(&terminal_draw_events_mirror[1]);
}

#[test]
fn multiple_packets_of_traffic_from_single_connection() {
    let keyboard_events = Box::new(KeyboardEvents::new(vec![
        None, // sleep
        None, // sleep
        Some(Event::Key(Key::Ctrl('c'))),
    ]));
    let network_frames = NetworkFrames::new(vec![
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
    ]);

    let terminal_width = Arc::new(Mutex::new(190));
    let terminal_height = Arc::new(Mutex::new(50));
    let terminal_events = LogWithMirror::new(Vec::new());
    let terminal_draw_events = LogWithMirror::new(Vec::new());

    let backend = TestBackend::new(
        terminal_events.write,
        terminal_draw_events.write,
        terminal_width,
        terminal_height,
    );
    let network_interface = get_interface();
    let lookup_addr = create_fake_lookup_addr(HashMap::new());
    let on_winch = create_fake_on_winch(false);
    let cleanup = Box::new(|| {});

    let os_input = crate::OsInput {
        network_interface,
        network_frames,
        get_open_sockets,
        keyboard_events,
        lookup_addr,
        on_winch,
        cleanup,
    };
    crate::start(backend, os_input);

    let terminal_events_mirror = terminal_events.mirror.lock().unwrap();
    let terminal_draw_events_mirror = terminal_draw_events.mirror.lock().unwrap();

    let expected_terminal_events = vec![
        Clear, HideCursor, Draw, Flush, Draw, Flush, Clear, ShowCursor,
    ];
    assert_eq!(&terminal_events_mirror[..], &expected_terminal_events[..]);

    assert_eq!(terminal_draw_events_mirror.len(), 2);
    assert_snapshot!(&terminal_draw_events_mirror[0]);
    assert_snapshot!(&terminal_draw_events_mirror[1]);
}

#[test]
fn one_process_with_multiple_connections() {
    let keyboard_events = Box::new(KeyboardEvents::new(vec![
        None, // sleep
        None, // sleep
        Some(Event::Key(Key::Ctrl('c'))),
    ]));
    let network_frames = NetworkFrames::new(vec![
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
            443,
            b"Funny that, I'm from 3.3.3.3",
        )),
    ]);

    let terminal_width = Arc::new(Mutex::new(190));
    let terminal_height = Arc::new(Mutex::new(50));
    let terminal_events = LogWithMirror::new(Vec::new());
    let terminal_draw_events = LogWithMirror::new(Vec::new());

    let backend = TestBackend::new(
        terminal_events.write,
        terminal_draw_events.write,
        terminal_width,
        terminal_height,
    );
    let network_interface = get_interface();
    let lookup_addr = create_fake_lookup_addr(HashMap::new());
    let on_winch = create_fake_on_winch(false);
    let cleanup = Box::new(|| {});

    let os_input = crate::OsInput {
        network_interface,
        network_frames,
        get_open_sockets,
        keyboard_events,
        lookup_addr,
        on_winch,
        cleanup,
    };
    crate::start(backend, os_input);

    let terminal_events_mirror = terminal_events.mirror.lock().unwrap();
    let terminal_draw_events_mirror = terminal_draw_events.mirror.lock().unwrap();

    let expected_terminal_events = vec![
        Clear, HideCursor, Draw, Flush, Draw, Flush, Clear, ShowCursor,
    ];
    assert_eq!(&terminal_events_mirror[..], &expected_terminal_events[..]);

    assert_eq!(terminal_draw_events_mirror.len(), 2);
    assert_snapshot!(&terminal_draw_events_mirror[0]);
    assert_snapshot!(&terminal_draw_events_mirror[1]);
}

#[test]
fn multiple_processes_with_multiple_connections() {
    let keyboard_events = Box::new(KeyboardEvents::new(vec![
        None, // sleep
        None, // sleep
        Some(Event::Key(Key::Ctrl('c'))),
    ]));
    let network_frames = NetworkFrames::new(vec![
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
            443,
            b"Awesome, I'm from 3.3.3.3",
        )),
        Some(build_tcp_packet(
            "2.2.2.2",
            "10.0.0.2",
            54321,
            443,
            b"You know, 2.2.2.2 is really nice!",
        )),
        Some(build_tcp_packet(
            "4.4.4.4",
            "10.0.0.2",
            1337,
            443,
            b"I'm partial to 4.4.4.4",
        )),
    ]);

    let terminal_width = Arc::new(Mutex::new(190));
    let terminal_height = Arc::new(Mutex::new(50));
    let terminal_events = LogWithMirror::new(Vec::new());
    let terminal_draw_events = LogWithMirror::new(Vec::new());

    let backend = TestBackend::new(
        terminal_events.write,
        terminal_draw_events.write,
        terminal_width,
        terminal_height,
    );
    let network_interface = get_interface();
    let lookup_addr = create_fake_lookup_addr(HashMap::new());
    let on_winch = create_fake_on_winch(false);
    let cleanup = Box::new(|| {});

    let os_input = crate::OsInput {
        network_interface,
        network_frames,
        get_open_sockets,
        keyboard_events,
        lookup_addr,
        on_winch,
        cleanup,
    };
    crate::start(backend, os_input);

    let terminal_events_mirror = terminal_events.mirror.lock().unwrap();
    let terminal_draw_events_mirror = terminal_draw_events.mirror.lock().unwrap();

    let expected_terminal_events = vec![
        Clear, HideCursor, Draw, Flush, Draw, Flush, Clear, ShowCursor,
    ];
    assert_eq!(&terminal_events_mirror[..], &expected_terminal_events[..]);

    assert_eq!(terminal_draw_events_mirror.len(), 2);
    assert_snapshot!(&terminal_draw_events_mirror[0]);
    assert_snapshot!(&terminal_draw_events_mirror[1]);
}

#[test]
fn multiple_connections_from_remote_ip() {
    let keyboard_events = Box::new(KeyboardEvents::new(vec![
        None, // sleep
        None, // sleep
        Some(Event::Key(Key::Ctrl('c'))),
    ]));
    let network_frames = NetworkFrames::new(vec![
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
    ]);

    let terminal_width = Arc::new(Mutex::new(190));
    let terminal_height = Arc::new(Mutex::new(50));
    let terminal_events = LogWithMirror::new(Vec::new());
    let terminal_draw_events = LogWithMirror::new(Vec::new());

    let backend = TestBackend::new(
        terminal_events.write,
        terminal_draw_events.write,
        terminal_width,
        terminal_height,
    );
    let network_interface = get_interface();
    let lookup_addr = create_fake_lookup_addr(HashMap::new());
    let on_winch = create_fake_on_winch(false);
    let cleanup = Box::new(|| {});

    let os_input = crate::OsInput {
        network_interface,
        network_frames,
        get_open_sockets,
        keyboard_events,
        lookup_addr,
        on_winch,
        cleanup,
    };
    crate::start(backend, os_input);

    let terminal_events_mirror = terminal_events.mirror.lock().unwrap();
    let terminal_draw_events_mirror = terminal_draw_events.mirror.lock().unwrap();

    let expected_terminal_events = vec![
        Clear, HideCursor, Draw, Flush, Draw, Flush, Clear, ShowCursor,
    ];
    assert_eq!(&terminal_events_mirror[..], &expected_terminal_events[..]);

    assert_eq!(terminal_draw_events_mirror.len(), 2);
    assert_snapshot!(&terminal_draw_events_mirror[0]);
    assert_snapshot!(&terminal_draw_events_mirror[1]);
}

#[test]
fn sustained_traffic_from_one_process() {
    let keyboard_events = Box::new(KeyboardEvents::new(vec![
        None, // sleep
        None, // sleep
        None, // sleep
        Some(Event::Key(Key::Ctrl('c'))),
    ]));
    let network_frames = NetworkFrames::new(vec![
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
    ]);

    let terminal_width = Arc::new(Mutex::new(190));
    let terminal_height = Arc::new(Mutex::new(50));
    let terminal_events = LogWithMirror::new(Vec::new());
    let terminal_draw_events = LogWithMirror::new(Vec::new());

    let backend = TestBackend::new(
        terminal_events.write,
        terminal_draw_events.write,
        terminal_width,
        terminal_height,
    );
    let network_interface = get_interface();
    let lookup_addr = create_fake_lookup_addr(HashMap::new());
    let on_winch = create_fake_on_winch(false);
    let cleanup = Box::new(|| {});

    let os_input = crate::OsInput {
        network_interface,
        network_frames,
        get_open_sockets,
        keyboard_events,
        lookup_addr,
        on_winch,
        cleanup,
    };
    crate::start(backend, os_input);

    let terminal_events_mirror = terminal_events.mirror.lock().unwrap();
    let terminal_draw_events_mirror = terminal_draw_events.mirror.lock().unwrap();

    let expected_terminal_events = vec![
        Clear, HideCursor, Draw, Flush, Draw, Flush, Draw, Flush, Clear, ShowCursor,
    ];
    assert_eq!(&terminal_events_mirror[..], &expected_terminal_events[..]);

    assert_eq!(terminal_draw_events_mirror.len(), 3);
    assert_snapshot!(&terminal_draw_events_mirror[1]);
    assert_snapshot!(&terminal_draw_events_mirror[2]);
}

#[test]
fn sustained_traffic_from_multiple_processes() {
    let keyboard_events = Box::new(KeyboardEvents::new(vec![
        None, // sleep
        None, // sleep
        None, // sleep
        Some(Event::Key(Key::Ctrl('c'))),
    ]));
    let network_frames = NetworkFrames::new(vec![
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
            443,
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
            443,
            b"I come 3.3.3.3 one second later",
        )),
    ]);

    let terminal_width = Arc::new(Mutex::new(190));
    let terminal_height = Arc::new(Mutex::new(50));
    let terminal_events = LogWithMirror::new(Vec::new());
    let terminal_draw_events = LogWithMirror::new(Vec::new());

    let backend = TestBackend::new(
        terminal_events.write,
        terminal_draw_events.write,
        terminal_width,
        terminal_height,
    );
    let network_interface = get_interface();
    let lookup_addr = create_fake_lookup_addr(HashMap::new());
    let on_winch = create_fake_on_winch(false);
    let cleanup = Box::new(|| {});

    let os_input = crate::OsInput {
        network_interface,
        network_frames,
        get_open_sockets,
        keyboard_events,
        lookup_addr,
        on_winch,
        cleanup,
    };
    crate::start(backend, os_input);

    let terminal_events_mirror = terminal_events.mirror.lock().unwrap();
    let terminal_draw_events_mirror = terminal_draw_events.mirror.lock().unwrap();

    let expected_terminal_events = vec![
        Clear, HideCursor, Draw, Flush, Draw, Flush, Draw, Flush, Clear, ShowCursor,
    ];
    assert_eq!(&terminal_events_mirror[..], &expected_terminal_events[..]);

    assert_eq!(terminal_draw_events_mirror.len(), 3);
    assert_snapshot!(&terminal_draw_events_mirror[1]);
    assert_snapshot!(&terminal_draw_events_mirror[2]);
}

#[test]
fn sustained_traffic_from_multiple_processes_bi_directional() {
    let keyboard_events = Box::new(KeyboardEvents::new(vec![
        None, // sleep
        None, // sleep
        None, // sleep
        Some(Event::Key(Key::Ctrl('c'))),
    ]));
    let network_frames = NetworkFrames::new(vec![
        Some(build_tcp_packet(
            "10.0.0.2",
            "3.3.3.3",
            443,
            1337,
            b"omw to 3.3.3.3",
        )),
        Some(build_tcp_packet(
            "3.3.3.3",
            "10.0.0.2",
            1337,
            443,
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
            443,
            1337,
            b"Wait for me!",
        )),
        Some(build_tcp_packet(
            "3.3.3.3",
            "10.0.0.2",
            1337,
            443,
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
    ]);

    let terminal_width = Arc::new(Mutex::new(190));
    let terminal_height = Arc::new(Mutex::new(50));
    let terminal_events = LogWithMirror::new(Vec::new());
    let terminal_draw_events = LogWithMirror::new(Vec::new());

    let backend = TestBackend::new(
        terminal_events.write,
        terminal_draw_events.write,
        terminal_width,
        terminal_height,
    );
    let network_interface = get_interface();
    let lookup_addr = create_fake_lookup_addr(HashMap::new());
    let on_winch = create_fake_on_winch(false);
    let cleanup = Box::new(|| {});

    let os_input = crate::OsInput {
        network_interface,
        network_frames,
        get_open_sockets,
        keyboard_events,
        lookup_addr,
        on_winch,
        cleanup,
    };
    crate::start(backend, os_input);

    let terminal_events_mirror = terminal_events.mirror.lock().unwrap();
    let terminal_draw_events_mirror = terminal_draw_events.mirror.lock().unwrap();

    let expected_terminal_events = vec![
        Clear, HideCursor, Draw, Flush, Draw, Flush, Draw, Flush, Clear, ShowCursor,
    ];
    assert_eq!(&terminal_events_mirror[..], &expected_terminal_events[..]);

    assert_eq!(terminal_draw_events_mirror.len(), 3);
    assert_snapshot!(&terminal_draw_events_mirror[1]);
    assert_snapshot!(&terminal_draw_events_mirror[2]);
}

#[test]
fn traffic_with_host_names() {
    let keyboard_events = Box::new(KeyboardEvents::new(vec![
        None, // sleep
        None, // sleep
        None, // sleep
        Some(Event::Key(Key::Ctrl('c'))),
    ]));
    let network_frames = NetworkFrames::new(vec![
        Some(build_tcp_packet(
            "10.0.0.2",
            "3.3.3.3",
            443,
            1337,
            b"omw to 3.3.3.3",
        )),
        Some(build_tcp_packet(
            "3.3.3.3",
            "10.0.0.2",
            1337,
            443,
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
            443,
            1337,
            b"Wait for me!",
        )),
        Some(build_tcp_packet(
            "3.3.3.3",
            "10.0.0.2",
            1337,
            443,
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
    ]);

    let terminal_width = Arc::new(Mutex::new(190));
    let terminal_height = Arc::new(Mutex::new(50));
    let terminal_events = LogWithMirror::new(Vec::new());
    let terminal_draw_events = LogWithMirror::new(Vec::new());

    let backend = TestBackend::new(
        terminal_events.write,
        terminal_draw_events.write,
        terminal_width,
        terminal_height,
    );
    let network_interface = get_interface();
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
    let lookup_addr = create_fake_lookup_addr(ips_to_hostnames);
    let on_winch = create_fake_on_winch(false);
    let cleanup = Box::new(|| {});

    let os_input = crate::OsInput {
        network_interface,
        network_frames,
        get_open_sockets,
        keyboard_events,
        lookup_addr,
        on_winch,
        cleanup,
    };
    crate::start(backend, os_input);

    let terminal_events_mirror = terminal_events.mirror.lock().unwrap();
    let terminal_draw_events_mirror = terminal_draw_events.mirror.lock().unwrap();

    let expected_terminal_events = vec![
        Clear, HideCursor, Draw, Flush, Draw, Flush, Draw, Flush, Clear, ShowCursor,
    ];
    assert_eq!(&terminal_events_mirror[..], &expected_terminal_events[..]);

    assert_eq!(terminal_draw_events_mirror.len(), 3);
    assert_snapshot!(&terminal_draw_events_mirror[1]);
    assert_snapshot!(&terminal_draw_events_mirror[2]);
}

#[test]
fn traffic_with_winch_event() {
    let keyboard_events = Box::new(KeyboardEvents::new(vec![
        None, // sleep
        None, // sleep
        Some(Event::Key(Key::Ctrl('c'))),
    ]));
    let network_frames = NetworkFrames::new(vec![Some(build_tcp_packet(
        "10.0.0.2",
        "1.1.1.1",
        443,
        12345,
        b"I am a fake tcp packet",
    ))]);

    let terminal_width = Arc::new(Mutex::new(190));
    let terminal_height = Arc::new(Mutex::new(50));
    let terminal_events = LogWithMirror::new(Vec::new());
    let terminal_draw_events = LogWithMirror::new(Vec::new());

    let backend = TestBackend::new(
        terminal_events.write,
        terminal_draw_events.write,
        terminal_width,
        terminal_height,
    );
    let network_interface = get_interface();
    let lookup_addr = create_fake_lookup_addr(HashMap::new());
    let on_winch = create_fake_on_winch(true);
    let cleanup = Box::new(|| {});

    let os_input = crate::OsInput {
        network_interface,
        network_frames,
        get_open_sockets,
        keyboard_events,
        lookup_addr,
        on_winch,
        cleanup,
    };
    crate::start(backend, os_input);

    let terminal_events_mirror = terminal_events.mirror.lock().unwrap();
    let terminal_draw_events_mirror = terminal_draw_events.mirror.lock().unwrap();

    let expected_terminal_events = vec![
        Clear, HideCursor, Draw, Flush, Draw, Flush, Draw, Flush, Clear, ShowCursor,
    ];
    assert_eq!(&terminal_events_mirror[..], &expected_terminal_events[..]);

    assert_eq!(terminal_draw_events_mirror.len(), 3);
    assert_snapshot!(&terminal_draw_events_mirror[0]);
    assert_snapshot!(&terminal_draw_events_mirror[1]);
    assert_snapshot!(&terminal_draw_events_mirror[2]);
}

#[test]
fn layout_full_width_under_30_height() {
    let keyboard_events = Box::new(KeyboardEvents::new(vec![
        None, // sleep
        None, // sleep
        Some(Event::Key(Key::Ctrl('c'))),
    ]));
    let network_frames = NetworkFrames::new(vec![
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
            443,
            b"Awesome, I'm from 3.3.3.3",
        )),
        Some(build_tcp_packet(
            "2.2.2.2",
            "10.0.0.2",
            54321,
            443,
            b"You know, 2.2.2.2 is really nice!",
        )),
        Some(build_tcp_packet(
            "4.4.4.4",
            "10.0.0.2",
            1337,
            443,
            b"I'm partial to 4.4.4.4",
        )),
    ]);

    let terminal_width = Arc::new(Mutex::new(190));
    let terminal_height = Arc::new(Mutex::new(29));
    let terminal_events = LogWithMirror::new(Vec::new());
    let terminal_draw_events = LogWithMirror::new(Vec::new());

    let backend = TestBackend::new(
        terminal_events.write,
        terminal_draw_events.write,
        terminal_width,
        terminal_height,
    );
    let network_interface = get_interface();
    let lookup_addr = create_fake_lookup_addr(HashMap::new());
    let on_winch = create_fake_on_winch(false);
    let cleanup = Box::new(|| {});

    let os_input = crate::OsInput {
        network_interface,
        network_frames,
        get_open_sockets,
        keyboard_events,
        lookup_addr,
        on_winch,
        cleanup,
    };
    crate::start(backend, os_input);

    let terminal_events_mirror = terminal_events.mirror.lock().unwrap();
    let terminal_draw_events_mirror = terminal_draw_events.mirror.lock().unwrap();

    let expected_terminal_events = vec![
        Clear, HideCursor, Draw, Flush, Draw, Flush, Clear, ShowCursor,
    ];
    assert_eq!(&terminal_events_mirror[..], &expected_terminal_events[..]);

    assert_eq!(terminal_draw_events_mirror.len(), 2);
    assert_snapshot!(&terminal_draw_events_mirror[0]);
    assert_snapshot!(&terminal_draw_events_mirror[1]);
}

#[test]
fn layout_under_150_width_full_height() {
    let keyboard_events = Box::new(KeyboardEvents::new(vec![
        None, // sleep
        None, // sleep
        Some(Event::Key(Key::Ctrl('c'))),
    ]));
    let network_frames = NetworkFrames::new(vec![
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
            443,
            b"Awesome, I'm from 3.3.3.3",
        )),
        Some(build_tcp_packet(
            "2.2.2.2",
            "10.0.0.2",
            54321,
            443,
            b"You know, 2.2.2.2 is really nice!",
        )),
        Some(build_tcp_packet(
            "4.4.4.4",
            "10.0.0.2",
            1337,
            443,
            b"I'm partial to 4.4.4.4",
        )),
    ]);

    let terminal_width = Arc::new(Mutex::new(149));
    let terminal_height = Arc::new(Mutex::new(50));
    let terminal_events = LogWithMirror::new(Vec::new());
    let terminal_draw_events = LogWithMirror::new(Vec::new());

    let backend = TestBackend::new(
        terminal_events.write,
        terminal_draw_events.write,
        terminal_width,
        terminal_height,
    );
    let network_interface = get_interface();
    let lookup_addr = create_fake_lookup_addr(HashMap::new());
    let on_winch = create_fake_on_winch(false);
    let cleanup = Box::new(|| {});

    let os_input = crate::OsInput {
        network_interface,
        network_frames,
        get_open_sockets,
        keyboard_events,
        lookup_addr,
        on_winch,
        cleanup,
    };
    crate::start(backend, os_input);

    let terminal_events_mirror = terminal_events.mirror.lock().unwrap();
    let terminal_draw_events_mirror = terminal_draw_events.mirror.lock().unwrap();

    let expected_terminal_events = vec![
        Clear, HideCursor, Draw, Flush, Draw, Flush, Clear, ShowCursor,
    ];
    assert_eq!(&terminal_events_mirror[..], &expected_terminal_events[..]);

    assert_eq!(terminal_draw_events_mirror.len(), 2);
    assert_snapshot!(&terminal_draw_events_mirror[0]);
    assert_snapshot!(&terminal_draw_events_mirror[1]);
}

#[test]
fn layout_under_150_width_under_30_height() {
    let keyboard_events = Box::new(KeyboardEvents::new(vec![
        None, // sleep
        None, // sleep
        Some(Event::Key(Key::Ctrl('c'))),
    ]));
    let network_frames = NetworkFrames::new(vec![
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
            443,
            b"Awesome, I'm from 3.3.3.3",
        )),
        Some(build_tcp_packet(
            "2.2.2.2",
            "10.0.0.2",
            54321,
            443,
            b"You know, 2.2.2.2 is really nice!",
        )),
        Some(build_tcp_packet(
            "4.4.4.4",
            "10.0.0.2",
            1337,
            443,
            b"I'm partial to 4.4.4.4",
        )),
    ]);

    let terminal_width = Arc::new(Mutex::new(149));
    let terminal_height = Arc::new(Mutex::new(29));
    let terminal_events = LogWithMirror::new(Vec::new());
    let terminal_draw_events = LogWithMirror::new(Vec::new());

    let backend = TestBackend::new(
        terminal_events.write,
        terminal_draw_events.write,
        terminal_width,
        terminal_height,
    );
    let network_interface = get_interface();
    let lookup_addr = create_fake_lookup_addr(HashMap::new());
    let on_winch = create_fake_on_winch(false);
    let cleanup = Box::new(|| {});

    let os_input = crate::OsInput {
        network_interface,
        network_frames,
        get_open_sockets,
        keyboard_events,
        lookup_addr,
        on_winch,
        cleanup,
    };
    crate::start(backend, os_input);

    let terminal_events_mirror = terminal_events.mirror.lock().unwrap();
    let terminal_draw_events_mirror = terminal_draw_events.mirror.lock().unwrap();

    let expected_terminal_events = vec![
        Clear, HideCursor, Draw, Flush, Draw, Flush, Clear, ShowCursor,
    ];
    assert_eq!(&terminal_events_mirror[..], &expected_terminal_events[..]);

    assert_eq!(terminal_draw_events_mirror.len(), 2);
    assert_snapshot!(&terminal_draw_events_mirror[0]);
    assert_snapshot!(&terminal_draw_events_mirror[1]);
}

#[test]
fn layout_under_120_width_full_height() {
    let keyboard_events = Box::new(KeyboardEvents::new(vec![
        None, // sleep
        None, // sleep
        Some(Event::Key(Key::Ctrl('c'))),
    ]));
    let network_frames = NetworkFrames::new(vec![
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
            443,
            b"Awesome, I'm from 3.3.3.3",
        )),
        Some(build_tcp_packet(
            "2.2.2.2",
            "10.0.0.2",
            54321,
            443,
            b"You know, 2.2.2.2 is really nice!",
        )),
        Some(build_tcp_packet(
            "4.4.4.4",
            "10.0.0.2",
            1337,
            443,
            b"I'm partial to 4.4.4.4",
        )),
    ]);

    let terminal_width = Arc::new(Mutex::new(119));
    let terminal_height = Arc::new(Mutex::new(50));
    let terminal_events = LogWithMirror::new(Vec::new());
    let terminal_draw_events = LogWithMirror::new(Vec::new());

    let backend = TestBackend::new(
        terminal_events.write,
        terminal_draw_events.write,
        terminal_width,
        terminal_height,
    );
    let network_interface = get_interface();
    let lookup_addr = create_fake_lookup_addr(HashMap::new());
    let on_winch = create_fake_on_winch(false);
    let cleanup = Box::new(|| {});

    let os_input = crate::OsInput {
        network_interface,
        network_frames,
        get_open_sockets,
        keyboard_events,
        lookup_addr,
        on_winch,
        cleanup,
    };
    crate::start(backend, os_input);

    let terminal_events_mirror = terminal_events.mirror.lock().unwrap();
    let terminal_draw_events_mirror = terminal_draw_events.mirror.lock().unwrap();

    let expected_terminal_events = vec![
        Clear, HideCursor, Draw, Flush, Draw, Flush, Clear, ShowCursor,
    ];
    assert_eq!(&terminal_events_mirror[..], &expected_terminal_events[..]);

    assert_eq!(terminal_draw_events_mirror.len(), 2);
    assert_snapshot!(&terminal_draw_events_mirror[0]);
    assert_snapshot!(&terminal_draw_events_mirror[1]);
}

#[test]
fn layout_under_120_width_under_30_height() {
    let keyboard_events = Box::new(KeyboardEvents::new(vec![
        None, // sleep
        None, // sleep
        Some(Event::Key(Key::Ctrl('c'))),
    ]));
    let network_frames = NetworkFrames::new(vec![
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
            443,
            b"Awesome, I'm from 3.3.3.3",
        )),
        Some(build_tcp_packet(
            "2.2.2.2",
            "10.0.0.2",
            54321,
            443,
            b"You know, 2.2.2.2 is really nice!",
        )),
        Some(build_tcp_packet(
            "4.4.4.4",
            "10.0.0.2",
            1337,
            443,
            b"I'm partial to 4.4.4.4",
        )),
    ]);

    let terminal_width = Arc::new(Mutex::new(119));
    let terminal_height = Arc::new(Mutex::new(29));
    let terminal_events = LogWithMirror::new(Vec::new());
    let terminal_draw_events = LogWithMirror::new(Vec::new());

    let backend = TestBackend::new(
        terminal_events.write,
        terminal_draw_events.write,
        terminal_width,
        terminal_height,
    );
    let network_interface = get_interface();
    let lookup_addr = create_fake_lookup_addr(HashMap::new());
    let on_winch = create_fake_on_winch(false);
    let cleanup = Box::new(|| {});

    let os_input = crate::OsInput {
        network_interface,
        network_frames,
        get_open_sockets,
        keyboard_events,
        lookup_addr,
        on_winch,
        cleanup,
    };
    crate::start(backend, os_input);

    let terminal_events_mirror = terminal_events.mirror.lock().unwrap();
    let terminal_draw_events_mirror = terminal_draw_events.mirror.lock().unwrap();

    let expected_terminal_events = vec![
        Clear, HideCursor, Draw, Flush, Draw, Flush, Clear, ShowCursor,
    ];
    assert_eq!(&terminal_events_mirror[..], &expected_terminal_events[..]);

    assert_eq!(terminal_draw_events_mirror.len(), 2);
    assert_snapshot!(&terminal_draw_events_mirror[0]);
    assert_snapshot!(&terminal_draw_events_mirror[1]);
}
