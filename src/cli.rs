use std::{net::Ipv4Addr, path::PathBuf};

use clap::{Args, Parser};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use derivative::Derivative;

use crate::display::BandwidthUnitFamily;

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

    #[arg(long)]
    /// Enable debug logging to a file
    pub log_to: Option<PathBuf>,

    #[command(flatten)]
    #[derivative(Default(value = "Verbosity::new(0, 0)"))]
    pub verbosity: Verbosity<InfoLevel>,

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
    pub unit_family: BandwidthUnitFamily,

    #[arg(short, long)]
    /// Show total (cumulative) usages
    pub total_utilization: bool,
}
