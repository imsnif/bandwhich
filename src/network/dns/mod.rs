use std::{collections::HashMap, net::IpAddr};

mod client;
mod resolver;

pub use client::*;
pub use resolver::*;

pub type IpTable = HashMap<IpAddr, String>;
