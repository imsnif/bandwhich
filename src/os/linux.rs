use ::std::collections::HashMap;

use ::procfs::process::FDTarget;

use crate::network::{Connection, Protocol};

pub(crate) fn get_open_sockets() -> HashMap<Connection, String> {
    let mut open_sockets = HashMap::new();
    let all_procs = procfs::process::all_processes().unwrap();

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

    let tcp = ::procfs::net::tcp().unwrap();
    for entry in tcp.into_iter() {
        let local_port = entry.local_address.port();
        if let (Some(connection), Some(procname)) = (
            Connection::new(entry.remote_address, local_port, Protocol::Tcp),
            inode_to_procname.get(&entry.inode),
        ) {
            open_sockets.insert(connection, procname.clone());
        };
    }

    let udp = ::procfs::net::udp().unwrap();
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
