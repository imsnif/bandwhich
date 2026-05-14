use std::{
    future::Future,
    net::{IpAddr, Ipv4Addr},
    slice,
};

use hickory_resolver::{
    config::{ResolverConfig, ServerGroup},
    net::runtime::TokioRuntimeProvider,
    TokioResolver,
};

pub trait Lookup {
    fn lookup(&self, ip: IpAddr) -> impl Future<Output = Option<String>> + Send;
}

pub struct Resolver(TokioResolver);

impl Resolver {
    pub async fn new(dns_server: Option<Ipv4Addr>) -> eyre::Result<Self> {
        let resolver = match dns_server {
            Some(dns_server_address) => {
                let addr = dns_server_address.into();
                let servers = ServerGroup {
                    ips: slice::from_ref(&addr),
                    server_name: "", // not currently used; only used for TLS
                    path: "",        // not currently used; only used for HTTP
                };
                let config = ResolverConfig::udp_and_tcp(&servers);
                TokioResolver::builder_with_config(config, TokioRuntimeProvider::default())
                    .build()?
            }
            None => TokioResolver::builder_tokio()?.build()?,
        };
        Ok(Self(resolver))
    }
}

impl Lookup for Resolver {
    async fn lookup(&self, ip: IpAddr) -> Option<String> {
        let lookup_future = self.0.reverse_lookup(ip);
        match lookup_future.await {
            Ok(lookup) => lookup.answers().first().map(|name| name.to_string()),
            Err(err) if err.is_no_records_found() => {
                // If the IP is not associated with a hostname, store the IP
                // so that we don't retry indefinitely
                Some(ip.to_string())
            }
            Err(_) => None,
        }
    }
}
