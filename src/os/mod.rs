pub(self) mod open_sockets;

#[cfg(target_os = "windows")]
pub(self) mod windows;

mod errors;
mod shared;

pub use shared::*;
