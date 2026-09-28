use std::{fs, path::PathBuf, process};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use pnet::datalink::DataLinkReceiver;

use crate::{
    start,
    tests::{
        cases::test_utils::{
            build_tcp_packet, opts_raw, opts_ui, os_input_output_factory, test_backend_factory,
        },
        fakes::{create_fake_dns_client, NetworkFrames, TerminalEvents},
    },
    Opt,
};

fn key(c: char) -> Option<Event> {
    Some(Event::Key(KeyEvent::new(
        KeyCode::Char(c),
        KeyModifiers::NONE,
    )))
}

/// A fresh, empty directory unique to this test.
fn test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("bandwhich-test-{}-{name}", process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn one_packet() -> Vec<Box<dyn DataLinkReceiver>> {
    vec![NetworkFrames::new(vec![Some(build_tcp_packet(
        "10.0.0.2",
        "1.1.1.1",
        443,
        12345,
        b"I am a fake tcp packet",
    ))]) as Box<dyn DataLinkReceiver>]
}

/// Run bandwhich with the given key events, returning the contents of each dump file created.
fn run_and_collect_dumps(name: &str, opts: Opt, events: Vec<Option<Event>>) -> Vec<String> {
    let dir = test_dir(name);
    let opts = Opt {
        dump_dir: Some(dir.clone()),
        ..opts
    };
    let (_, _, backend) = test_backend_factory(190, 50);
    let os_input = os_input_output_factory(
        one_packet(),
        None,
        create_fake_dns_client(Default::default()),
        Box::new(TerminalEvents::new(events)),
    );
    start(backend, os_input, opts);

    let mut files = fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    files.sort();
    for file in &files {
        let file_name = file.file_name().unwrap().to_string_lossy();
        assert!(
            file_name.starts_with("bandwhich-dump-") && file_name.ends_with(".log"),
            "unexpected dump file name: {file_name}"
        );
    }
    let dumps = files
        .iter()
        .map(|file| fs::read_to_string(file).unwrap())
        .collect();
    fs::remove_dir_all(&dir).unwrap();
    dumps
}

fn assert_dump_has_traffic(dump: &str) {
    let lines = dump.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 3, "unexpected dump:\n{dump}");
    assert!(lines[0].starts_with("process: <") && lines[0].contains("\"1\""));
    assert!(lines[1].starts_with("connection: <") && lines[1].contains("1.1.1.1:12345"));
    assert!(lines[2].starts_with("remote_address: <") && lines[2].contains("1.1.1.1"));
}

#[test]
fn dump_state_to_file() {
    let dumps = run_and_collect_dumps("dump", opts_ui(), vec![None, None, key('d'), key('q')]);
    assert_eq!(dumps.len(), 1);
    assert_dump_has_traffic(&dumps[0]);
}

#[test]
fn dump_state_while_paused() {
    let dumps = run_and_collect_dumps(
        "dump_paused",
        opts_ui(),
        vec![None, None, key(' '), None, None, key('d'), key('q')],
    );
    assert_eq!(dumps.len(), 1);
    assert_dump_has_traffic(&dumps[0]);
}

#[test]
fn dump_state_multiple_times() {
    let dumps = run_and_collect_dumps(
        "dump_multiple",
        opts_ui(),
        vec![None, None, key('d'), None, key('d'), key('q')],
    );
    assert_eq!(dumps.len(), 2);
}

#[test]
fn dump_state_includes_all_tables() {
    let opts = Opt {
        render_opts: crate::cli::RenderOpts {
            processes: true,
            ..Default::default()
        },
        ..opts_ui()
    };
    let dumps = run_and_collect_dumps(
        "dump_all_tables",
        opts,
        vec![None, None, key('d'), key('q')],
    );
    assert_eq!(dumps.len(), 1);
    assert_dump_has_traffic(&dumps[0]);
}

#[test]
fn dump_state_in_raw_mode() {
    let dumps = run_and_collect_dumps("dump_raw", opts_raw(), vec![None, None, key('d'), key('q')]);
    assert_eq!(dumps.len(), 1);
    assert_dump_has_traffic(&dumps[0]);
}

#[test]
fn dump_state_with_no_traffic() {
    let dir = test_dir("dump_no_traffic");
    let (_, _, backend) = test_backend_factory(190, 50);
    let os_input = os_input_output_factory(
        vec![NetworkFrames::new(vec![None]) as Box<dyn DataLinkReceiver>],
        None,
        create_fake_dns_client(Default::default()),
        Box::new(TerminalEvents::new(vec![None, key('d'), key('q')])),
    );
    let opts = Opt {
        dump_dir: Some(dir.clone()),
        ..opts_ui()
    };
    start(backend, os_input, opts);

    let files = fs::read_dir(&dir).unwrap().collect::<Vec<_>>();
    assert_eq!(files.len(), 1);
    let dump = fs::read_to_string(files[0].as_ref().unwrap().path()).unwrap();
    assert_eq!(dump, "<NO TRAFFIC>\n");
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn dump_state_to_missing_dir_does_not_crash() {
    let dir = test_dir("dump_missing_dir").join("does-not-exist");
    let (_, _, backend) = test_backend_factory(190, 50);
    let os_input = os_input_output_factory(
        one_packet(),
        None,
        create_fake_dns_client(Default::default()),
        Box::new(TerminalEvents::new(vec![None, key('d'), key('q')])),
    );
    let opts = Opt {
        dump_dir: Some(dir.clone()),
        ..opts_ui()
    };
    start(backend, os_input, opts);
    assert!(!dir.exists());
    fs::remove_dir_all(dir.parent().unwrap()).unwrap();
}

#[test]
fn dump_state_shows_notice_in_footer() {
    let dir = test_dir("dump_notice");
    let (_, terminal_draw_events, backend) = test_backend_factory(190, 50);
    let os_input = os_input_output_factory(
        one_packet(),
        None,
        create_fake_dns_client(Default::default()),
        Box::new(TerminalEvents::new(vec![None, key('d'), key('q')])),
    );
    let opts = Opt {
        dump_dir: Some(dir.clone()),
        ..opts_ui()
    };
    start(backend, os_input, opts);

    let draws = terminal_draw_events.lock().unwrap();
    assert!(!draws[0].contains("State dumped to"));
    // only changed cells are drawn, so the unchanged "Press <SPACE>..." prefix is not included
    assert!(draws
        .iter()
        .any(|draw| draw.contains("State dumped to /") && draw.contains("/bandwhich-dump-")));
    fs::remove_dir_all(&dir).unwrap();
}
