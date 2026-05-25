use std::{
    collections::{HashMap, HashSet},
    net::IpAddr,
    sync::{Arc, Mutex},
    thread::{Builder, JoinHandle},
    time::{Duration, Instant},
};

use tokio::{
    runtime::Runtime,
    sync::mpsc::{self, Sender},
};

use crate::network::dns::{resolver::Lookup, IpTable};

type PendingAddrs = HashSet<IpAddr>;

const CHANNEL_SIZE: usize = 1_000;
const INITIAL_BACKOFF: Duration = Duration::from_secs(1);
const MAX_BACKOFF: Duration = Duration::from_secs(60);

struct BackoffState {
    last_attempt: Instant,
    interval: Duration,
}

pub struct Client {
    cache: Arc<Mutex<IpTable>>,
    pending: Arc<Mutex<PendingAddrs>>,
    failed: Arc<Mutex<HashMap<IpAddr, BackoffState>>>,
    tx: Option<Sender<Vec<IpAddr>>>,
    handle: Option<JoinHandle<()>>,
}

impl Client {
    pub fn new<R>(resolver: R, runtime: Runtime) -> eyre::Result<Self>
    where
        R: Lookup + Send + Sync + 'static,
    {
        let cache = Arc::new(Mutex::new(IpTable::new()));
        let pending = Arc::new(Mutex::new(PendingAddrs::new()));
        let failed = Arc::new(Mutex::new(HashMap::<IpAddr, BackoffState>::new()));
        let (tx, mut rx) = mpsc::channel::<Vec<IpAddr>>(CHANNEL_SIZE);

        let handle = Builder::new().name("resolver".into()).spawn({
            let cache = cache.clone();
            let pending = pending.clone();
            let failed = failed.clone();
            move || {
                runtime.block_on(async {
                    let resolver = Arc::new(resolver);

                    while let Some(ips) = rx.recv().await {
                        for ip in ips {
                            tokio::spawn({
                                let resolver = resolver.clone();
                                let cache = cache.clone();
                                let pending = pending.clone();
                                let failed = failed.clone();

                                async move {
                                    match resolver.lookup(ip).await {
                                        Some(name) => {
                                            cache.lock().unwrap().insert(ip, name);
                                            failed.lock().unwrap().remove(&ip);
                                        }
                                        None => {
                                            let mut failed = failed.lock().unwrap();
                                            let prev_interval = failed
                                                .get(&ip)
                                                .map(|s| s.interval)
                                                .unwrap_or(INITIAL_BACKOFF);
                                            let next_interval =
                                                (prev_interval * 2).min(MAX_BACKOFF);
                                            failed.insert(
                                                ip,
                                                BackoffState {
                                                    last_attempt: Instant::now(),
                                                    interval: next_interval,
                                                },
                                            );
                                        }
                                    }
                                    pending.lock().unwrap().remove(&ip);
                                }
                            });
                        }
                    }
                });
            }
        })?;

        Ok(Self {
            cache,
            pending,
            failed,
            tx: Some(tx),
            handle: Some(handle),
        })
    }

    pub fn resolve(&mut self, ips: Vec<IpAddr>) {
        let failed = self.failed.lock().unwrap();
        let now = Instant::now();

        let ips = ips
            .into_iter()
            .filter(|ip| {
                if let Some(state) = failed.get(ip) {
                    if now.duration_since(state.last_attempt) < state.interval {
                        return false;
                    }
                }
                true
            })
            .filter(|ip| self.pending.lock().unwrap().insert(*ip))
            .collect::<Vec<_>>();
        drop(failed);

        if !ips.is_empty() {
            let _ = self.tx.as_mut().unwrap().try_send(ips);
        }
    }

    pub fn cache(&mut self) -> IpTable {
        let cache = self.cache.lock().unwrap();
        cache.clone()
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        // Do the Option dance to be able to drop the sender so that the receiver finishes and the thread can be joined
        drop(self.tx.take().unwrap());
        self.handle.take().unwrap().join().unwrap();
    }
}
