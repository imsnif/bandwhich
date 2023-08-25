use ::std::collections::HashMap;

use crate::network::LocalSocket;
use crate::OpenSockets;

use super::lsof_utils;

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
