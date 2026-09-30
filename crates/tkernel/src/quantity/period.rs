//! OCCT `Quantity_Period`: nonnegative intervals with microsecond resolution.
use crate::{Error, ErrorKind, error::seconds_i32};

/// A period represented by signed 32-bit seconds and normalized microseconds.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Period {
    seconds: i32,
    micros: i32,
}

impl Period {
    /// Constructs an interval, carrying excess microseconds into seconds.
    pub fn new(seconds: i32, micros: i32) -> Result<Self, Error> {
        if !Self::is_valid(seconds, micros) {
            return Err(Error::new(
                ErrorKind::PeriodDefinition,
                "Quantity_Period::SetValues invalid parameters",
            ));
        }
        Ok(Self {
            seconds: seconds_i32(i64::from(seconds) + i64::from(micros / 1_000_000))?,
            micros: micros % 1_000_000,
        })
    }

    /// Constructs an interval from days, hours, minutes, seconds, milliseconds and microseconds.
    /// Like OCCT, construction validates the combined totals, not each component.
    pub fn from_components(
        dd: i32,
        hh: i32,
        mn: i32,
        ss: i32,
        mis: i32,
        mics: i32,
    ) -> Result<Self, Error> {
        Self::new(
            seconds_i32(
                i64::from(dd) * 86400 + i64::from(hh) * 3600 + i64::from(mn) * 60 + i64::from(ss),
            )?,
            seconds_i32(i64::from(mis) * 1000 + i64::from(mics))?,
        )
    }

    /// Returns normalized seconds and microseconds (`Values` with two outputs).
    pub const fn values(self) -> (i32, i32) {
        (self.seconds, self.micros)
    }

    /// Returns days, hours, minutes, seconds, milliseconds and remaining microseconds.
    pub const fn components(self) -> (i32, i32, i32, i32, i32, i32) {
        (
            self.seconds / 86400,
            self.seconds % 86400 / 3600,
            self.seconds % 3600 / 60,
            self.seconds % 60,
            self.micros / 1000,
            self.micros % 1000,
        )
    }

    /// Checks the two-component `IsValid` contract (nonnegative inputs).
    pub const fn is_valid(seconds: i32, micros: i32) -> bool {
        seconds >= 0 && micros >= 0
    }
}

impl Period {
    /// Checks each of the six components for nonnegativity, as OCCT `IsValid` does.
    pub const fn is_valid_components(
        dd: i32,
        hh: i32,
        mn: i32,
        ss: i32,
        mis: i32,
        mics: i32,
    ) -> bool {
        dd >= 0 && hh >= 0 && mn >= 0 && ss >= 0 && mis >= 0 && mics >= 0
    }

    /// Replaces the interval; an error leaves the original value unchanged.
    pub fn set_values(&mut self, seconds: i32, micros: i32) -> Result<(), Error> {
        *self = Self::new(seconds, micros)?;
        Ok(())
    }

    /// Replaces the interval from six components; validates the combined totals.
    pub fn set_components(
        &mut self,
        dd: i32,
        hh: i32,
        mn: i32,
        ss: i32,
        mis: i32,
        mics: i32,
    ) -> Result<(), Error> {
        *self = Self::from_components(dd, hh, mn, ss, mis, mics)?;
        Ok(())
    }

    /// Adds an interval, carrying microseconds. Fails on signed 32-bit seconds overflow.
    #[allow(clippy::should_implement_trait)] // Also implements Add; explicit OCCT method retained.
    pub fn add(self, other: Self) -> Result<Self, Error> {
        Self::new(
            seconds_i32(i64::from(self.seconds) + i64::from(other.seconds))?,
            self.micros + other.micros,
        )
    }

    /// Returns the absolute difference, as OCCT `Subtract` does.
    pub fn subtract(self, other: Self) -> Result<Self, Error> {
        Ok(Self::from_total_micros(
            (self.total_micros() - other.total_micros()).abs(),
        ))
    }

    /// Tests equality in const contexts.
    pub const fn is_equal(self, other: Self) -> bool {
        self.seconds == other.seconds && self.micros == other.micros
    }

    /// Tests strict chronological length ordering in const contexts.
    pub const fn is_shorter(self, other: Self) -> bool {
        self.seconds < other.seconds
            || (self.seconds == other.seconds && self.micros < other.micros)
    }

    /// Tests whether this interval is strictly longer.
    pub const fn is_longer(self, other: Self) -> bool {
        other.is_shorter(self)
    }

    pub(crate) const fn total_micros(self) -> i64 {
        self.seconds as i64 * 1_000_000 + self.micros as i64
    }

    // Only called with a difference of two nonnegative, representable values.
    pub(crate) const fn from_total_micros(total: i64) -> Self {
        Self {
            seconds: (total / 1_000_000) as i32,
            micros: (total % 1_000_000) as i32,
        }
    }
}

impl std::ops::Add for Period {
    type Output = Result<Self, Error>;
    fn add(self, rhs: Self) -> Self::Output {
        self.add(rhs)
    }
}

impl std::ops::Sub for Period {
    type Output = Result<Self, Error>;
    fn sub(self, rhs: Self) -> Self::Output {
        self.subtract(rhs)
    }
}
