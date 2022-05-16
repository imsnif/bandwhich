use async_trait::async_trait;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::net::{Ipv4Addr, SocketAddrV4};
use trust_dns_resolver::config::{NameServerConfig, Protocol, ResolverConfig, ResolverOpts};
use trust_dns_resolver::{error::ResolveErrorKind, TokioAsyncResolver};

#[async_trait]
pub trait Lookup {
    async fn lookup(&self, ip: IpAddr) -> Option<String>;
}

pub struct Resolver(TokioAsyncResolver);

impl Resolver {
    pub fn new(dns_server: &Option<Ipv4Addr>) -> Result<Self, failure::Error> {
        let resolver = match dns_server {
            Some(dns_server_address) => {
                let mut config = ResolverConfig::new();
                let options = ResolverOpts::default();
                let socket = SocketAddr::V4(SocketAddrV4::new(*dns_server_address, 53));
                let nameserver_config = NameServerConfig {
                    socket_addr: socket,
                    protocol: Protocol::Udp,
                    tls_dns_name: None,
                    bind_addr: None,
                    trust_nx_responses: false,
                };
                config.add_name_server(nameserver_config);
                TokioAsyncResolver::tokio(config, options).unwrap()
            }
            None => TokioAsyncResolver::tokio_from_system_conf()?,
        };
        Ok(Self(resolver))
    }
}

#[async_trait]
impl Lookup for Resolver {
    async fn lookup(&self, ip: IpAddr) -> Option<String> {
        let lookup_future = self.0.reverse_lookup(ip);
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
