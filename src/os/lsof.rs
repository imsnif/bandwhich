use crate::{os::lsof_utils::get_connections, OpenSockets};

pub(crate) fn get_open_sockets() -> OpenSockets {
    let sockets_to_procs = get_connections()
        .filter_map(|raw| raw.as_local_socket().map(|s| (s, raw.proc_info)))
        .collect();

    OpenSockets { sockets_to_procs }
}
