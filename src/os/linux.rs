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

    if let Ok(mut tcp) = ::procfs::net::tcp() {
        if let Ok(mut tcp6) = ::procfs::net::tcp6() {
            tcp.append(&mut tcp6);
        }
        for entry in tcp.into_iter() {
            let local_port = entry.local_address.port();
            let local_ip = entry.local_address.ip();
            if let (connection, Some(procname)) = (
                Connection::new(entry.remote_address, local_ip, local_port, Protocol::Tcp),
                inode_to_procname.get(&entry.inode),
            ) {
                open_sockets.insert(connection.local_socket, procname.clone());
                connections.push(connection);
            };
        }
    }

    if let Ok(mut udp) = ::procfs::net::udp() {
        if let Ok(mut udp6) = ::procfs::net::udp6() {
            udp.append(&mut udp6);
        }
        for entry in udp.into_iter() {
            let local_port = entry.local_address.port();
            let local_ip = entry.local_address.ip();
            if let (connection, Some(procname)) = (
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
