use thiserror::Error;

#[derive(Clone, Eq, PartialEq, Debug, Error)]
pub enum GetInterfaceError {
    #[error("{0}")]
    PermissionError(String),
    #[error("{0}")]
    OtherError(String),
}
