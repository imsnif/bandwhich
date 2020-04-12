#[cfg(target_os = "linux")]
pub(self) mod linux;

#[cfg(any(target_os = "macos", target_os = "freebsd"))]
pub(self) mod lsof;

#[cfg(any(target_os = "macos", target_os = "freebsd"))]
mod lsof_utils;

mod errors;
mod shared;

pub use linux::*;
pub use shared::*;
