use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4},
    time::Duration,
};

use async_trait::async_trait;
use log::warn;
use tokio::time::sleep;
use trust_dns_resolver::{
    config::{NameServerConfig, Protocol, ResolverConfig, ResolverOpts},
    TokioAsyncResolver,
};

#[async_trait]
pub trait Lookup {
    async fn lookup(&self, ip: IpAddr) -> Option<String>;
}

pub struct Resolver(TokioAsyncResolver);

impl Resolver {
    pub async fn new(dns_server: Option<Ipv4Addr>) -> eyre::Result<Self> {
        let resolver = match dns_server {
            Some(dns_server_address) => {
                let mut config = ResolverConfig::new();
                let options = ResolverOpts::default();
                let socket = SocketAddr::V4(SocketAddrV4::new(dns_server_address, 53));
                let nameserver_config = NameServerConfig {
                    socket_addr: socket,
                    protocol: Protocol::Udp,
                    tls_dns_name: None,
                    trust_negative_responses: false,
                    bind_addr: None,
                };
                config.add_name_server(nameserver_config);
                TokioAsyncResolver::tokio(config, options)
            }
            None => TokioAsyncResolver::tokio_from_system_conf()?,
        };
        Ok(Self(resolver))
    }
}

#[async_trait]
impl Lookup for Resolver {
    async fn lookup(&self, ip: IpAddr) -> Option<String> {
        let retry_config = RetryPolicy {
            max_retries: 3,
            ..Default::default()
        };

        retry_with_backoff(
            || {
                let resolver = &self.0;
                async move {
                    resolver
                        .reverse_lookup(ip)
                        .await
                        .ok()
                        .and_then(|names| names.iter().next().map(|n| n.to_string()))
                        .or_else(|| Some(ip.to_string()))
                }
            },
            retry_config.max_retries,
            retry_config.base_delay,
        )
        .await
        .or_else(|| Some("DNS lookup timeout.".into()))
    }
}

struct RetryPolicy {
    max_retries: u8,
    base_delay: tokio::time::Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 2,
            base_delay: Duration::from_millis(1000),
        }
    }
}

pub async fn retry_with_backoff<F, Fut, T>(
    mut operation: F,
    max_reties: u8,
    inittial_delay: tokio::time::Duration,
) -> Option<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Option<T>>,
{
    let mut delay = inittial_delay;
    for attemp in 0..=max_reties {
        match operation().await {
            Some(value) => return Some(value),
            None if attemp < max_reties => {
                warn!(
                    "Retrying.. attemp: {}/{} (waiting {:?})",
                    attemp + 1,
                    max_reties,
                    delay
                );
                sleep(delay).await;
                delay *= 2;
            }
            None => return None,
        }
    }
    None
}
