#[cfg(any(target_os = "android", target_os = "linux"))]
mod linux;

#[cfg(any(target_os = "macos", target_os = "freebsd"))]
mod lsof;

#[cfg(any(target_os = "macos", target_os = "freebsd"))]
mod lsof_utils;

#[cfg(target_os = "windows")]
mod windows;

mod errors;
pub(crate) mod shared;

pub use shared::*;
