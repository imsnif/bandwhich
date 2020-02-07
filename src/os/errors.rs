use std::fmt;

use failure::{Backtrace, Context, Fail};

#[derive(Debug)]
pub struct GetInterfaceError {
    inner: Context<GetInterfaceErrorKind>,
}
impl GetInterfaceError {
    pub fn new(kind: GetInterfaceErrorKind) -> GetInterfaceError {
        GetInterfaceError::from(kind)
    }
    pub fn kind(&self) -> GetInterfaceErrorKind {
        self.inner.get_context().clone()
    }
}

impl From<GetInterfaceErrorKind> for GetInterfaceError {
    fn from(kind: GetInterfaceErrorKind) -> GetInterfaceError {
        GetInterfaceError {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<GetInterfaceErrorKind>> for GetInterfaceError {
    fn from(inner: Context<GetInterfaceErrorKind>) -> GetInterfaceError {
        GetInterfaceError { inner }
    }
}
#[derive(Clone, Eq, PartialEq, Debug, Fail)]
pub enum GetInterfaceErrorKind {
    #[fail(display = "{}::Permission error message", _0)]
    PermissionError(String),
    #[fail(display = "{}::", _0)]
    OtherError(String),
}
impl Fail for GetInterfaceError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for GetInterfaceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}
