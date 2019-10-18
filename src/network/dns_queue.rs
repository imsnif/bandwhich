use ::std::collections::{VecDeque, HashMap};
use ::std::net::Ipv4Addr;
use ::std::sync::{Condvar, Mutex};

use crate::network::Connection;

pub struct DnsQueue {
    jobs: Mutex<VecDeque<Option<Ipv4Addr>>>,
    cvar: Condvar,
}

impl DnsQueue {
    pub fn new() -> Self {
        DnsQueue {
            jobs: Mutex::new(VecDeque::new()),
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
            if !ip_to_host.contains_key(&connection.remote_socket.ip) {
                queue.push_back(Some(connection.remote_socket.ip));
            }
        }
        self.cvar.notify_all();
    }
    pub fn wait_for_job(&self) -> Option<Ipv4Addr> {
        let mut jobs = self.jobs.lock().unwrap();
        loop {
            match jobs.pop_front() {
                Some(job) => return job,
                None => {
                    jobs = self.cvar.wait(jobs).unwrap();
                }
            }
        }
    }
    pub fn end(&self) {
        let mut jobs = self.jobs.lock().unwrap();
        jobs.clear();
        jobs.push_back(None);
        self.cvar.notify_all();
    }
}
