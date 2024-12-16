use std::{
    ffi::OsStr,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    process::Command,
    str::FromStr,
};

use eyre::{bail, Context, OptionExt};
use log::warn;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::{
    network::{LocalSocket, Protocol},
    os::ProcessInfo,
    OpenSockets,
};

pub(crate) fn get_open_sockets() -> OpenSockets {
    let sockets_to_procs = get_connections()
        .into_iter()
        .map(|conn| (conn.as_local_socket(), conn.proc_info))
        .collect();

    OpenSockets { sockets_to_procs }
}

fn get_connections() -> Vec<Connection> {
    let raw_lines = run_lsof(["-n", "-P", "-i4", "-i6", "+c", "0"]);

    raw_lines
        .lines()
        .map(Connection::from_str)
        .filter_map(|res| res.inspect_err(|err| warn!("{err}")).ok())
        .collect()
}

fn run_lsof<I, S>(args: I) -> String
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

/// Helper enum for strong typing.
#[derive(Copy, Clone, Debug)]
enum IpVer {
    V4,
    V6,
}
impl IpVer {
    fn get_null_addr(&self) -> IpAddr {
        match self {
            Self::V4 => Ipv4Addr::UNSPECIFIED.into(),
            Self::V6 => Ipv6Addr::UNSPECIFIED.into(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Connection {
    local: (IpAddr, u16),
    /// None if listening
    remote: Option<(IpAddr, u16)>,
    protocol: Protocol,
    proc_info: ProcessInfo,
}

impl FromStr for Connection {
    type Err = eyre::Report;

    fn from_str(raw_line: &str) -> Result<Self, Self::Err> {
        // Example row
        // com.apple   664     user  198u  IPv4 0xeb179a6650592b8d      0t0    TCP 192.168.1.187:58535->1.2.3.4:443 (ESTABLISHED)
        let mut fields = raw_line.split_ascii_whitespace();

        let process_name = fields
            .next()
            .ok_or_eyre("Missing field: process name")?
            .replace("\\x20", " ");
        let pid = {
            let pid_str = fields.next().ok_or_eyre("Missing field: PID")?;
            pid_str
                .parse()
                .wrap_err_with(|| format!("PID `{pid_str}` failed parsing"))?
        };
        let proc_info = ProcessInfo::new(&process_name, pid);

        let _user = fields.next().ok_or_eyre("Missing field: user")?;
        let _fd = fields.next().ok_or_eyre("Missing field: file descriptor")?;

        let ip_ver = if fields
            .next()
            .ok_or_eyre("Missing field: IP version")?
            .contains('4')
        {
            IpVer::V4
        } else {
            IpVer::V6
        };

        let _device = fields.next().ok_or_eyre("Missing field: device")?;
        let _size = fields.next().ok_or_eyre("Missing field: size")?;

        let protocol = {
            let proto_str = fields.next().ok_or_eyre("Missing field: protocol")?;
            proto_str.parse().wrap_err_with(|| {
                format!("Protocol `{proto_str}` failed parsing for process `{process_name}`")
            })?
        };

        let connection_str = fields.next().ok_or_eyre("Missing field: connection")?;
        static ESTABLISHED_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"\[?([^\s\]]*)\]?:(\d+)->\[?([^\s\]]*)\]?:(\d+)").unwrap());
        static LISTENING_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"\[?([^\s\[\]]*)\]?:(.*)").unwrap());
        let (local, remote) = if let Some(caps) = ESTABLISHED_REGEX.captures(connection_str) {
            macro_rules! parse {
                ($n: expr, $name: expr) => {{
                    let s = caps.get($n).unwrap().as_str();
                    s.parse().wrap_err_with(|| {
                        format!(
                            "{} `{s}` failed parsing for process `{process_name}`",
                            $name
                        )
                    })
                }};
            }
            let local_ip = parse!(1, "Local IP")?;
            let local_port = parse!(2, "Local port")?;
            let remote_ip = parse!(3, "Remote IP")?;
            let remote_port = parse!(4, "Remote port")?;
            ((local_ip, local_port), Some((remote_ip, remote_port)))
        } else if let Some(caps) = LISTENING_REGEX.captures(connection_str) {
            let local_ip = match caps.get(1).unwrap().as_str() {
                "*" => ip_ver.get_null_addr(),
                ip => ip.parse().wrap_err_with(|| {
                    format!("Local IP `{ip}` failed parsing for process `{process_name}`")
                })?,
            };
            let local_port = match caps.get(2).unwrap().as_str() {
                "*" => 0,
                port => port.parse().wrap_err_with(|| {
                    format!("Local port `{port}` failed parsing for process `{process_name}`")
                })?,
            };
            ((local_ip, local_port), None)
        } else {
            bail!(
                r#"lsof output line matches matches neither established nor listening format: "{raw_line}""#
            );
        };

        // "(LISTEN)" or "(ESTABLISHED)",  this column may or may not be present
        let _connection_state = fields.next(); // allow missing

        Ok(Self {
            local,
            remote,
            protocol,
            proc_info,
        })
    }
}

impl Connection {
    fn as_local_socket(&self) -> LocalSocket {
        let &Self {
            local: (ip, port),
            protocol,
            ..
        } = self;
        LocalSocket { ip, port, protocol }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    const IPV6_LINE_RAW_OUTPUT: &str = "ProcessName     29266 user    9u  IPv6 0x5d53dfe5445cee01      0t0  UDP [fe80:4::aede:48ff:fe00:1122]:1111->[fe80:4::aede:48ff:fe33:4455]:2222";
    const IPV4_LINE_RAW_OUTPUT: &str = "ProcessName 29266 user   39u  IPv4 0x28ffb9c0021196bf      0t0  UDP 192.168.0.1:1111->198.252.206.25:2222";
    const FULL_RAW_OUTPUT: &str = r#"
com.apple   590 etoledom  193u  IPv4 0x28ffb9c041115627      0t0  TCP 192.168.1.37:60298->31.13.83.36:443 (ESTABLISHED)
com.apple   590 etoledom  198u  IPv4 0x28ffb9c04110ea8f      0t0  TCP 192.168.1.37:60299->31.13.83.8:443 (ESTABLISHED)
com.apple   590 etoledom  203u  IPv4 0x28ffb9c04110ea8f      0t0  TCP 192.168.1.37:60299->31.13.83.8:443 (ESTABLISHED)
com.apple   590 etoledom  204u  IPv4 0x28ffb9c04111253f      0t0  TCP 192.168.1.37:60374->140.82.114.26:443
"#;

    #[test]
    fn test_multiline_parse() {
        for res in FULL_RAW_OUTPUT.lines().map(Connection::from_str) {
            let _conn = res.unwrap();
        }
    }

    #[rstest]
    #[case(IPV4_LINE_RAW_OUTPUT, "ProcessName", Protocol::Udp, 1111)]
    #[case(IPV6_LINE_RAW_OUTPUT, "ProcessName", Protocol::Udp, 1111)]
    fn test_parse(
        #[case] raw: &str,
        #[case] process_name: &str,
        #[case] protocol: Protocol,
        #[case] port: u16,
    ) {
        let conn = Connection::from_str(raw).unwrap();
        assert_eq!(conn.proc_info.name, process_name);
        assert_eq!(conn.protocol, protocol);
        assert_eq!(conn.local.1, port);
    }
}
