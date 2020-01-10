use async_trait::async_trait;
use std::net::Ipv4Addr;
use tokio::runtime::Handle;
use trust_dns_resolver::{error::ResolveErrorKind, AsyncResolver, TokioAsyncResolver};

#[async_trait]
pub trait Lookup {
    async fn lookup(&self, ip: Ipv4Addr) -> Option<String>;
}

pub struct Resolver(TokioAsyncResolver);

impl Resolver {
    pub async fn new(runtime: Handle) -> Result<Self, failure::Error> {
        let resolver = AsyncResolver::from_system_conf(runtime).await?;
        Ok(Self(resolver))
    }
}

#[async_trait]
impl Lookup for Resolver {
    async fn lookup(&self, ip: Ipv4Addr) -> Option<String> {
        let lookup_future = self.0.reverse_lookup(ip.into());
        match lookup_future.await {
            Ok(names) => {
                // Take the first result and convert it to a string
                names.into_iter().next().map(|name| name.to_string())
            }
            Err(e) => match e.kind() {
                // If the IP is not associated with a hostname, store the IP
                // so that we don't retry indefinitely
                ResolveErrorKind::NoRecordsFound { .. } => Some(ip.to_string()),
                _ => None,
            },
        }
    }
}
