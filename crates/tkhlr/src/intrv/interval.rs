// Rust translation of ModelingAlgorithms/TKHLR/Intrv/Intrv_Interval.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Intrv_Interval: a parameter interval whose two bounds each carry a tolerance.
use rustycad_tkernel::standard::real::{epsilon, real_first, real_last};
use rustycad_tkg2d::geom2d::native::{std_max, std_min};

/// Intrv_Interval: the interval `[start, end]` where each bound is fuzzy by its own
/// single-precision tolerance. The comparisons keep the native mixed precision: a sum
/// of two tolerances is a `float` sum, compared with a `double` difference of bounds.
///
/// ```
/// use rustycad_tkhlr::intrv::Interval;
/// let mut interval = Interval::with_tolerances(0.0, 0.1, 1.0, 0.1);
/// interval.fuse_at_start(-0.05, 0.1);
/// assert!(interval.start() < 0.0);
/// assert!(!interval.is_probably_empty());
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Interval {
    pub(super) start: f64,
    pub(super) end: f64,
    pub(super) tol_start: f32,
    pub(super) tol_end: f32,
}

impl Default for Interval {
    fn default() -> Self {
        Self::new()
    }
}

/// The native `(float)Epsilon(value)`: the spacing of doubles at `value`, rounded to
/// single precision (0 at RealFirst and RealLast, which have no double beyond them).
fn float_epsilon(value: f64) -> f32 {
    epsilon(value) as f32
}

/// AreFused (file function of the .lxx): two fuzzy bounds meet when their tolerances
/// (summed in single precision) reach the distance of their centres.
pub(super) fn are_fused(c1: f64, t1: f32, c2: f64, t2: f32) -> bool {
    f64::from(t1 + t2) >= (c1 - c2).abs()
}

impl Interval {
    /// Intrv_Interval(): the whole real line, from RealFirst to RealLast, with the
    /// tolerances `(float)Epsilon(RealFirst())` and `(float)Epsilon(RealLast())`, both 0
    /// (no double lies beyond them); `Interval::new()`.
    pub fn new() -> Self {
        Self {
            start: real_first(),
            end: real_last(),
            tol_start: float_epsilon(real_first()),
            tol_end: float_epsilon(real_last()),
        }
    }

    /// Intrv_Interval(Start, End): each bound's tolerance is the spacing of doubles at
    /// it; `Interval::from_bounds(0.0, 1.0)`.
    pub fn from_bounds(start: f64, end: f64) -> Self {
        Self {
            start,
            end,
            tol_start: float_epsilon(start),
            tol_end: float_epsilon(end),
        }
    }

    /// Intrv_Interval(Start, TolStart, End, TolEnd): a tolerance below the spacing of
    /// doubles at its bound is raised to it; `Interval::with_tolerances(0.0, 0.1, 1.0, 0.1)`.
    pub fn with_tolerances(start: f64, tol_start: f32, end: f64, tol_end: f32) -> Self {
        let (eps_start, eps_end) = (float_epsilon(start), float_epsilon(end));
        let mut interval = Self {
            start,
            end,
            tol_start,
            tol_end,
        };
        if interval.tol_start < eps_start {
            interval.tol_start = eps_start;
        }
        if interval.tol_end < eps_end {
            interval.tol_end = eps_end;
        }
        interval
    }

    /// Start: the first bound; `interval.start()`.
    pub fn start(&self) -> f64 {
        self.start
    }

    /// End: the last bound; `interval.end()`.
    pub fn end(&self) -> f64 {
        self.end
    }

    /// TolStart: the tolerance of the first bound; `interval.tol_start()`.
    pub fn tol_start(&self) -> f32 {
        self.tol_start
    }

    /// TolEnd: the tolerance of the last bound; `interval.tol_end()`.
    pub fn tol_end(&self) -> f32 {
        self.tol_end
    }

    /// Bounds: `(start, tol_start, end, tol_end)`, the native output arguments in
    /// order; `let (s, ts, e, te) = interval.bounds();`.
    pub fn bounds(&self) -> (f64, f32, f64, f32) {
        (self.start, self.tol_start, self.end, self.tol_end)
    }

    /// SetStart: replaces the first bound and its tolerance, unchecked;
    /// `interval.set_start(0.5, 0.01)`.
    pub fn set_start(&mut self, start: f64, tol_start: f32) {
        self.start = start;
        self.tol_start = tol_start;
    }

    /// SetEnd: replaces the last bound and its tolerance, unchecked;
    /// `interval.set_end(0.5, 0.01)`.
    pub fn set_end(&mut self, end: f64, tol_end: f32) {
        self.end = end;
        self.tol_end = tol_end;
    }

    /// FuseAtStart: the first bound becomes the union of the two fuzzy starts (the
    /// lower of their low ends and of their high ends); an infinite start (RealFirst)
    /// is kept; `interval.fuse_at_start(-0.05, 0.1)`.
    pub fn fuse_at_start(&mut self, start: f64, tol_start: f32) {
        if self.start == real_first() {
            return;
        }
        let low = std_min(
            self.start - f64::from(self.tol_start),
            start - f64::from(tol_start),
        );
        let high = std_min(
            self.start + f64::from(self.tol_start),
            start + f64::from(tol_start),
        );
        (self.start, self.tol_start) = centre_and_radius(low, high);
    }

    /// CutAtStart: the first bound becomes the higher of the two fuzzy starts; an
    /// infinite start (RealFirst) is kept; `interval.cut_at_start(0.05, 0.1)`.
    pub fn cut_at_start(&mut self, start: f64, tol_start: f32) {
        if self.start == real_first() {
            return;
        }
        let low = std_max(
            self.start - f64::from(self.tol_start),
            start - f64::from(tol_start),
        );
        let high = std_max(
            self.start + f64::from(self.tol_start),
            start + f64::from(tol_start),
        );
        (self.start, self.tol_start) = centre_and_radius(low, high);
    }

    /// FuseAtEnd: the last bound becomes the union of the two fuzzy ends (the higher
    /// of their ends); an infinite end (RealLast) is kept; `interval.fuse_at_end(1.05, 0.1)`.
    pub fn fuse_at_end(&mut self, end: f64, tol_end: f32) {
        if self.end == real_last() {
            return;
        }
        let low = std_max(self.end - f64::from(self.tol_end), end - f64::from(tol_end));
        let high = std_max(self.end + f64::from(self.tol_end), end + f64::from(tol_end));
        (self.end, self.tol_end) = centre_and_radius(low, high);
    }

    /// CutAtEnd: the last bound becomes the lower of the two fuzzy ends; an infinite
    /// end (RealLast) is kept; `interval.cut_at_end(0.95, 0.1)`.
    pub fn cut_at_end(&mut self, end: f64, tol_end: f32) {
        if self.end == real_last() {
            return;
        }
        let low = std_min(self.end - f64::from(self.tol_end), end - f64::from(tol_end));
        let high = std_min(self.end + f64::from(self.tol_end), end + f64::from(tol_end));
        (self.end, self.tol_end) = centre_and_radius(low, high);
    }

    /// IsProbablyEmpty: the fuzzy start and end meet; `interval.is_probably_empty()`.
    pub fn is_probably_empty(&self) -> bool {
        are_fused(self.start, self.tol_start, self.end, self.tol_end)
    }
}

/// The fuzzy bound spanning `[low, high]`: its centre and, as natively,
/// `(float)(high - low) / 2` (rounded to single precision before the halving).
fn centre_and_radius(low: f64, high: f64) -> (f64, f32) {
    ((low + high) / 2.0, ((high - low) as f32) / 2.0)
}
