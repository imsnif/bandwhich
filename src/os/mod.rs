#[cfg(target_os = "linux")]
pub (self) mod linux;

#[cfg(target_os = "macos")]
pub (self) mod macos;

#[cfg(target_os = "macos")]
mod lsof_utils;

mod shared;

pub use shared::*;
