use std::{collections::HashMap, net::IpAddr};

use pnet::datalink::{self, NetworkInterface};
use procfs::process::FDTarget;

use crate::{
    network::{LocalSocket, Protocol},
    os::ProcessInfo,
    OpenSockets,
};

pub(crate) fn get_local_addresses() -> Vec<IpAddr> {
    local_addresses_from_interfaces(datalink::interfaces())
}

fn local_addresses_from_interfaces(interfaces: Vec<NetworkInterface>) -> Vec<IpAddr> {
    interfaces
        .into_iter()
        .filter(|interface| interface.is_up())
        .flat_map(|interface| interface.ips.into_iter().map(|ip_network| ip_network.ip()))
        .collect()
}

pub(crate) fn get_open_sockets() -> OpenSockets {
    let mut open_sockets = HashMap::new();
    let mut inode_to_proc = HashMap::new();

    if let Ok(all_procs) = procfs::process::all_processes() {
        for process in all_procs.filter_map(|res| res.ok()) {
            let Ok(fds) = process.fd() else { continue };
            let Ok(stat) = process.stat() else { continue };
            let proc_name = stat.comm;
            let proc_info = ProcessInfo::new(&proc_name, stat.pid as u32);
            for fd in fds.filter_map(|res| res.ok()) {
                if let FDTarget::Socket(inode) = fd.target {
                    inode_to_proc.insert(inode, proc_info.clone());
                }
            }
        }
    }

    macro_rules! insert_proto {
        ($source: expr, $proto: expr) => {
            let entries = $source.into_iter().filter_map(|res| res.ok()).flatten();
            for entry in entries {
                if let Some(proc_info) = inode_to_proc.get(&entry.inode) {
                    let socket = LocalSocket {
                        ip: entry.local_address.ip(),
                        port: entry.local_address.port(),
                        protocol: $proto,
                    };
                    open_sockets.insert(socket, proc_info.clone());
                }
            }
        };
    }

    insert_proto!([procfs::net::tcp(), procfs::net::tcp6()], Protocol::Tcp);
    insert_proto!([procfs::net::udp(), procfs::net::udp6()], Protocol::Udp);

    OpenSockets {
        sockets_to_procs: open_sockets,
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    use pnet::{
        datalink::NetworkInterface,
        ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network},
    };

    use super::local_addresses_from_interfaces;

    fn interface(name: &str, is_up: bool, ips: Vec<IpNetwork>) -> NetworkInterface {
        NetworkInterface {
            name: name.to_string(),
            description: String::new(),
            index: 0,
            mac: None,
            ips,
            flags: u32::from(is_up),
        }
    }

    #[test]
    fn includes_ipv4_and_ipv6_addresses_from_all_active_interfaces() {
        let local_addresses = local_addresses_from_interfaces(vec![
            interface(
                "eth0",
                true,
                vec![IpNetwork::V4(
                    Ipv4Network::new(Ipv4Addr::new(192, 168, 1, 10), 24).unwrap(),
                )],
            ),
            interface(
                "tun0",
                true,
                vec![IpNetwork::V6(
                    Ipv6Network::new(Ipv6Addr::LOCALHOST, 128).unwrap(),
                )],
            ),
        ]);

        assert_eq!(
            local_addresses,
            vec![
                IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)),
                IpAddr::V6(Ipv6Addr::LOCALHOST),
            ]
        );
    }

    #[test]
    fn skips_addresses_from_inactive_interfaces() {
        let local_addresses = local_addresses_from_interfaces(vec![
            interface(
                "eth0",
                false,
                vec![IpNetwork::V4(
                    Ipv4Network::new(Ipv4Addr::new(10, 0, 0, 10), 24).unwrap(),
                )],
            ),
            interface(
                "wlan0",
                true,
                vec![IpNetwork::V4(
                    Ipv4Network::new(Ipv4Addr::new(10, 0, 0, 11), 24).unwrap(),
                )],
            ),
        ]);

        assert_eq!(
            local_addresses,
            vec![IpAddr::V4(Ipv4Addr::new(10, 0, 0, 11))]
        );
    }

    #[test]
    fn returns_empty_when_no_active_interfaces_have_addresses() {
        let local_addresses = local_addresses_from_interfaces(vec![
            interface("eth0", true, vec![]),
            interface(
                "wlan0",
                false,
                vec![IpNetwork::V4(
                    Ipv4Network::new(Ipv4Addr::new(10, 0, 0, 11), 24).unwrap(),
                )],
            ),
        ]);

        assert!(local_addresses.is_empty());
    }
}
