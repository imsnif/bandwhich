use crate::network::Protocol;
use lazy_static::lazy_static;
use regex::Regex;
use std::ffi::OsStr;
use std::net::IpAddr;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct RawConnection {
    remote_ip: String,
    local_ip: String,
    local_port: String,
    remote_port: String,
    protocol: String,
    pub process_name: String,
}

lazy_static! {
    static ref CONNECTION_REGEX: Regex =
        Regex::new(r"\[?([^\s\]]*)\]?:(\d+)->\[?([^\s\]]*)\]?:(\d+)").unwrap();
    static ref LISTEN_REGEX: Regex = Regex::new(r"\[?([^\s\[\]]*)\]?:(.*)").unwrap();
}

fn get_null_addr(ip_type: &str) -> &str {
    if ip_type.contains('4') {
        "0.0.0.0"
    } else {
        "::0"
    }
}

impl RawConnection {
    pub fn new(raw_line: &str) -> Option<RawConnection> {
        // Example row
        // com.apple   664     user  198u  IPv4 0xeb179a6650592b8d      0t0    TCP 192.168.1.187:58535->1.2.3.4:443 (ESTABLISHED)
        let columns: Vec<&str> = raw_line.split_ascii_whitespace().collect();
        if columns.len() < 9 {
            return None;
        }
        let process_name = columns[0].replace("\\x20", " ");
        // Unneeded
        // let pid = columns[1];
        // let username = columns[2];
        // let fd = columns[3];

        // IPv4 or IPv6
        let ip_type = columns[4];
        // let device = columns[5];
        // let size = columns[6];
        // UDP/TCP
        let protocol = columns[7].to_ascii_uppercase();
        if protocol != "TCP" && protocol != "UDP" {
            return None;
        }
        let connection_str = columns[8];
        // "(LISTEN)" or "(ESTABLISHED)",  this column may or may not be present
        // let connection_state = columns[9];
        // If this socket is in a "connected" state
        if let Some(caps) = CONNECTION_REGEX.captures(connection_str) {
            // Example
            // 192.168.1.187:64230->0.1.2.3:5228
            // *:*
            // *:4567
            let local_ip = String::from(caps.get(1).unwrap().as_str());
            let local_port = String::from(caps.get(2).unwrap().as_str());
            let remote_ip = String::from(caps.get(3).unwrap().as_str());
            let remote_port = String::from(caps.get(4).unwrap().as_str());
            let connection = RawConnection {
                local_ip,
                local_port,
                remote_ip,
                remote_port,
                protocol,
                process_name,
            };
            Some(connection)
        } else if let Some(caps) = LISTEN_REGEX.captures(connection_str) {
            let local_ip = if caps.get(1).unwrap().as_str() == "*" {
                get_null_addr(ip_type)
            } else {
                caps.get(1).unwrap().as_str()
            };
            let local_ip = String::from(local_ip);
            let local_port = String::from(if caps.get(2).unwrap().as_str() == "*" {
                "0"
            } else {
                caps.get(2).unwrap().as_str()
            });
            let remote_ip = String::from(get_null_addr(ip_type));
            let remote_port = String::from("0");
            let connection = RawConnection {
                local_ip,
                local_port,
                remote_ip,
                remote_port,
                protocol,
                process_name,
            };
            Some(connection)
        } else {
            None
        }
    }

    pub fn get_protocol(&self) -> Protocol {
        Protocol::from_str(&self.protocol).unwrap()
    }

    pub fn get_local_ip(&self) -> IpAddr {
        self.local_ip.parse().unwrap()
    }

    pub fn get_local_port(&self) -> u16 {
        self.local_port.parse::<u16>().unwrap()
    }
}

pub fn get_connections() -> RawConnections {
    let content = run(&["-n", "-P", "-i4", "-i6", "+c", "0"]);
    RawConnections::new(content)
}

fn run<I, S>(args: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new("lsof")
        .args(args)
        .output()
        .expect("failed to execute process");

    String::from_utf8_lossy(&output.stdout).into_owned()
}

pub struct RawConnections {
    content: Vec<RawConnection>,
}

impl RawConnections {
    pub fn new(content: String) -> RawConnections {
        let lines: Vec<RawConnection> = content
            .lines()
            .flat_map(|string| RawConnection::new(string))
            .collect();

        RawConnections { content: lines }
    }
}

impl Iterator for RawConnections {
    type Item = RawConnection;

    fn next(&mut self) -> Option<Self::Item> {
        self.content.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const IPV6_LINE_RAW_OUTPUT: &str = "ProcessName     29266 user    9u  IPv6 0x5d53dfe5445cee01      0t0  UDP [fe80:4::aede:48ff:fe00:1122]:1111->[fe80:4::aede:48ff:fe33:4455]:2222";
    const LINE_RAW_OUTPUT: &str = "ProcessName 29266 user   39u  IPv4 0x28ffb9c0021196bf      0t0  UDP 192.168.0.1:1111->198.252.206.25:2222";
    const FULL_RAW_OUTPUT: &str = r#"
com.apple   590 etoledom  193u  IPv4 0x28ffb9c041115627      0t0  TCP 192.168.1.37:60298->31.13.83.36:443 (ESTABLISHED)
com.apple   590 etoledom  198u  IPv4 0x28ffb9c04110ea8f      0t0  TCP 192.168.1.37:60299->31.13.83.8:443 (ESTABLISHED)
com.apple   590 etoledom  203u  IPv4 0x28ffb9c04110ea8f      0t0  TCP 192.168.1.37:60299->31.13.83.8:443 (ESTABLISHED)
com.apple   590 etoledom  204u  IPv4 0x28ffb9c04111253f      0t0  TCP 192.168.1.37:60374->140.82.114.26:443
"#;

    #[test]
    fn test_iterator_multiline() {
        let iterator = RawConnections::new(String::from(FULL_RAW_OUTPUT));
        let connections: Vec<RawConnection> = iterator.collect();
        assert_eq!(connections.len(), 4);
    }

    #[test]
    fn test_raw_connection_is_created_from_raw_output_ipv4() {
        test_raw_connection_is_created_from_raw_output(LINE_RAW_OUTPUT);
    }
    #[test]
    fn test_raw_connection_is_created_from_raw_output_ipv6() {
        test_raw_connection_is_created_from_raw_output(IPV6_LINE_RAW_OUTPUT);
    }
    fn test_raw_connection_is_created_from_raw_output(raw_output: &str) {
        let connection = RawConnection::new(raw_output);
        assert!(connection.is_some());
    }

    #[test]
    fn test_raw_connection_is_not_created_from_wrong_raw_output() {
        let connection = RawConnection::new("not a process");
        assert!(connection.is_none());
    }

    #[test]
    fn test_raw_connection_parse_local_port_ipv4() {
        test_raw_connection_parse_local_port(LINE_RAW_OUTPUT);
    }
    #[test]
    fn test_raw_connection_parse_local_port_ipv6() {
        test_raw_connection_parse_local_port(IPV6_LINE_RAW_OUTPUT);
    }
    fn test_raw_connection_parse_local_port(raw_output: &str) {
        let connection = RawConnection::new(raw_output).unwrap();
        assert_eq!(connection.get_local_port(), 1111);
    }

    #[test]
    fn test_raw_connection_parse_protocol_ipv4() {
        test_raw_connection_parse_protocol(LINE_RAW_OUTPUT);
    }
    #[test]
    fn test_raw_connection_parse_protocol_ipv6() {
        test_raw_connection_parse_protocol(IPV6_LINE_RAW_OUTPUT);
    }
    fn test_raw_connection_parse_protocol(raw_line: &str) {
        let connection = RawConnection::new(raw_line).unwrap();
        assert_eq!(connection.get_protocol(), Protocol::Udp);
    }

    #[test]
    fn test_raw_connection_parse_process_name_ipv4() {
        test_raw_connection_parse_process_name(LINE_RAW_OUTPUT);
    }
    #[test]
    fn test_raw_connection_parse_process_name_ipv6() {
        test_raw_connection_parse_process_name(IPV6_LINE_RAW_OUTPUT);
    }
    fn test_raw_connection_parse_process_name(raw_line: &str) {
        let connection = RawConnection::new(raw_line).unwrap();
        assert_eq!(connection.process_name, String::from("ProcessName"));
    }
}
