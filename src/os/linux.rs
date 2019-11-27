use ::std::collections::HashMap;

use ::procfs::FDTarget;

use crate::network::{Connection, Protocol};
use crate::OsInputOutput;

use crate::os::shared::{
    KeyboardEvents,
    get_datalink_channel,
    get_interface,
    lookup_addr,
    sigwinch,
    create_write_to_stdout,
};

fn get_open_sockets() -> HashMap<Connection, String> {
    let mut open_sockets = HashMap::new();
    let all_procs = procfs::all_processes();

    let mut inode_to_procname = HashMap::new();
    for process in all_procs {
        if let Ok(fds) = process.fd() {
            let procname = process.stat.comm;
            for fd in fds {
                if let FDTarget::Socket(inode) = fd.target {
                    inode_to_procname.insert(inode, procname.clone());
                }
            }
        }
    }

    let tcp = ::procfs::tcp().unwrap();
    for entry in tcp.into_iter() {
        let local_port = entry.local_address.port();
        if let (Some(connection), Some(procname)) = (
            Connection::new(entry.remote_address, local_port, Protocol::Tcp),
            inode_to_procname.get(&entry.inode),
        ) {
            open_sockets.insert(connection, procname.clone());
        };
    }

    let udp = ::procfs::udp().unwrap();
    for entry in udp.into_iter() {
        let local_port = entry.local_address.port();
        if let (Some(connection), Some(procname)) = (
            Connection::new(entry.remote_address, local_port, Protocol::Udp),
            inode_to_procname.get(&entry.inode),
        ) {
            open_sockets.insert(connection, procname.clone());
        };
    }
    open_sockets
}



pub fn get_input(interface_name: &str) -> Result<OsInputOutput, failure::Error> {
    let keyboard_events = Box::new(KeyboardEvents);
    let network_interface = match get_interface(interface_name) {
        Some(interface) => interface,
        None => {
            failure::bail!("Cannot find interface {}", interface_name);
        }
    };
    let network_frames = get_datalink_channel(&network_interface)?;
    let lookup_addr = Box::new(lookup_addr);
    let write_to_stdout = create_write_to_stdout();
    let (on_winch, cleanup) = sigwinch();

    Ok(OsInputOutput {
        network_interface,
        network_frames,
        get_open_sockets,
        keyboard_events,
        lookup_addr,
        on_winch,
        cleanup,
        write_to_stdout,
    })
}
