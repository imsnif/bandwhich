use ::std::net::Ipv4Addr;
use ::std::sync::{Condvar, Mutex};
use ::std::collections::HashMap;

use crate::network::Connection;

pub struct DnsQueue {
    jobs: Mutex<Vec<Option<Ipv4Addr>>>,
    cvar: Condvar,
}

impl DnsQueue {
    pub fn new() -> Self {
        DnsQueue {
            jobs: Mutex::new(Vec::new()),
            cvar: Condvar::new(),
        }
    }
}

impl DnsQueue {
    pub fn find_ips_to_resolve(
        &self,
        connections_to_procs: &HashMap<Connection, String>,
        ip_to_host: &HashMap<Ipv4Addr, String>,
    ) {
        let mut queue = self.jobs.lock().unwrap();
        for connection in connections_to_procs.keys() {
            if !ip_to_host.contains_key(&connection.local_socket.ip) {
                queue.push(Some(connection.local_socket.ip.clone()));
            }
            if !ip_to_host.contains_key(&connection.remote_socket.ip) {
                queue.push(Some(connection.remote_socket.ip.clone()));
            }
        };
        self.cvar.notify_all();
    }
    pub fn wait_for_job(&self) -> Option<Ipv4Addr> {
        let mut jobs = self.jobs.lock().unwrap();
        if jobs.is_empty() {
            jobs = self.cvar.wait(jobs).unwrap();
        }
        jobs.pop()?
    }
    pub fn end(&self) {
        self.jobs.lock().unwrap().push(None);
        self.cvar.notify_all();
    }
}
