//! Terminal user interface components
//!
//! This module provides the display functionality for bandwhich:
//! - Terminal UI rendering using ratatui
//! - Raw output mode for piping to other programs
//! - UI state management and component rendering

mod components;
mod raw_terminal_backend;
mod ui;
mod ui_state;

pub use components::*;
pub use raw_terminal_backend::*;
pub use ui::*;
pub use ui_state::*;
