//! Network packet capture and analysis
//!
//! This module provides the core networking functionality for bandwhich:
//! - Packet sniffing from network interfaces
//! - Connection tracking and bandwidth utilization
//! - DNS resolution for IP addresses

mod connection;
pub mod dns;
mod sniffer;
mod utilization;

pub use connection::*;
pub use sniffer::*;
pub use utilization::*;
