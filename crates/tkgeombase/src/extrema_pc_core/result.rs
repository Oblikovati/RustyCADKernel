// Rust translation of the Status, SearchMode, ExtremumResult and Result declarations of
// ModelingData/TKGeomBase/ExtremaPC2d/ExtremaPC2d.hxx and ExtremaPC/ExtremaPC.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The result types of the point-curve extrema packages, generic over the point type:
//! `ExtremaPC2d::Result` is `PcResult<Point2>` and `ExtremaPC::Result` is
//! `PcResult<Point3>`. The two native declarations are identical apart from the point.

/// Status of an extrema computation (`ExtremaPC2d::Status`, `ExtremaPC::Status`).
///
/// Both native packages declare the same six values, so one Rust enum serves both;
/// `PcResult::<Point2>::default().status == Status::NotDone`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Status {
    /// Computation succeeded, finite number of extrema found.
    OK,
    /// Computation not performed.
    NotDone,
    /// Infinite solutions exist (e.g., point at circle center).
    InfiniteSolutions,
    /// No extrema found in the given parameter range.
    NoSolution,
    /// Invalid tolerance or parameter domain.
    InvalidInput,
    /// Numerical issues during computation.
    NumericalError,
}

/// Which extrema a computation looks for (`ExtremaPC2d::SearchMode`, `ExtremaPC::SearchMode`);
/// `SearchMode::default() == SearchMode::MinMax`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SearchMode {
    /// Find all local extrema (both minima and maxima) - the native default.
    #[default]
    MinMax,
    /// Find local minima only.
    Min,
    /// Find local maxima only.
    Max,
}

/// One extremum (`ExtremaPC2d::ExtremumResult`, `ExtremaPC::ExtremumResult`).
///
/// The native default is a minimum at parameter 0 with a default point;
/// `PcExtremum::<Point2>::default().is_minimum`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PcExtremum<P> {
    /// Parameter value on the curve (`Parameter`).
    pub parameter: f64,
    /// Point on the curve at the parameter (`Point`).
    pub point: P,
    /// Square of the distance from the query point to the curve point (`SquareDistance`).
    pub square_distance: f64,
    /// True for a local minimum (`IsMinimum`, default true).
    pub is_minimum: bool,
    /// True for a local maximum (`IsMaximum`, default false).
    pub is_maximum: bool,
}

impl<P: Default> Default for PcExtremum<P> {
    fn default() -> Self {
        Self {
            parameter: 0.0,
            point: P::default(),
            square_distance: 0.0,
            is_minimum: true,
            is_maximum: false,
        }
    }
}

impl<P> PcExtremum<P> {
    /// The aggregate initialization `{Parameter, Point, SquareDistance, IsMinimum, IsMaximum}`;
    /// `PcExtremum::new(0.0, Point2::default(), 9.0, true, false)`.
    pub const fn new(
        parameter: f64,
        point: P,
        square_distance: f64,
        is_minimum: bool,
        is_maximum: bool,
    ) -> Self {
        Self {
            parameter,
            point,
            square_distance,
            is_minimum,
            is_maximum,
        }
    }
}

/// All extrema of one computation (`ExtremaPC2d::Result`, `ExtremaPC::Result`).
///
/// The native struct is move-only; the Rust struct does not derive `Clone` for the
/// same reason (evaluators hand out references to their reusable result);
/// `let result = PcResult::<Point2>::default(); result.nb_ext() == 0`.
#[derive(Debug, PartialEq)]
pub struct PcResult<P> {
    /// Computation status (`Status`, default NotDone).
    pub status: Status,
    /// The extrema found, in the order the algorithm appended them (`Extrema`).
    pub extrema: Vec<PcExtremum<P>>,
    /// The constant square distance of infinite solutions (`InfiniteSquareDistance`);
    /// meaningful only when the status is InfiniteSolutions.
    pub infinite_square_distance: f64,
}

impl<P> Default for PcResult<P> {
    fn default() -> Self {
        Self::not_done()
    }
}

impl<P> PcResult<P> {
    /// Sentinel returned by the index queries when there is no extremum (`INVALID_INDEX`).
    pub const INVALID_INDEX: usize = usize::MAX;

    /// An empty NotDone result (`Result()`); `PcResult::<Point2>::not_done()`.
    pub const fn not_done() -> Self {
        Self {
            status: Status::NotDone,
            extrema: Vec::new(),
            infinite_square_distance: 0.0,
        }
    }
    /// True when a finite search completed, including one without matching extrema
    /// (`IsDone`); `result.is_done()`.
    pub fn is_done(&self) -> bool {
        self.status == Status::OK || self.status == Status::NoSolution
    }
    /// True when there are infinitely many solutions (`IsInfinite`); `result.is_infinite()`.
    pub fn is_infinite(&self) -> bool {
        self.status == Status::InfiniteSolutions
    }
    /// Number of extrema found (`NbExt`); `result.nb_ext()`.
    pub fn nb_ext(&self) -> usize {
        self.extrema.len()
    }
    /// The smallest square distance, infinity without extrema (`MinSquareDistance`);
    /// `result.min_square_distance()`.
    pub fn min_square_distance(&self) -> f64 {
        match self.min_index() {
            Self::INVALID_INDEX => f64::INFINITY,
            index => self.extrema[index].square_distance,
        }
    }
    /// The 0-based index of the first closest extremum, or INVALID_INDEX (`MinIndex`);
    /// `result.min_index()`.
    pub fn min_index(&self) -> usize {
        self.extreme_index(|candidate, best| candidate < best)
    }
    /// The largest square distance, 0 without extrema (`MaxSquareDistance`);
    /// `result.max_square_distance()`.
    pub fn max_square_distance(&self) -> f64 {
        match self.max_index() {
            Self::INVALID_INDEX => 0.0,
            index => self.extrema[index].square_distance,
        }
    }
    /// The 0-based index of the first farthest extremum, or INVALID_INDEX (`MaxIndex`);
    /// `result.max_index()`.
    pub fn max_index(&self) -> usize {
        self.extreme_index(|candidate, best| candidate > best)
    }
    /// Reset to an empty NotDone result, keeping the allocation (`Clear`); `result.clear()`.
    pub fn clear(&mut self) {
        self.status = Status::NotDone;
        self.extrema.clear();
        self.infinite_square_distance = 0.0;
    }

    // The native scans keep the first index on ties: the comparison is strict.
    fn extreme_index(&self, better: impl Fn(f64, f64) -> bool) -> usize {
        let Some(first) = self.extrema.first() else {
            return Self::INVALID_INDEX;
        };
        let (mut index, mut best) = (0, first.square_distance);
        for (candidate_index, candidate) in self.extrema.iter().enumerate().skip(1) {
            if better(candidate.square_distance, best) {
                (index, best) = (candidate_index, candidate.square_distance);
            }
        }
        index
    }
}

/// `operator[]`: the extremum at a 0-based index; `result[0].parameter`.
impl<P> std::ops::Index<usize> for PcResult<P> {
    type Output = PcExtremum<P>;
    fn index(&self, index: usize) -> &PcExtremum<P> {
        &self.extrema[index]
    }
}
