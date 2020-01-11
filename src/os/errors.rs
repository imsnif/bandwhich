use std::fmt;

use failure::{Backtrace, Context, Fail};


#[derive(Debug)]
pub struct MyError {
    inner: Context<MyErrorKind>,
}
impl MyError{
    pub fn new(kind:MyErrorKind)->MyError{
        MyError::from(kind)
    }
}

impl From<MyErrorKind> for MyError {
    fn from(kind: MyErrorKind) -> MyError {
        MyError {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<MyErrorKind>> for MyError {
    fn from(inner: Context<MyErrorKind>) -> MyError {
        MyError { inner}
    }
}
#[derive(Clone, Eq, PartialEq, Debug, Fail)]
pub enum MyErrorKind {
    #[fail(display = "Type error message {}", _0)]
    TypeError(String),
    #[fail(display = "Other error message {}", _0)]
    OtherError(String),
}
impl Fail for MyError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

