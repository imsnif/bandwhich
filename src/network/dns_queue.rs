use ::std::collections::VecDeque;
use ::std::net::Ipv4Addr;
use ::std::sync::{Condvar, Mutex};

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
    pub fn resolve_ips(&self, unresolved_ips: Vec<Ipv4Addr>) {
        let mut queue = self.jobs.lock().unwrap();
        for ip in unresolved_ips {
            queue.push_back(Some(ip))
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
