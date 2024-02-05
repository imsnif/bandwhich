use std::{
    net::{Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6},
    path::PathBuf,
    str::FromStr,
};

#[derive(Clone, Debug)]
pub enum HostFilter {
    Ipv4Addr(Ipv4Addr),
    Ipv6Addr(Ipv6Addr),
    SocketAddrV4(SocketAddrV4),
    SocketAddrV6(SocketAddrV6),
    Hostname(String),
}

impl FromStr for HostFilter {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(ipv4) = s.parse() {
            Ok(HostFilter::Ipv4Addr(ipv4))
        } else if let Ok(ipv6) = s.parse() {
            Ok(HostFilter::Ipv6Addr(ipv6))
        } else if let Ok(socketv4) = s.parse() {
            Ok(HostFilter::SocketAddrV4(socketv4))
        } else if let Ok(socketv6) = s.parse() {
            Ok(HostFilter::SocketAddrV6(socketv6))
        } else {
            // might need validation
            Ok(HostFilter::Hostname(s.to_string()))
        }
    }
}

use clap::{Args, Parser, ValueEnum, ValueHint};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use derivative::Derivative;
use strum::EnumIter;

#[derive(Clone, Debug, Derivative, Parser)]
#[derivative(Default)]
#[command(name = "bandwhich", version)]
pub struct Opt {
    #[arg(short, long)]
    /// The network interface to listen on, eg. eth0
    pub interface: Option<String>,

    #[arg(short, long)]
    /// Machine friendlier output
    pub raw: bool,

    #[arg(short, long)]
    /// Do not attempt to resolve IPs to their hostnames
    pub no_resolve: bool,

    #[arg(short, long)]
    /// Show DNS queries
    pub show_dns: bool,

    #[arg(short, long)]
    /// A dns server ip to use instead of the system default
    pub dns_server: Option<Ipv4Addr>,

    #[arg(long, value_hint = ValueHint::FilePath)]
    /// Enable debug logging to a file
    pub log_to: Option<PathBuf>,

    #[command(flatten)]
    #[derivative(Default(value = "Verbosity::new(0, 0)"))]
    pub verbosity: Verbosity<InfoLevel>,

    #[arg(short, long)]
    /// exclude ip addres with <-e x.x.x.x>
    /// exclude multiple ip addresses with <-e x.x.x.x -e y.y.y.y>
    /// examples:
    /// IpV4: 127.0.0.1
    /// IpV6: 2001:db8::1 OR 2001:0db8:85a3:0000:0000:8a2e:0370:7334
    /// SocketAddrV4: 127.0.0.1:8080
    /// SocketAddrV6: "[2001:0db8:85a3:0000:0000:8a2e:0370:7334]:8080"
    /// hostname: String
    pub excluded: Option<Vec<HostFilter>>,

    #[command(flatten)]
    pub render_opts: RenderOpts,
}

#[derive(Copy, Clone, Debug, Default, Args)]
pub struct RenderOpts {
    #[arg(short, long)]
    /// Show processes table only
    pub processes: bool,

    #[arg(short, long)]
    /// Show connections table only
    pub connections: bool,

    #[arg(short, long)]
    /// Show remote addresses table only
    pub addresses: bool,

    #[arg(short, long, value_enum, default_value_t)]
    /// Choose a specific family of units
    pub unit_family: UnitFamily,

    #[arg(short, long)]
    /// Show total (cumulative) usages
    pub total_utilization: bool,
}

// IMPRV: it would be nice if we can `#[cfg_attr(not(build), derive(strum::EnumIter))]` this
// unfortunately there is no configuration option for build script detection
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, ValueEnum, EnumIter)]
pub enum UnitFamily {
    #[default]
    /// bytes, in powers of 2^10
    BinBytes,
    /// bits, in powers of 2^10
    BinBits,
    /// bytes, in powers of 10^3
    SiBytes,
    /// bits, in powers of 10^3
    SiBits,
}
