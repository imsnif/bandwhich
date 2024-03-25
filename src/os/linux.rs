use std::collections::HashMap;

use procfs::process::FDTarget;

use crate::{
    network::{LocalSocket, Protocol},
    os::ProcessInfo,
    OpenSockets,
};

pub(crate) fn get_open_sockets() -> OpenSockets {
    let mut open_sockets = HashMap::new();
    let mut inode_to_proc = HashMap::new();

    if let Ok(all_procs) = procfs::process::all_processes() {
        for process in all_procs.filter_map(|res| res.ok()) {
            let Ok(fds) = process.fd() else { continue };
            let Ok(stat) = process.stat() else { continue };
            let proc_name = stat.comm;
            let proc_info = ProcessInfo::new(&proc_name, stat.pid as u32);
            for fd in fds.filter_map(|res| res.ok()) {
                if let FDTarget::Socket(inode) = fd.target {
                    inode_to_proc.insert(inode, proc_info.clone());
                }
            }
        }
    }

    macro_rules! insert_proto {
        ($source: expr, $proto: expr) => {
            let entries = $source.into_iter().filter_map(|res| res.ok()).flatten();
            for entry in entries {
                if let Some(proc_info) = inode_to_proc.get(&entry.inode) {
                    let socket = LocalSocket {
                        ip: entry.local_address.ip(),
                        port: entry.local_address.port(),
                        protocol: $proto,
                    };
                    open_sockets.insert(socket, proc_info.clone());
                }
            }
        };
    }

    insert_proto!([procfs::net::tcp(), procfs::net::tcp6()], Protocol::Tcp);
    insert_proto!([procfs::net::udp(), procfs::net::udp6()], Protocol::Udp);

    OpenSockets {
        sockets_to_procs: open_sockets,
    }
}
