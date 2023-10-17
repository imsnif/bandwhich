use std::fmt;

pub struct DisplayBandwidth {
    pub bandwidth: f64,
}

impl fmt::Display for DisplayBandwidth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // see https://github.com/rust-lang/rust/issues/41620
        let (div, suffix) = if self.bandwidth >= 1e12 {
            (1_099_511_627_776.0, "TiB")
        } else if self.bandwidth >= 1e9 {
            (1_073_741_824.0, "GiB")
        } else if self.bandwidth >= 1e6 {
            (1_048_576.0, "MiB")
        } else if self.bandwidth >= 1e3 {
            (1024.0, "KiB")
        } else {
            (1.0, "B")
        };

        write!(f, "{:.2}{suffix}", self.bandwidth / div)
    }
}
