use ::std::fmt;

pub struct DisplayBandwidth {
    pub bandwidth: f64,
    pub as_rate: bool,
}

impl fmt::Display for DisplayBandwidth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = if self.as_rate { "" } else { "ps" };
        if self.bandwidth > 999_999_999.0 {
            write!(f, "{:.2}GB{}", self.bandwidth / 1_000_000_000.0, suffix)
        } else if self.bandwidth > 999_999.0 {
            write!(f, "{:.2}MB{}", self.bandwidth / 1_000_000.0, suffix)
        } else if self.bandwidth > 999.0 {
            write!(f, "{:.2}KB{}", self.bandwidth / 1000.0, suffix)
        } else {
            write!(f, "{}B{}", self.bandwidth, suffix)
        }
    }
}
