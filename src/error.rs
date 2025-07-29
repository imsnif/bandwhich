//! Custom error types for bandwhich
//!
//! This module provides structured error handling throughout the application,
//! replacing generic error types with domain-specific ones for better
//! error messages and recovery strategies.

use std::io;
use thiserror::Error;

/// Main error type for bandwhich operations
#[derive(Debug, Error)]
pub enum BandwhichError {
    /// Terminal initialization or operation failed
    #[error("Terminal error: {0}")]
    Terminal(#[from] io::Error),

    /// Thread spawning or joining failed
    #[error("Thread error: {0}")]
    Thread(String),

    /// Lock acquisition failed (poisoned mutex)
    #[error("Lock poisoned: {0}")]
    LockPoisoned(String),

    /// DNS resolution error
    #[error("DNS error: {0}")]
    Dns(String),

    /// Network interface error
    #[error("Network interface error: {0}")]
    NetworkInterface(String),

    /// Process information retrieval error
    #[error("Process info error: {0}")]
    ProcessInfo(String),

    /// Configuration or CLI argument error
    #[error("Configuration error: {0}")]
    Config(String),
}

/// Result type alias for bandwhich operations
pub type Result<T> = std::result::Result<T, BandwhichError>;

/// Convert from std::sync::PoisonError to BandwhichError
impl<T> From<std::sync::PoisonError<T>> for BandwhichError {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        BandwhichError::LockPoisoned(format!("Mutex poisoned: {}", err))
    }
}
