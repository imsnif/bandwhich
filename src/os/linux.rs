use ::std::collections::HashMap;

use ::procfs::process::FDTarget;

use crate::network::{LocalSocket, Protocol};
use crate::OpenSockets;

pub(crate) fn get_open_sockets() -> OpenSockets {
    let mut open_sockets = HashMap::new();
    let mut inode_to_procname = HashMap::new();

    if let Ok(all_procs) = procfs::process::all_processes() {
        for process in all_procs.flatten() {
            let proc_name = process.stat().unwrap().comm;
            if let Ok(fd_it) = process.fd() {
                for fdi in fd_it.flatten() {
                    if let FDTarget::Socket(inode) = fdi.target {
                        inode_to_procname.insert(inode, proc_name.clone());
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
            if let Some(procname) = inode_to_procname.get(&entry.inode) {
                open_sockets.insert(
                    LocalSocket {
                        ip: entry.local_address.ip(),
                        port: entry.local_address.port(),
                        protocol: Protocol::Tcp,
                    },
                    procname.clone(),
                );
            };
        }
    }

    if let Ok(mut udp) = ::procfs::net::udp() {
        if let Ok(mut udp6) = ::procfs::net::udp6() {
            udp.append(&mut udp6);
        }
        for entry in udp.into_iter() {
            if let Some(procname) = inode_to_procname.get(&entry.inode) {
                open_sockets.insert(
                    LocalSocket {
                        ip: entry.local_address.ip(),
                        port: entry.local_address.port(),
                        protocol: Protocol::Udp,
                    },
                    procname.clone(),
                );
            };
        }
    }

    OpenSockets {
        sockets_to_procs: open_sockets,
    }
}
