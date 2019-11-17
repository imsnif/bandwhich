use std::process::{Command};
use std::ffi::OsStr;
use regex::{Regex};
use crate::network::Protocol;
use std::net::IpAddr;
use lazy_static::lazy_static;

#[derive(Debug, Clone)]
pub struct RawConnection {
    ip: String,
    local_port: String,
    remote_port: String,
    protocol: String,
    pub process_name: String,
}

lazy_static! {
    static ref CONNECTION_REGEX: Regex = Regex::new(r"([^\s]+).*(TCP|UDP).*:(.*)->(.*):(\d*)(\s|$)").unwrap();
}

impl RawConnection {
    pub fn new(raw_line: &str) -> Option<RawConnection> {
        let raw_connection_iter = CONNECTION_REGEX.captures_iter(raw_line).filter_map(|cap| {
            let process_name = String::from(cap.get(1).unwrap().as_str());
            let protocol = String::from(cap.get(2).unwrap().as_str());
            let local_port = String::from(cap.get(3).unwrap().as_str());
            let ip = String::from(cap.get(4).unwrap().as_str());
            let remote_port = String::from(cap.get(5).unwrap().as_str());
            let connection = RawConnection { process_name, ip, local_port, remote_port, protocol };
            Some(connection)
        });
        let raw_connection_vec = raw_connection_iter.map(|m| m).collect::<Vec<_>>();
        if raw_connection_vec.is_empty() {
            None
        } else {
            Some(raw_connection_vec[0].clone())
        }
    }

    pub fn get_protocol(&self) -> Protocol {
        return Protocol::from_string(&self.protocol).unwrap();
    }

    pub fn get_ip_address(&self) -> IpAddr {
        return IpAddr::V4(self.ip.parse().unwrap());
    }

    pub fn get_remote_port(&self) -> u16 {
        return self.remote_port.parse::<u16>().unwrap();
    }

    pub fn get_local_port(&self) -> u16 {
        return self.local_port.parse::<u16>().unwrap();
    }
}

pub fn get_connections<'a>() -> RawConnections {
    let content = run(&["-n","-P", "-i4"]);
    RawConnections::new(content)
}

fn run<'a, I, S>(args: I) -> String
    where I: IntoIterator<Item=S>, S: AsRef<OsStr>
{
    let output = Command::new("lsof")
        .args(args)
        .output()
        .expect("failed to execute process");

    String::from_utf8(output.stdout).unwrap()
}

pub struct RawConnections {
    content: Vec<RawConnection>,
}

impl RawConnections {
    pub fn new(content: String) -> RawConnections {
        let lines: Vec<RawConnection> = content.lines()
            .flat_map(|string| RawConnection::new(string))
            .collect();

        RawConnections{ content: lines }
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
    fn test_iterator() {
        let iterator = RawConnections::new(String::from(LINE_RAW_OUTPUT));
        let mut worked = false;
        for raw_connection in iterator {
            worked = true;
            assert_eq!(raw_connection.process_name, String::from("ProcessName"));
        }
        assert!(worked);
    }

    #[test]
    fn test_raw_connection_is_created_from_raw_output() {
        let connection = RawConnection::new(LINE_RAW_OUTPUT);
        assert!(connection.is_some());
    }

    #[test]
    fn test_raw_connection_is_not_created_from_wrong_raw_output() {
        let connection = RawConnection::new("not a process");
        assert!(connection.is_none());
    }

    #[test]
    fn test_raw_connection_parse_remote_port() {
        let connection = RawConnection::new(LINE_RAW_OUTPUT).unwrap();
        assert_eq!(connection.get_remote_port(), 2222);
    }

    #[test]
    fn test_raw_connection_parse_local_port() {
        let connection = RawConnection::new(LINE_RAW_OUTPUT).unwrap();
        assert_eq!(connection.get_local_port(), 1111);
    }

    #[test]
    fn test_raw_connection_parse_ip_address() {
        let connection = RawConnection::new(LINE_RAW_OUTPUT).unwrap();
        assert_eq!(connection.get_ip_address().to_string(), String::from("198.252.206.25"));
    }

    #[test]
    fn test_raw_connection_parse_protocol() {
        let connection = RawConnection::new(LINE_RAW_OUTPUT).unwrap();
        assert_eq!(connection.get_protocol(), Protocol::Udp);
    }

    #[test]
    fn test_raw_connection_parse_process_name() {
        let connection = RawConnection::new(LINE_RAW_OUTPUT).unwrap();
        assert_eq!(connection.process_name, String::from("ProcessName"));
    }
}
