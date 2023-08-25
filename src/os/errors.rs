#[derive(Clone, Eq, PartialEq, Debug, thiserror::Error)]
pub enum GetInterfaceError {
    #[error("Permission error: {0}")]
    PermissionError(String),
    #[error("Other error: {0}")]
    OtherError(String),
}
