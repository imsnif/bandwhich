use ::std::collections::HashMap;

use ::procfs::process::FDTarget;

use crate::network::{Connection, Protocol};
use crate::OpenSockets;

pub(crate) fn get_open_sockets() -> OpenSockets {
    let mut open_sockets = HashMap::new();
    let mut connections = std::vec::Vec::new();
    let mut inode_to_procname = HashMap::new();

    if let Ok(all_procs) = procfs::process::all_processes() {
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
    }

    if let Ok(tcp) = ::procfs::net::tcp() {
        for entry in tcp.into_iter() {
            let local_port = entry.local_address.port();
            let local_ip = entry.local_address.ip();
            if let (Some(connection), Some(procname)) = (
                Connection::new(entry.remote_address, local_ip, local_port, Protocol::Tcp),
                inode_to_procname.get(&entry.inode),
            ) {
                open_sockets.insert(connection.local_socket, procname.clone());
                connections.push(connection);
            };
        }
    }

    if let Ok(udp) = ::procfs::net::udp() {
        for entry in udp.into_iter() {
            let local_port = entry.local_address.port();
            let local_ip = entry.local_address.ip();
            if let (Some(connection), Some(procname)) = (
                Connection::new(entry.remote_address, local_ip, local_port, Protocol::Udp),
                inode_to_procname.get(&entry.inode),
            ) {
                open_sockets.insert(connection.local_socket, procname.clone());
                connections.push(connection);
            };
        }
    }
    OpenSockets {
        sockets_to_procs: open_sockets,
        connections,
    }
}
