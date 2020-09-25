#[cfg(target_os = "linux")]
pub(self) mod linux;

#[cfg(any(target_os = "macos", target_os = "freebsd"))]
pub(self) mod lsof;

#[cfg(any(target_os = "macos", target_os = "freebsd"))]
mod lsof_utils;

#[cfg(target_os = "windows")]
pub(self) mod windows;

mod errors;
pub(crate) mod shared;

pub use shared::*;
