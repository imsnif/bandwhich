use std::fmt;

pub struct DisplayBandwidth {
    pub bandwidth: f64,
    pub as_rate: bool,
}

impl fmt::Display for DisplayBandwidth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = if self.as_rate { "ps" } else { "" };
        if self.bandwidth > 999_999_999_999.0 {
            // 1024 * 1024 * 1024 * 1024
            write!(f, "{:.2}TiB{suffix}", self.bandwidth / 1_099_511_627_776.0,)
        } else if self.bandwidth > 999_999_999.0 {
            // 1024 * 1024 * 1024
            write!(f, "{:.2}GiB{suffix}", self.bandwidth / 1_073_741_824.0)
        } else if self.bandwidth > 999_999.0 {
            //  1024 * 1024
            write!(f, "{:.2}MiB{suffix}", self.bandwidth / 1_048_576.0)
        } else if self.bandwidth > 999.0 {
            write!(f, "{:.2}KiB{suffix}", self.bandwidth / 1024.0)
        } else {
            write!(f, "{}B{suffix}", self.bandwidth)
        }
    }
}
