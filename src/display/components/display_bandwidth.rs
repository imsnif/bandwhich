use ::std::fmt;

pub struct DisplayString<'a>(pub &'a str);

impl fmt::Display for DisplayString<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct DisplayBandwidth(pub f64);

impl fmt::Display for DisplayBandwidth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 > 999_999_999.0 {
            write!(f, "{:.2}GBps", self.0 / 1_000_000_000.0)
        } else if self.0 > 999_999.0 {
            write!(f, "{:.2}MBps", self.0 / 1_000_000.0)
        } else if self.0 > 999.0 {
            write!(f, "{:.2}KBps", self.0 / 1000.0)
        } else {
            write!(f, "{}Bps", self.0)
        }
    }
}
