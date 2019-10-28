use ::std::collections::VecDeque;
use ::std::net::Ipv4Addr;
use ::std::sync::{Condvar, Mutex};

pub struct DnsQueue {
    jobs: Mutex<Option<VecDeque<Ipv4Addr>>>,
    cvar: Condvar,
}

impl DnsQueue {
    pub fn new() -> Self {
        DnsQueue {
            jobs: Mutex::new(Some(VecDeque::new())),
            cvar: Condvar::new(),
        }
    }
}

impl DnsQueue {
    pub fn resolve_ips(&self, unresolved_ips: Vec<Ipv4Addr>) {
        let mut jobs = self.jobs.lock().unwrap();
        if let Some(queue) = jobs.as_mut() {
            queue.extend(unresolved_ips);
            self.cvar.notify_all();
        }
    }
    pub fn wait_for_job(&self) -> Option<Ipv4Addr> {
        let mut jobs = self.jobs.lock().unwrap();
        loop {
            match jobs.as_mut()?.pop_front() {
                Some(job) => return Some(job),
                None => {
                    jobs = self.cvar.wait(jobs).unwrap()
                }
            }
        }
    }
    pub fn end(&self) {
        let mut jobs = self.jobs.lock().unwrap();
        *jobs = None;
        self.cvar.notify_all();
    }
}
