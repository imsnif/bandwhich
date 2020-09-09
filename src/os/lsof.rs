use ::std::collections::HashMap;

use crate::network::{Connection, LocalSocket};
use crate::OpenSockets;

use super::lsof_utils;
use std::net::SocketAddr;

#[derive(Debug)]
struct RawConnection {
    ip: String,
    local_port: String,
    remote_port: String,
    protocol: String,
    process_name: String,
}

pub(crate) fn get_open_sockets() -> OpenSockets {
    let mut open_sockets = HashMap::new();

    let connections = lsof_utils::get_connections();

    for raw_connection in connections {
        open_sockets.insert(
            LocalSocket {
                ip: raw_connection.get_local_ip(),
                port: raw_connection.get_local_port(),
                protocol: raw_connection.get_protocol(),
            },
            raw_connection.process_name.clone(),
        );
    }

    OpenSockets {
        sockets_to_procs: open_sockets,
    }
}
