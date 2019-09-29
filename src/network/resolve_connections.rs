use crate::Connection;

use ::std::net::Ipv4Addr;
use ::std::collections::HashMap;

pub fn resolve_connections (
    open_sockets: HashMap<Connection, String>,
    ip_to_host: &HashMap<Ipv4Addr, String>
) -> (Vec<Ipv4Addr>, HashMap<Connection, String>) {
    let mut unresolved_ips = vec![];
    let mut resolved_connections_to_procs: HashMap<Connection, String> = HashMap::new();
    for connection in open_sockets.keys() {
        let mut connection = connection.clone();
        match ip_to_host.get(&connection.local_socket.ip) {
            Some(local_host_addr) => connection.set_local_host_addr(local_host_addr),
            None => unresolved_ips.push(connection.local_socket.ip.clone()),
        }
        match ip_to_host.get(&connection.remote_socket.ip) {
            Some(remote_host_addr) => connection.set_remote_host_addr(remote_host_addr),
            None => unresolved_ips.push(connection.remote_socket.ip.clone()),
        }
        let connection_value = open_sockets.get(&connection).unwrap().clone();
        &resolved_connections_to_procs.insert(connection, connection_value);
    }
    (unresolved_ips, resolved_connections_to_procs)
}
