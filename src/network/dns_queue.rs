use ::std::net::Ipv4Addr;

use ::std::sync::{Condvar, Mutex};

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
    pub fn add_ips_to_resolve(&self, unresolved_ips: Vec<Ipv4Addr>) {
        let mut queue = self.jobs.lock().unwrap();
        for ip in unresolved_ips {
            queue.push(Some(ip));
        }
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
