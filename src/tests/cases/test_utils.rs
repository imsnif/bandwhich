use crate::tests::fakes::{
    create_fake_dns_client, create_fake_on_winch, get_interfaces, get_open_sockets, KeyboardEvents,
};
use std::iter;

use crate::network::dns::Client;
use crate::OsInputOutput;
use ::termion::event::{Event, Key};
use pnet::datalink::DataLinkReceiver;
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