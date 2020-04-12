use ::std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use ::std::collections::HashMap;

use ::procfs::process::FDTarget;

use crate::network::{Connection, Protocol};
use crate::OpenSockets;

#[derive(Eq, PartialEq, PartialOrd)]
pub struct ProcessPid {
    pub procname: String,
    pub pid: i32,
}

impl Ord for ProcessPid {
    fn cmp(&self, other: &Self) -> Ordering {
        self.pid.cmp(&other.pid)
    }
}

pub(crate) fn get_open_sockets() -> OpenSockets {
    let mut open_sockets = HashMap::new();
    let mut connections = std::vec::Vec::new();
    let mut inode_to_procname = HashMap::new();

    if let Ok(all_procs) = procfs::process::all_processes() {
        for process in all_procs {
            if let Ok(fds) = process.fd() {
                for fd in fds {
                    if let FDTarget::Socket(inode) = fd.target {
                        inode_to_procname.insert(
                            inode,
                            ProcessPid {
                                procname: process.stat.comm.clone(),
                                pid: process.stat.pid,
                            },
                        );
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
            if let (connection, Some(proc_pid)) = (
                Connection::new(entry.remote_address, local_ip, local_port, Protocol::Tcp),
                inode_to_procname.get(&entry.inode),
            ) {
                open_sockets.insert(
                    connection.local_socket,
                    ProcessPid {
                        procname: proc_pid.procname.clone(),
                        pid: proc_pid.pid,
                    },
                );
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
            if let (connection, Some(proc_pid)) = (
                Connection::new(entry.remote_address, local_ip, local_port, Protocol::Udp),
                inode_to_procname.get(&entry.inode),
            ) {
                open_sockets.insert(
                    connection.local_socket,
                    ProcessPid {
                        procname: proc_pid.procname.clone(),
                        pid: proc_pid.pid,
                    },
                );
                connections.push(connection);
            };
        }
    }
    OpenSockets {
        sockets_to_procs: open_sockets,
        connections,
    }
}
