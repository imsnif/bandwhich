mod os;

use ::std::io;
use ::termion::raw::IntoRawMode;
use ::tui::backend::TermionBackend;

fn main () {

    #[cfg(not(target_os = "linux"))]
    panic!("Sorry, no implementations for platforms other than linux yet :( - PRs welcome!");

    use os::{KeyboardEvents, get_interface, get_datalink_channel, get_process_name, get_open_sockets};

    let stdout = io::stdout().into_raw_mode().unwrap();
    let terminal_backend = TermionBackend::new(stdout);

    let keyboard_events = Box::new(KeyboardEvents);
    let network_interface = get_interface().unwrap();
    let network_frames = get_datalink_channel(&network_interface);

    let os_input = what::OsInput {
        network_interface,
        network_frames,
        get_process_name,
        get_open_sockets,
        keyboard_events
    };

    what::start(terminal_backend, os_input)
}
