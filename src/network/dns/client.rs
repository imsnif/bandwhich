use crate::network::dns::{resolver::Lookup, IpTable};
use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{Arc, Mutex},
    thread::{Builder, JoinHandle},
};
use tokio::{
    runtime::Runtime,
    sync::mpsc::{self, Sender},
};

type PendingAddrs = HashMap<IpAddr, f32>;

const CHANNEL_SIZE: usize = 1_000;

pub struct Client {
    cache: Arc<Mutex<IpTable>>,
    tx: Option<Sender<Vec<IpAddr>>>,
    handle: Option<JoinHandle<()>>,
}

impl Client {
    pub fn new<R>(resolver: R, mut runtime: Runtime) -> Result<Self, failure::Error>
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
                        let ips = ips
                            .into_iter()
                            .filter(|ip| {
                                let mut pending = pending.lock().unwrap().clone();
                                let cnt = pending.entry(*ip).or_insert(0.0);
                                let pwr_of2: bool = (*cnt == 0.0) | (cnt.log2() % 1.0 == 0.0);
                                *cnt += 1.0;
                                pwr_of2
                            })
                            .collect::<Vec<_>>();

                        for ip in ips {
                            tokio::spawn({
                                let resolver = resolver.clone();
                                let cache = cache.clone();
                                let pending = pending.clone();

                                async move {
                                    if let Some(name) = resolver.lookup(ip).await {
                                        cache.lock().unwrap().insert(ip, name);
                                        pending.lock().unwrap().remove(&ip);
                                    }
                                }
                            });
                        }
                    }
                });
            }
        })?;

        Ok(Self {
            cache,
            tx: Some(tx),
            handle: Some(handle),
        })
    }

    pub fn resolve(&mut self, ips: Vec<IpAddr>) {
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
