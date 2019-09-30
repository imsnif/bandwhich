use ::std::net::Ipv4Addr;

use ::std::mem::swap;
use ::std::sync::{Condvar, Mutex};

pub struct DnsQueue {
    jobs: Mutex<Vec<Ipv4Addr>>,
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
    pub fn add_ips_to_resolve(&self, mut unresolved_ips: Vec<Ipv4Addr>) {
        let mut queue = self.jobs.lock().unwrap();
        queue.append(&mut unresolved_ips);
        self.cvar.notify_all();
    }
    pub fn wait_for_jobs(&self) -> Vec<Ipv4Addr> {
        let mut jobs = self.cvar.wait(self.jobs.lock().unwrap()).unwrap();
        let mut new_jobs = Vec::new();
        swap(&mut new_jobs, &mut jobs);
        new_jobs
    }
    pub fn end(&self) {
        self.cvar.notify_all();
    }
}
