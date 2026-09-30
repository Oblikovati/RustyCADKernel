//! OCCT `Quantity_Date`: elapsed time from January 1, 1979, without a timezone conversion.
use super::Period;
use crate::{Error, ErrorKind, error::seconds_i32};

const MONTH_DAYS: [i32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

/// A date in the defined signed 32-bit seconds range of OCCT `Quantity_Date`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct Date {
    seconds: i32,
    micros: i32,
}

#[allow(clippy::too_many_arguments)] // Keep the upstream eight-component public contract.
impl Date {
    /// Constructs a date from month, day, year, hour, minute, second, millisecond and microsecond.
    pub fn new(
        mm: i32,
        dd: i32,
        yy: i32,
        hh: i32,
        mn: i32,
        ss: i32,
        mis: i32,
        mics: i32,
    ) -> Result<Self, Error> {
        if !Self::is_valid(mm, dd, yy, hh, mn, ss, mis, mics) {
            return Err(Error::new(
                ErrorKind::DateDefinition,
                "Quantity_Date::Quantity_Date invalid parameters",
            ));
        }
        // Count Gregorian leap days in O(1), then match OCCT's month accumulation.
        let year = i64::from(yy) - 1;
        let leaps = year / 4 - year / 100 + year / 400;
        let epoch_leaps = 1978 / 4 - 1978 / 100 + 1978 / 400;
        let mut days = (i64::from(yy) - 1979) * 365 + leaps - epoch_leaps;
        for month in 1..mm {
            days += i64::from(Self::days_in_month(month, yy));
        }
        days += i64::from(dd - 1);
        Ok(Self {
            seconds: seconds_i32(
                days * 86400 + i64::from(hh) * 3600 + i64::from(mn) * 60 + i64::from(ss),
            )?,
            micros: mis * 1000 + mics,
        })
    }

    /// Returns month, day, year, hour, minute, second, millisecond and microsecond.
    pub fn values(self) -> (i32, i32, i32, i32, i32, i32, i32, i32) {
        let mut days = self.seconds / 86400;
        let mut year = 1979;
        loop {
            let count = if Self::is_leap(year) { 366 } else { 365 };
            if days < count {
                break;
            }
            days -= count;
            year += 1;
        }
        let mut month = 1;
        while days >= Self::days_in_month(month, year) {
            days -= Self::days_in_month(month, year);
            month += 1;
        }
        (
            month,
            days + 1,
            year,
            self.seconds % 86400 / 3600,
            self.seconds % 3600 / 60,
            self.seconds % 60,
            self.micros / 1000,
            self.micros % 1000,
        )
    }

    /// Gregorian leap-year predicate, including years earlier than the epoch.
    pub const fn is_leap(year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
    }

    const fn days_in_month(month: i32, year: i32) -> i32 {
        if month == 2 && Self::is_leap(year) {
            29
        } else {
            MONTH_DAYS[(month - 1) as usize]
        }
    }

    /// Checks OCCT's component validity; this does not check seconds-storage overflow.
    pub const fn is_valid(
        mm: i32,
        dd: i32,
        yy: i32,
        hh: i32,
        mn: i32,
        ss: i32,
        mis: i32,
        mics: i32,
    ) -> bool {
        mm >= 1
            && mm <= 12
            && yy >= 1979
            && dd >= 1
            && dd <= Self::days_in_month(mm, yy)
            && hh >= 0
            && hh <= 23
            && mn >= 0
            && mn <= 59
            && ss >= 0
            && ss <= 59
            && mis >= 0
            && mis <= 999
            && mics >= 0
            && mics <= 999
    }
}

#[allow(clippy::too_many_arguments)]
impl Date {
    /// Replaces the date; invalid input leaves the original date unchanged.
    pub fn set_values(
        &mut self,
        mm: i32,
        dd: i32,
        yy: i32,
        hh: i32,
        mn: i32,
        ss: i32,
        mis: i32,
        mics: i32,
    ) -> Result<(), Error> {
        *self = Self::new(mm, dd, yy, hh, mn, ss, mis, mics)?;
        Ok(())
    }

    /// Returns the absolute time interval between the dates.
    pub const fn difference(self, other: Self) -> Period {
        Period::from_total_micros((self.total_micros() - other.total_micros()).abs())
    }

    /// Adds an interval, failing if the seconds storage would overflow.
    #[allow(clippy::should_implement_trait)]
    pub fn add(self, period: Period) -> Result<Self, Error> {
        let (seconds, micros) = Period::new(self.seconds, self.micros)?
            .add(period)?
            .values();
        Ok(Self { seconds, micros })
    }

    /// Subtracts an interval; dates before January 1, 1979 are rejected.
    pub fn subtract(self, period: Period) -> Result<Self, Error> {
        let total = self.total_micros() - period.total_micros();
        if total < 0 {
            return Err(Error::new(
                ErrorKind::DateDefinition,
                "Quantity_Date::Subtract : The result date is anterior to Jan,1 1979",
            ));
        }
        let (seconds, micros) = Period::from_total_micros(total).values();
        Ok(Self { seconds, micros })
    }

    const fn total_micros(self) -> i64 {
        self.seconds as i64 * 1_000_000 + self.micros as i64
    }

    /// Tests equality in a const context.
    pub const fn is_equal(self, other: Self) -> bool {
        self.seconds == other.seconds && self.micros == other.micros
    }
    /// Tests whether this date is strictly earlier.
    pub const fn is_earlier(self, other: Self) -> bool {
        self.seconds < other.seconds
            || (self.seconds == other.seconds && self.micros < other.micros)
    }
    /// Tests whether this date is strictly later.
    pub const fn is_later(self, other: Self) -> bool {
        other.is_earlier(self)
    }
    /// Year component.
    pub fn year(self) -> i32 {
        self.values().2
    }
    /// Month component (1–12).
    pub fn month(self) -> i32 {
        self.values().0
    }
    /// Day component (1–31).
    pub fn day(self) -> i32 {
        self.values().1
    }
    /// Hour component (0–23).
    pub fn hour(self) -> i32 {
        self.values().3
    }
    /// Minute component (0–59).
    pub fn minute(self) -> i32 {
        self.values().4
    }
    /// Second component (0–59).
    pub fn second(self) -> i32 {
        self.values().5
    }
    /// Millisecond component (0–999).
    pub fn millisecond(self) -> i32 {
        self.values().6
    }
    /// Remaining microsecond component (0–999).
    pub fn microsecond(self) -> i32 {
        self.values().7
    }
}

impl std::ops::Add<Period> for Date {
    type Output = Result<Self, Error>;
    fn add(self, rhs: Period) -> Self::Output {
        self.add(rhs)
    }
}

impl std::ops::Sub<Period> for Date {
    type Output = Result<Self, Error>;
    fn sub(self, rhs: Period) -> Self::Output {
        self.subtract(rhs)
    }
}
