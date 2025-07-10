use std::{
    io::{self, ErrorKind, Write},
    net::Ipv4Addr,
    time,
};

use crossterm::event::{read, Event};
use eyre::{bail, eyre};
use itertools::Itertools;
use log::{debug, warn};
use pnet::datalink::{self, Channel::Ethernet, Config, DataLinkReceiver, NetworkInterface};
use tokio::runtime::Runtime;

use crate::{network::dns, os::errors::GetInterfaceError, OsInputOutput};

#[cfg(any(target_os = "android", target_os = "linux"))]
use crate::os::linux::get_open_sockets;
#[cfg(any(target_os = "macos", target_os = "freebsd"))]
use crate::os::lsof::get_open_sockets;
#[cfg(target_os = "windows")]
use crate::os::windows::get_open_sockets;

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct ProcessInfo {
    pub name: String,
    pub pid: u32,
}

impl ProcessInfo {
    pub fn new(name: &str, pid: u32) -> Self {
        Self {
            name: name.to_string(),
            pid,
        }
    }
}

pub struct TerminalEvents;

impl Iterator for TerminalEvents {
    type Item = Event;
    fn next(&mut self) -> Option<Event> {
        read().ok()
    }
}

pub(crate) fn get_datalink_channel(
    interface: &NetworkInterface,
) -> Result<Box<dyn DataLinkReceiver>, GetInterfaceError> {
    let config = Config {
        read_timeout: Some(time::Duration::new(1, 0)),
        read_buffer_size: 65536,
        ..Default::default()
    };

    match datalink::channel(interface, config) {
        Ok(Ethernet(_tx, rx)) => Ok(rx),
        Ok(_) => Err(GetInterfaceError::OtherError(format!(
            "{}: Unsupported interface type",
            interface.name
        ))),
        Err(e) => match e.kind() {
            ErrorKind::PermissionDenied => Err(GetInterfaceError::PermissionError(
                interface.name.to_owned(),
            )),
            _ => Err(GetInterfaceError::OtherError(format!(
                "{}: {e}",
                &interface.name
            ))),
        },
    }
}

fn get_interface(interface_name: &str) -> Option<NetworkInterface> {
    datalink::interfaces()
        .into_iter()
        .find(|iface| iface.name == interface_name)
}

fn create_write_to_stdout() -> Box<dyn FnMut(&str) + Send> {
    let mut stdout = io::stdout();
    Box::new({
        move |output| match writeln!(stdout, "{output}") {
            Ok(_) => (),
            Err(e) if e.kind() == ErrorKind::BrokenPipe => {
                // A process that was listening to bandwhich stdout has exited
                // We can't do much here, lets just exit as well
                std::process::exit(0)
            }
            Err(e) => panic!("Failed to write to stdout: {e}"),
        }
    })
}

pub fn get_input(
    interface_name: Option<&str>,
    resolve: bool,
    dns_server: Option<Ipv4Addr>,
) -> eyre::Result<OsInputOutput> {
    // get the user's requested interface, if any
    // IDEA: allow requesting multiple interfaces
    let requested_interfaces = interface_name
        .map(|name| get_interface(name).ok_or_else(|| eyre!("Cannot find interface {name}")))
        .transpose()?
        .map(|interface| vec![interface]);

    // take the user's requested interfaces (or all interfaces), and filter for up ones
    let available_interfaces = requested_interfaces
        .unwrap_or_else(datalink::interfaces)
        .into_iter()
        .filter(|interface| {
            // see https://github.com/libpnet/libpnet/issues/564
            let keep = if cfg!(target_os = "windows") {
                !interface.ips.is_empty()
            } else {
                interface.is_up() && !interface.ips.is_empty()
            };
            if !keep {
                debug!("{} is down. Skipping it.", interface.name);
            }
            keep
        })
        .collect_vec();

    // bail if no interfaces are up
    if available_interfaces.is_empty() {
        bail!("Failed to find any network interface to listen on.");
    }

    // try to get a frame receiver for each interface
    let interfaces_with_frames_res = available_interfaces
        .into_iter()
        .map(|interface| {
            let frames_res = get_datalink_channel(&interface);
            (interface, frames_res)
        })
        .collect_vec();

    // warn for all frame receivers we failed to acquire
    interfaces_with_frames_res
        .iter()
        .filter_map(|(interface, frames_res)| frames_res.as_ref().err().map(|err| (interface, err)))
        .for_each(|(interface, err)| {
            warn!(
                "Failed to acquire a frame receiver for {}: {err}",
                interface.name
            )
        });

    // bail if all of them fail
    // note that `Iterator::all` returns `true` for an empty iterator, so it is important to handle
    // that failure mode separately, which we already have
    if interfaces_with_frames_res
        .iter()
        .all(|(_, frames)| frames.is_err())
    {
        let (permission_err_interfaces, other_errs) = interfaces_with_frames_res.iter().fold(
            (vec![], vec![]),
            |(mut perms, mut others), (_, res)| {
                match res {
                    Ok(_) => (),
                    Err(GetInterfaceError::PermissionError(interface)) => {
                        perms.push(interface.as_str())
                    }
                    Err(GetInterfaceError::OtherError(err)) => others.push(err.as_str()),
                }
                (perms, others)
            },
        );

        let err_msg = match (permission_err_interfaces.is_empty(), other_errs.is_empty()) {
            (false, false) => format!(
                "\n\n{}: {}\nAdditional errors:\n{}",
                permission_err_interfaces.join(", "),
                eperm_message(),
                other_errs.join("\n")
            ),
            (false, true) => format!(
                "\n\n{}: {}",
                permission_err_interfaces.join(", "),
                eperm_message()
            ),
            (true, false) => format!("\n\n{}", other_errs.join("\n")),
            (true, true) => unreachable!("Found no errors in error handling code path."),
        };
        bail!(err_msg);
    }

    // filter out interfaces for which we failed to acquire a frame receiver
    let interfaces_with_frames = interfaces_with_frames_res
        .into_iter()
        .filter_map(|(interface, res)| res.ok().map(|frames| (interface, frames)))
        .collect();

    let dns_client = if resolve {
        let runtime = Runtime::new()?;
        let resolver = runtime
            .block_on(dns::Resolver::new(dns_server))
            .map_err(|err| {
                eyre!("Could not initialize the DNS resolver. Are you offline?\n\nReason: {err}")
            })?;
        let dns_client = dns::Client::new(resolver, runtime)?;
        Some(dns_client)
    } else {
        None
    };

    let write_to_stdout = create_write_to_stdout();

    Ok(OsInputOutput {
        interfaces_with_frames,
        get_open_sockets,
        terminal_events: Box::new(TerminalEvents),
        dns_client,
        write_to_stdout,
    })
}

#[inline]
#[cfg(any(target_os = "macos", target_os = "freebsd"))]
fn eperm_message() -> &'static str {
    "Insufficient permissions to listen on network interface(s). Try running with sudo."
}

#[inline]
#[cfg(any(target_os = "android", target_os = "linux"))]
fn eperm_message() -> &'static str {
    r#"
    Insufficient permissions to listen on network interface(s). You can work around
    this issue like this:

    * Try running `bandwhich` with `sudo`

    * Build a `setcap(8)` wrapper for `bandwhich` with the following rules:
        `cap_sys_ptrace,cap_dac_read_search,cap_net_raw,cap_net_admin+ep`
    "#
}

#[inline]
#[cfg(target_os = "windows")]
fn eperm_message() -> &'static str {
    "Insufficient permissions to listen on network interface(s). Try running with administrator rights."
}
