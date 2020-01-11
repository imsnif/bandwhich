use failure::{Backtrace, Context, Fail};

#[derive(Debug)]
pub struct MyError {
    inner: Context<MyErrorKind>,
}
impl MyError {
    pub fn kind(&self) -> MyErrorKind {
        *self.inner.get_context()
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
        MyError { inner: inner }
    }
}
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
enum MyErrorKind {
    #[fail(display = "Unsupported interface type")]
    WrongInterface,
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
