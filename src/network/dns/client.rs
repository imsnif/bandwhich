use std::{
    collections::HashSet,
    net::IpAddr,
    sync::{Arc, Mutex},
    thread::{Builder, JoinHandle},
};

use tokio::{
    runtime::Runtime,
    sync::mpsc::{self, Sender},
};

use crate::network::dns::{resolver::Lookup, IpTable};

type PendingAddrs = HashSet<IpAddr>;

const CHANNEL_SIZE: usize = 1_000;

pub struct Client {
    cache: Arc<Mutex<IpTable>>,
    pending: Arc<Mutex<PendingAddrs>>,
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
        let (tx, mut rx) = mpsc::channel::<Vec<IpAddr>>(CHANNEL_SIZE);

        let handle = Builder::new().name("resolver".into()).spawn({
            let cache = cache.clone();
            let pending = pending.clone();
            move || {
                runtime.block_on(async {
                    let resolver = Arc::new(resolver);

                    while let Some(ips) = rx.recv().await {
                        for ip in ips {
                            tokio::spawn({
                                let resolver = resolver.clone();
                                let cache = cache.clone();
                                let pending = pending.clone();

                                async move {
                                    if let Some(name) = resolver.lookup(ip).await {
                                        cache.lock().unwrap().insert(ip, name);
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
            tx: Some(tx),
            handle: Some(handle),
        })
    }

    pub fn resolve(&mut self, ips: Vec<IpAddr>) {
        // Remove ips that are already being resolved
        let ips = ips
            .into_iter()
            .filter(|ip| self.pending.lock().unwrap().insert(*ip))
            .collect::<Vec<_>>();

        if !ips.is_empty() {
            // Discard the message if the channel is full; it will be retried eventually
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
