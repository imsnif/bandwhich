use std::{collections::HashMap, net::Ipv4Addr};

mod client;
mod resolver;

pub use client::*;
pub use resolver::*;

pub type IpTable = HashMap<Ipv4Addr, String>;
