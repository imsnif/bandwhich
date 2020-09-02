use ::std::fmt;

pub struct DisplayBandwidth {
    pub bandwidth: f64,
    pub as_rate: bool,
}

impl fmt::Display for DisplayBandwidth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = if self.as_rate { "ps" } else { "" };
        if self.bandwidth > 999_999_999_999.0 {
            // 1024 * 1024 * 1024 * 1024
            write!(f, "{:.2}TiB{}", self.bandwidth / 1_099_511_627_776.0, suffix)
        } else if self.bandwidth > 999_999_999.0 {
            write!(f, "{:.2}GiB{}", self.bandwidth / 1_073_741_824.0, suffix) // 1024 * 1024 * 1024
        } else if self.bandwidth > 999_999.0 {
            write!(f, "{:.2}MiB{}", self.bandwidth / 1_048_576.0, suffix) //  1024 * 1024
        } else if self.bandwidth > 999.0 {
            write!(f, "{:.2}KiB{}", self.bandwidth / 1024.0, suffix)
        } else {
            write!(f, "{}B{}", self.bandwidth, suffix)
        }
    }
}
