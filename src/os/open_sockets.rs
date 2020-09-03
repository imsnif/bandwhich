use ::std::collections::HashMap;

use ::sysinfo::ProcessExt;
use netstat2::*;

use crate::network::{LocalSocket, Protocol};
use crate::OpenSockets;
use sysinfo::{Pid, System, SystemExt};

pub(crate) fn get_open_sockets() -> OpenSockets {
    let mut open_sockets = HashMap::new();

    let mut sysinfo = System::new_all();
    sysinfo.refresh_processes();

    let af_flags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
    let proto_flags = ProtocolFlags::TCP | ProtocolFlags::UDP;
    let sockets_info = get_sockets_info(af_flags, proto_flags);

    if let Ok(sockets_info) = sockets_info {
        for si in sockets_info {
            let mut procname = "";
            for pid in si.associated_pids {
                if let Some(process) = sysinfo.get_process(pid as Pid) {
                    procname = process.name();
                    break;
                }
            }

            match si.protocol_socket_info {
                ProtocolSocketInfo::Tcp(tcp_si) => {
                    open_sockets.insert(
                        LocalSocket {
                            ip: tcp_si.local_addr,
                            port: tcp_si.local_port,
                            protocol: Protocol::Tcp,
                        },
                        String::from(procname),
                    );
                }
                ProtocolSocketInfo::Udp(udp_si) => {
                    open_sockets.insert(
                        LocalSocket {
                            ip: udp_si.local_addr,
                            port: udp_si.local_port,
                            protocol: Protocol::Udp,
                        },
                        String::from(procname),
                    );
                }
            }
        }
    }

    OpenSockets {
        sockets_to_procs: open_sockets,
    }
}
