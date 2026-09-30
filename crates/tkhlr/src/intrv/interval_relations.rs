// Rust translation of ModelingAlgorithms/TKHLR/Intrv/Intrv_Interval.lxx/.cxx (Position and the Is* tests) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The relative positions of two fuzzy intervals.
//!
//! Every test reads as natively: a sum of two single-precision tolerances (a `float`
//! sum) against a double difference of bounds, or two fuzzy bounds fused. The tests
//! are independent: `Position` decides with its own double-precision bound envelopes,
//! so where the rounding of those envelopes and of the float sums differ (at ties),
//! `position` and the matching `is_*` test can disagree, as natively.
use super::interval::{Interval, are_fused};
use super::position::Position;

/// The low and high ends of each fuzzy bound of an interval, in double precision.
struct Envelope {
    start_min: f64,
    start_max: f64,
    end_min: f64,
    end_max: f64,
}

impl Envelope {
    fn of(interval: &Interval) -> Self {
        let (start, end) = (interval.start, interval.end);
        let (tol_start, tol_end) = (f64::from(interval.tol_start), f64::from(interval.tol_end));
        Self {
            start_min: start - tol_start,
            start_max: start + tol_start,
            end_min: end - tol_end,
            end_max: end + tol_end,
        }
    }
}

impl Interval {
    /// Position: where this interval lies relative to `other`, decided on the fuzzy
    /// bound envelopes (see [`Position`]);
    /// `Interval::from_bounds(0.0, 1.0).position(&Interval::from_bounds(2.0, 3.0))`.
    pub fn position(&self, other: &Interval) -> Position {
        let (me, ot) = (Envelope::of(self), Envelope::of(other));
        if me.start_max < ot.start_min {
            return position_starting_before(&me, &ot);
        }
        if ot.start_max >= me.start_min {
            return position_starting_together(&me, &ot);
        }
        if me.start_max < ot.end_min {
            return position_starting_inside(&me, &ot);
        }
        if ot.end_max >= me.start_min {
            Position::JustAfter
        } else {
            Position::After
        }
    }

    /// IsBefore: this interval ends before `other` starts; `a.is_before(&b)`.
    pub fn is_before(&self, other: &Interval) -> bool {
        tolerances_below(self.tol_end, other.tol_start, other.start - self.end)
    }

    /// IsAfter: this interval starts after `other` ends; `a.is_after(&b)`.
    pub fn is_after(&self, other: &Interval) -> bool {
        tolerances_below(self.tol_start, other.tol_end, self.start - other.end)
    }

    /// IsInside: both bounds lie strictly inside `other`; `a.is_inside(&b)`.
    pub fn is_inside(&self, other: &Interval) -> bool {
        tolerances_below(self.tol_start, other.tol_start, self.start - other.start)
            && tolerances_below(self.tol_end, other.tol_end, other.end - self.end)
    }

    /// IsEnclosing: `other` lies strictly inside this interval; `a.is_enclosing(&b)`.
    pub fn is_enclosing(&self, other: &Interval) -> bool {
        tolerances_below(self.tol_start, other.tol_start, other.start - self.start)
            && tolerances_below(self.tol_end, other.tol_end, self.end - other.end)
    }

    /// IsJustEnclosingAtStart: the starts are fused and this interval ends after
    /// `other`; `a.is_just_enclosing_at_start(&b)`.
    pub fn is_just_enclosing_at_start(&self, other: &Interval) -> bool {
        self.starts_fused(other)
            && tolerances_below(self.tol_end, other.tol_end, self.end - other.end)
    }

    /// IsJustEnclosingAtEnd: this interval starts before `other` and the ends are
    /// fused; `a.is_just_enclosing_at_end(&b)`.
    pub fn is_just_enclosing_at_end(&self, other: &Interval) -> bool {
        tolerances_below(self.tol_start, other.tol_start, other.start - self.start)
            && self.ends_fused(other)
    }

    /// IsJustBefore: this interval's end is fused with the start of `other`;
    /// `a.is_just_before(&b)`.
    pub fn is_just_before(&self, other: &Interval) -> bool {
        are_fused(self.end, self.tol_end, other.start, other.tol_start)
    }

    /// IsJustAfter: this interval's start is fused with the end of `other`;
    /// `a.is_just_after(&b)`.
    pub fn is_just_after(&self, other: &Interval) -> bool {
        are_fused(other.end, other.tol_end, self.start, self.tol_start)
    }

    /// IsOverlappingAtStart: this interval starts before `other` and ends inside it;
    /// `a.is_overlapping_at_start(&b)`.
    pub fn is_overlapping_at_start(&self, other: &Interval) -> bool {
        tolerances_below(self.tol_start, other.tol_start, other.start - self.start)
            && tolerances_below(self.tol_end, other.tol_start, self.end - other.start)
            && tolerances_below(self.tol_end, other.tol_end, other.end - self.end)
    }

    /// IsOverlappingAtEnd: this interval starts inside `other` and ends after it;
    /// `a.is_overlapping_at_end(&b)`.
    pub fn is_overlapping_at_end(&self, other: &Interval) -> bool {
        tolerances_below(self.tol_start, other.tol_start, self.start - other.start)
            && tolerances_below(self.tol_start, other.tol_end, other.end - self.start)
            && tolerances_below(self.tol_end, other.tol_end, self.end - other.end)
    }

    /// IsJustOverlappingAtStart: the starts are fused and this interval ends inside
    /// `other`; `a.is_just_overlapping_at_start(&b)`.
    pub fn is_just_overlapping_at_start(&self, other: &Interval) -> bool {
        self.starts_fused(other)
            && tolerances_below(self.tol_end, other.tol_end, other.end - self.end)
    }

    /// IsJustOverlappingAtEnd: this interval starts inside `other` and the ends are
    /// fused; `a.is_just_overlapping_at_end(&b)`.
    pub fn is_just_overlapping_at_end(&self, other: &Interval) -> bool {
        tolerances_below(self.tol_start, other.tol_start, self.start - other.start)
            && self.ends_fused(other)
    }

    /// IsSimilar: both bounds are fused with those of `other`; `a.is_similar(&b)`.
    pub fn is_similar(&self, other: &Interval) -> bool {
        let starts = self.starts_fused(other);
        let ends = self.ends_fused(other);
        starts && ends
    }

    fn starts_fused(&self, other: &Interval) -> bool {
        are_fused(self.start, self.tol_start, other.start, other.tol_start)
    }

    /// The native argument order of the end tests: `AreFused(Other.myEnd, ..., myEnd, ...)`.
    fn ends_fused(&self, other: &Interval) -> bool {
        are_fused(other.end, other.tol_end, self.end, self.tol_end)
    }
}

/// `t1 + t2 < gap`: the single-precision sum of two tolerances is below a gap.
fn tolerances_below(t1: f32, t2: f32, gap: f64) -> bool {
    f64::from(t1 + t2) < gap
}

/// The `mySMax < otSMin` branch of Position: this interval starts before `other`.
fn position_starting_before(me: &Envelope, ot: &Envelope) -> Position {
    if me.end_max < ot.start_min {
        Position::Before
    } else if ot.start_max >= me.end_min {
        Position::JustBefore
    } else if me.end_max < ot.end_min {
        Position::OverlappingAtStart
    } else if ot.end_max >= me.end_min {
        Position::JustEnclosingAtEnd
    } else {
        Position::Enclosing
    }
}

/// The `otSMax >= mySMin` branch of Position: the starts meet.
fn position_starting_together(me: &Envelope, ot: &Envelope) -> Position {
    if me.end_max < ot.end_min {
        Position::JustOverlappingAtStart
    } else if ot.end_max >= me.end_min {
        Position::Similar
    } else {
        Position::JustEnclosingAtStart
    }
}

/// The `mySMax < otEMin` branch of Position: this interval starts inside `other`.
fn position_starting_inside(me: &Envelope, ot: &Envelope) -> Position {
    if me.end_max < ot.end_min {
        Position::Inside
    } else if ot.end_max >= me.end_min {
        Position::JustOverlappingAtEnd
    } else {
        Position::OverlappingAtEnd
    }
}
