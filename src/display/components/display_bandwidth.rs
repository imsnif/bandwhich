use std::fmt;

use derivative::Derivative;

use crate::cli::UnitFamily;

#[derive(Copy, Clone, Derivative)]
#[derivative(Debug)]
pub struct DisplayBandwidth {
    #[derivative(Debug(format_with = "fmt_f64"))]
    pub bandwidth: f64,
    pub unit_family: BandwidthUnitFamily,
}

impl fmt::Display for DisplayBandwidth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (div, suffix) = self.unit_family.get_unit_for(self.bandwidth);
        write!(f, "{:.2}{suffix}", self.bandwidth / div)
    }
}

/// Custom formatter with reduced precision.
///
/// Workaround for FP calculation discrepancy between Unix and Windows.
/// See https://github.com/rust-lang/rust/issues/111405#issuecomment-2055964223.
fn fmt_f64(val: &f64, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    write!(f, "{val:.10e}")
}

/// Type wrapper around [`UnitFamily`] to provide extra functionality.
#[derive(Copy, Clone, Derivative, Default, Eq, PartialEq)]
#[derivative(Debug = "transparent")]
pub struct BandwidthUnitFamily(UnitFamily);
impl From<UnitFamily> for BandwidthUnitFamily {
    fn from(value: UnitFamily) -> Self {
        Self(value)
    }
}
impl BandwidthUnitFamily {
    #[inline]
    /// Returns an array of tuples, corresponding to the steps of this unit family.
    ///
    /// Each step contains a divisor, an upper bound, and a unit suffix.
    fn steps(&self) -> [(f64, f64, &'static str); 6] {
        /// The fraction of the next unit the value has to meet to step up.
        const STEP_UP_FRAC: f64 = 0.95;
        /// Binary base: 2^10.
        const BB: f64 = 1024.0;

        use UnitFamily as F;
        // probably could macro this stuff, but I'm too lazy
        match self.0 {
            F::BinBytes => [
                (1.0, BB * STEP_UP_FRAC, "B"),
                (BB, BB.powi(2) * STEP_UP_FRAC, "KiB"),
                (BB.powi(2), BB.powi(3) * STEP_UP_FRAC, "MiB"),
                (BB.powi(3), BB.powi(4) * STEP_UP_FRAC, "GiB"),
                (BB.powi(4), BB.powi(5) * STEP_UP_FRAC, "TiB"),
                (BB.powi(5), f64::MAX, "PiB"),
            ],
            F::BinBits => [
                (1.0 / 8.0, BB / 8.0 * STEP_UP_FRAC, "b"),
                (BB / 8.0, BB.powi(2) / 8.0 * STEP_UP_FRAC, "Kib"),
                (BB.powi(2) / 8.0, BB.powi(3) / 8.0 * STEP_UP_FRAC, "Mib"),
                (BB.powi(3) / 8.0, BB.powi(4) / 8.0 * STEP_UP_FRAC, "Gib"),
                (BB.powi(4) / 8.0, BB.powi(5) / 8.0 * STEP_UP_FRAC, "Tib"),
                (BB.powi(5) / 8.0, f64::MAX, "Pib"),
            ],
            F::SiBytes => [
                (1.0, 1e3 * STEP_UP_FRAC, "B"),
                (1e3, 1e6 * STEP_UP_FRAC, "kB"),
                (1e6, 1e9 * STEP_UP_FRAC, "MB"),
                (1e9, 1e12 * STEP_UP_FRAC, "GB"),
                (1e12, 1e15 * STEP_UP_FRAC, "TB"),
                (1e15, f64::MAX, "PB"),
            ],
            F::SiBits => [
                (1.0 / 8.0, 1e3 / 8.0 * STEP_UP_FRAC, "b"),
                (1e3 / 8.0, 1e6 / 8.0 * STEP_UP_FRAC, "kb"),
                (1e6 / 8.0, 1e9 / 8.0 * STEP_UP_FRAC, "Mb"),
                (1e9 / 8.0, 1e12 / 8.0 * STEP_UP_FRAC, "Gb"),
                (1e12 / 8.0, 1e15 / 8.0 * STEP_UP_FRAC, "Tb"),
                (1e15 / 8.0, f64::MAX, "Pb"),
            ],
        }
    }

    /// Select a unit for a given value, returning its divisor and suffix.
    fn get_unit_for(&self, bytes: f64) -> (f64, &'static str) {
        let Some((div, _, suffix)) = self
            .steps()
            .into_iter()
            .find(|&(_, bound, _)| bound >= bytes)
        else {
            panic!("Cannot select an appropriate unit for {bytes:.2}B.")
        };

        (div, suffix)
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Write;

    use insta::assert_snapshot;
    use itertools::Itertools;
    use strum::IntoEnumIterator;

    use crate::{cli::UnitFamily, display::DisplayBandwidth};

    #[test]
    fn bandwidth_formatting() {
        let test_bandwidths_formatted = UnitFamily::iter()
            .map_into()
            .cartesian_product(
                // I feel like this is a decent selection of values
                (-6..60)
                    .map(|exp| 2f64.powi(exp))
                    .chain((-5..45).map(|exp| 2.5f64.powi(exp)))
                    .chain((-4..38).map(|exp| 3f64.powi(exp)))
                    .chain((-3..26).map(|exp| 5f64.powi(exp))),
            )
            .map(|(unit_family, bandwidth)| DisplayBandwidth {
                bandwidth,
                unit_family,
            })
            .fold(String::new(), |mut buf, b| {
                let _ = writeln!(buf, "{b:?}: {b}");
                buf
            });

        assert_snapshot!(test_bandwidths_formatted);
    }
}
