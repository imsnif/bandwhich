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
    #[allow(dead_code)]
    Thread(String),

    /// Lock acquisition failed (poisoned mutex)
    #[error("Lock poisoned: {0}")]
    LockPoisoned(String),

    /// DNS resolution error
    #[error("DNS error: {0}")]
    #[allow(dead_code)]
    Dns(String),

    /// Network interface error
    #[error("Network interface error: {0}")]
    #[allow(dead_code)]
    NetworkInterface(String),

    /// Process information retrieval error
    #[error("Process info error: {0}")]
    #[allow(dead_code)]
    ProcessInfo(String),

    /// Configuration or CLI argument error
    #[error("Configuration error: {0}")]
    #[allow(dead_code)]
    Config(String),
}

/// Result type alias for bandwhich operations
#[allow(dead_code)]
pub type Result<T> = std::result::Result<T, BandwhichError>;

/// Convert from std::sync::PoisonError to BandwhichError
impl<T> From<std::sync::PoisonError<T>> for BandwhichError {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        BandwhichError::LockPoisoned(format!("Mutex poisoned: {err}"))
    }
}
