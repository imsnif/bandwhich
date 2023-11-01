use std::fmt;

use clap::ValueEnum;

#[derive(Copy, Clone, Debug)]
pub struct DisplayBandwidth {
    pub bandwidth: f64,
    pub unit_family: BandwidthUnitFamily,
}

impl fmt::Display for DisplayBandwidth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (div, suffix) = self.unit_family.get_unit_for(self.bandwidth);
        write!(f, "{:.2}{suffix}", self.bandwidth / div)
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, ValueEnum)]
pub enum BandwidthUnitFamily {
    #[default]
    /// bytes, in powers 2^10
    BinBytes,
    /// bits, in powers 2^10
    BinBits,
    /// bytes, in powers of 10^3
    SiBytes,
    /// bits, in powers 10^3
    SiBits,
}
impl BandwidthUnitFamily {
    #[inline]
    /// Returns an array of tuples, corresponding to the steps of this unit family.
    ///
    /// Each step contains a divisor, an upper bound, and a unit suffix.
    fn steps(&self) -> [(f64, f64, &'static str); 5] {
        /// The fraction of the next unit the value has to meet to step up.
        const STEP_UP_FRAC: f64 = 0.95;
        const BIN_POW: f64 = 2usize.pow(10) as f64; // 1024

        use BandwidthUnitFamily as F;
        // probably could macro this stuff, but I'm too lazy
        match self {
            F::BinBytes => [
                (1.0, BIN_POW * STEP_UP_FRAC, "B"),
                (BIN_POW, BIN_POW.powi(2) * STEP_UP_FRAC, "KiB"),
                (BIN_POW.powi(2), BIN_POW.powi(3) * STEP_UP_FRAC, "MiB"),
                (BIN_POW.powi(3), BIN_POW.powi(4) * STEP_UP_FRAC, "GiB"),
                (BIN_POW.powi(4), f64::MAX, "TiB"),
            ],
            F::BinBits => [
                (1.0 / 8.0, BIN_POW / 8.0 * STEP_UP_FRAC, "b"),
                (BIN_POW / 8.0, BIN_POW.powi(2) / 8.0 * STEP_UP_FRAC, "Kib"),
                (
                    BIN_POW.powi(2) / 8.0,
                    BIN_POW.powi(3) / 8.0 * STEP_UP_FRAC,
                    "Mib",
                ),
                (
                    BIN_POW.powi(3) / 8.0,
                    BIN_POW.powi(4) / 8.0 * STEP_UP_FRAC,
                    "Gib",
                ),
                (BIN_POW.powi(4) / 8.0, f64::MAX, "Tib"),
            ],
            F::SiBytes => [
                (1.0, 1e3 * STEP_UP_FRAC, "B"),
                (1e3, 1e6 * STEP_UP_FRAC, "kB"),
                (1e6, 1e9 * STEP_UP_FRAC, "MB"),
                (1e9, 1e12 * STEP_UP_FRAC, "GB"),
                (1e12, f64::MAX, "TB"),
            ],
            F::SiBits => [
                (1.0 / 8.0, 1e3 / 8.0 * STEP_UP_FRAC, "b"),
                (1e3 / 8.0, 1e6 / 8.0 * STEP_UP_FRAC, "kb"),
                (1e6 / 8.0, 1e9 / 8.0 * STEP_UP_FRAC, "Mb"),
                (1e9 / 8.0, 1e12 / 8.0 * STEP_UP_FRAC, "Gb"),
                (1e12 / 8.0, f64::MAX, "Tb"),
            ],
        }
    }

    /// Select a unit for a given value, returning its divisor and suffix.
    fn get_unit_for(&self, bytes: f64) -> (f64, &'static str) {
        let (div, _, suffix) = self
            .steps()
            .into_iter()
            .find(|&(_, bound, _)| bound >= bytes)
            .expect("Cannot select an appropriate unit");

        (div, suffix)
    }
}
