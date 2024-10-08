use std::collections::HashMap;

use netstat2::*;
use sysinfo::{Pid, ProcessesToUpdate, System};

use crate::{
    network::{LocalSocket, Protocol},
    os::ProcessInfo,
    OpenSockets,
};

pub(crate) fn get_open_sockets() -> OpenSockets {
    let mut open_sockets = HashMap::new();

    let mut sysinfo = System::new_all();
    sysinfo.refresh_processes(ProcessesToUpdate::All, true);

    let af_flags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
    let proto_flags = ProtocolFlags::TCP | ProtocolFlags::UDP;
    let sockets_info = get_sockets_info(af_flags, proto_flags);

    if let Ok(sockets_info) = sockets_info {
        for si in sockets_info {
            let proc_info = si
                .associated_pids
                .into_iter()
                .find_map(|pid| sysinfo.process(Pid::from_u32(pid)))
                .map(|p| ProcessInfo::new(&p.name().to_string_lossy(), p.pid().as_u32()))
                .unwrap_or_default();

            match si.protocol_socket_info {
                ProtocolSocketInfo::Tcp(tcp_si) => {
                    open_sockets.insert(
                        LocalSocket {
                            ip: tcp_si.local_addr,
                            port: tcp_si.local_port,
                            protocol: Protocol::Tcp,
                        },
                        proc_info,
                    );
                }
                ProtocolSocketInfo::Udp(udp_si) => {
                    open_sockets.insert(
                        LocalSocket {
                            ip: udp_si.local_addr,
                            port: udp_si.local_port,
                            protocol: Protocol::Udp,
                        },
                        proc_info,
                    );
                }
            }
        }
    }

    OpenSockets {
        sockets_to_procs: open_sockets,
    }
}
