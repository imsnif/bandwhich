use crate::tests::fakes::{
    create_fake_dns_client, create_fake_on_winch, get_interfaces, get_open_sockets, KeyboardEvents,
    TerminalEvent, TestBackend,
};
use std::iter;

use crate::network::dns::Client;
use crate::{Opt, OsInputOutput};
use ::termion::event::{Event, Key};
use pnet_bandwhich_fork::datalink::DataLinkReceiver;
use std::collections::HashMap;
use std::io::Write;
use std::sync::{Arc, Mutex};

pub fn sleep_and_quit_events(sleep_num: usize) -> Box<KeyboardEvents> {
    let mut events: Vec<Option<Event>> = iter::repeat(None).take(sleep_num).collect();
    events.push(Some(Event::Key(Key::Ctrl('c'))));
    Box::new(KeyboardEvents::new(events))
}

pub fn os_input_output(
    network_frames: Vec<Box<dyn DataLinkReceiver>>,
    sleep_num: usize,
) -> OsInputOutput {
    os_input_output_factory(
        network_frames,
        sleep_num,
        None,
        create_fake_dns_client(HashMap::new()),
    )
}
pub fn os_input_output_stdout(
    network_frames: Vec<Box<dyn DataLinkReceiver>>,
    sleep_num: usize,
    stdout: Option<Arc<Mutex<Vec<u8>>>>,
) -> OsInputOutput {
    os_input_output_factory(
        network_frames,
        sleep_num,
        stdout,
        create_fake_dns_client(HashMap::new()),
    )
}

pub fn os_input_output_dns(
    network_frames: Vec<Box<dyn DataLinkReceiver>>,
    sleep_num: usize,
    stdout: Option<Arc<Mutex<Vec<u8>>>>,
    dns_client: Option<Client>,
) -> OsInputOutput {
    os_input_output_factory(network_frames, sleep_num, stdout, dns_client)
}

fn os_input_output_factory(
    network_frames: Vec<Box<dyn DataLinkReceiver>>,
    sleep_num: usize,
    stdout: Option<Arc<Mutex<Vec<u8>>>>,
    dns_client: Option<Client>,
) -> OsInputOutput {
    let on_winch = create_fake_on_winch(false);
    let cleanup = Box::new(|| {});

    let write_to_stdout: Box<dyn FnMut(String) + Send> = match stdout {
        Some(stdout) => Box::new({
            move |output: String| {
                let mut stdout = stdout.lock().unwrap();
                writeln!(&mut stdout, "{}", output).unwrap();
            }
        }),
        None => Box::new({ move |_output: String| {} }),
    };

    OsInputOutput {
        network_interfaces: get_interfaces(),
        network_frames,
        get_open_sockets,
        keyboard_events: sleep_and_quit_events(sleep_num),
        dns_client,
        on_winch,
        cleanup,
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
