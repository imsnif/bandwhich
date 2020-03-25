use ::std::fmt;

// Should we make a `DisplayMode` enum with `Rate` and `Total`?
// Instead of the ambiguous bool?

// This might be better as a traditional struct now?
pub struct DisplayBandwidth(pub f64, pub bool);

impl fmt::Display for DisplayBandwidth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = if self.1 { "" } else { "ps" }; // Should this be the other way around?
        if self.0 > 999_999_999.0 {
            write!(f, "{:.2}GB{}", self.0 / 1_000_000_000.0, suffix)
        } else if self.0 > 999_999.0 {
            write!(f, "{:.2}MB{}", self.0 / 1_000_000.0, suffix)
        } else if self.0 > 999.0 {
            write!(f, "{:.2}KB{}", self.0 / 1000.0, suffix)
        } else {
            write!(f, "{}B{}", self.0, suffix)
        }
    }
}
