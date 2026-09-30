// Rust translation of ModelingData/TKGeomBase/Approx/Approx_ComputeCLine.gxx, Approx_FitAndDivide.hxx, Approx_FitAndDivide2d.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Approx_ComputeCLine`: the Bezier approximation of a continuous function
//! (`AppCont_Function`) by the continuous least squares, cutting the parameter
//! range in halves until each piece fits. `Approx_FitAndDivide` and
//! `Approx_FitAndDivide2d` are its instantiations over `AppCont_Function`.
mod perform;

use crate::app_cont::{Function, LeastSquare};
use crate::app_par_curves::{Constraint, MultiCurve};
use rustycad_tkernel::standard::Failure;

/// `MAXSEGM`: the default largest number of segments.
const MAX_SEGMENTS: i32 = 1000;

/// The constructor arguments of `Approx_ComputeCLine`, with the native defaults
/// (`degreemin = 3`, `degreemax = 8`, `Tolerance3d = Tolerance2d = 1.0e-5`,
/// `cutting = false`, both constraints `AppParCurves_TangencyPoint`).
///
/// ```
/// use rustycad_tkgeombase::approx::ComputeCLineOptions;
/// let options = ComputeCLineOptions { cutting: true, ..ComputeCLineOptions::default() };
/// assert_eq!((options.degree_min, options.degree_max), (3, 8));
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComputeCLineOptions {
    /// `degreemin`.
    pub degree_min: i32,
    /// `degreemax`.
    pub degree_max: i32,
    /// `Tolerance3d`.
    pub tolerance_3d: f64,
    /// `Tolerance2d`.
    pub tolerance_2d: f64,
    /// `cutting`: whether the range may be cut.
    pub cutting: bool,
    /// `FirstC`.
    pub first_constraint: Constraint,
    /// `LastC`.
    pub last_constraint: Constraint,
}

impl Default for ComputeCLineOptions {
    fn default() -> Self {
        Self {
            degree_min: 3,
            degree_max: 8,
            tolerance_3d: 1.0e-5,
            tolerance_2d: 1.0e-5,
            cutting: false,
            first_constraint: Constraint::TangencyPoint,
            last_constraint: Constraint::TangencyPoint,
        }
    }
}

/// `Approx_ComputeCLine` over `AppCont_Function`: each stored piece is a Bezier
/// MultiCurve with its parameter range and its 3D and 2D errors.
///
/// ```ignore
/// let fit = FitAndDivide::with_line(&function, ComputeCLineOptions::default())?;
/// for i in 1..=fit.nb_multi_curves() {
///     let curve = fit.value(i)?;
/// }
/// ```
#[derive(Clone)]
pub struct ComputeCLine {
    multi_curves: Vec<MultiCurve>,
    first_params: Vec<f64>,
    last_params: Vec<f64>,
    the_multi_curve: MultiCurve,
    alldone: bool,
    tolreached: bool,
    tolers3d: Vec<f64>,
    tolers2d: Vec<f64>,
    options: ComputeCLineOptions,
    currenttol3d: f64,
    currenttol2d: f64,
    max_segments: i32,
    inv_order: bool,
    hang_checking: bool,
}

/// `Approx_FitAndDivide`: [`ComputeCLine`] over `AppCont_Function`.
pub type FitAndDivide = ComputeCLine;
/// `Approx_FitAndDivide2d`: the same instantiation (the native classes are twins).
pub type FitAndDivide2d = ComputeCLine;

impl ComputeCLine {
    /// The constructor without a line: initializes the fields only;
    /// `ComputeCLine::new(ComputeCLineOptions::default())`.
    pub fn new(options: ComputeCLineOptions) -> Self {
        Self {
            multi_curves: Vec::new(),
            first_params: Vec::new(),
            last_params: Vec::new(),
            the_multi_curve: MultiCurve::new(),
            alldone: false,
            tolreached: false,
            tolers3d: Vec::new(),
            tolers2d: Vec::new(),
            options,
            currenttol3d: 0.0,
            currenttol2d: 0.0,
            max_segments: MAX_SEGMENTS,
            inv_order: true,
            hang_checking: true,
        }
    }

    /// The constructor with a line: initializes and performs;
    /// `ComputeCLine::with_line(&function, ComputeCLineOptions::default())?`.
    pub fn with_line(line: &dyn Function, options: ComputeCLineOptions) -> Result<Self, Failure> {
        let mut compute: Self = Self::new(options);
        compute.perform(line)?;
        Ok(compute)
    }

    /// `SetDegrees(degreemin, degreemax)`; `compute.set_degrees(2, 6)`.
    pub fn set_degrees(&mut self, degree_min: i32, degree_max: i32) {
        self.options.degree_min = degree_min;
        self.options.degree_max = degree_max;
    }

    /// `SetTolerances(Tolerance3d, Tolerance2d)`; `compute.set_tolerances(1e-4, 1e-6)`.
    pub fn set_tolerances(&mut self, tolerance_3d: f64, tolerance_2d: f64) {
        self.options.tolerance_3d = tolerance_3d;
        self.options.tolerance_2d = tolerance_2d;
    }

    /// `SetConstraints(FirstC, LastC)`;
    /// `compute.set_constraints(Constraint::PassPoint, Constraint::PassPoint)`.
    pub fn set_constraints(&mut self, first: Constraint, last: Constraint) {
        self.options.first_constraint = first;
        self.options.last_constraint = last;
    }

    /// `SetMaxSegments(theMaxSegments)`: the largest number of pieces when cutting;
    /// `compute.set_max_segments(10)`.
    pub fn set_max_segments(&mut self, max_segments: i32) {
        self.max_segments = max_segments;
    }

    /// `SetInvOrder(theInvOrder)`: try the degrees from the largest down (the
    /// default); `compute.set_inv_order(false)`.
    pub fn set_inv_order(&mut self, inv_order: bool) {
        self.inv_order = inv_order;
    }

    /// `SetHangChecking(theHangChecking)`: stop cutting when the cuts stop improving
    /// (the default); `compute.set_hang_checking(false)`.
    pub fn set_hang_checking(&mut self, hang_checking: bool) {
        self.hang_checking = hang_checking;
    }

    /// `IsAllApproximated()`; `compute.is_all_approximated()`.
    pub fn is_all_approximated(&self) -> bool {
        self.alldone
    }

    /// `IsToleranceReached()`; `compute.is_tolerance_reached()`.
    pub fn is_tolerance_reached(&self) -> bool {
        self.tolreached
    }

    /// `Error(Index, tol3d, tol2d)`: the errors of piece `index`;
    /// Standard_OutOfRange outside `1..=NbMultiCurves()`; `compute.error(1)?`.
    pub fn error(&self, index: i32) -> Result<(f64, f64), Failure> {
        Ok((
            sequence_value(&self.tolers3d, index)?,
            sequence_value(&self.tolers2d, index)?,
        ))
    }

    /// `NbMultiCurves()`; `compute.nb_multi_curves()`.
    pub fn nb_multi_curves(&self) -> i32 {
        self.multi_curves.len() as i32
    }

    /// `Value(Index)`: a copy (sharing the poles) of piece `index`;
    /// Standard_OutOfRange outside `1..=NbMultiCurves()`; `compute.value(1)?`.
    pub fn value(&self, index: i32) -> Result<MultiCurve, Failure> {
        sequence_value(&self.multi_curves, index)
    }

    /// `Parameters(Index, firstp, lastp)`: the parameter range of piece `index`;
    /// `compute.parameters(1)?`.
    pub fn parameters(&self, index: i32) -> Result<(f64, f64), Failure> {
        Ok((
            sequence_value(&self.first_params, index)?,
            sequence_value(&self.last_params, index)?,
        ))
    }

    // Stores a piece that fits (`myMultiCurves`, `myfirstparam`... appends).
    fn store(&mut self, curve: MultiCurve, [u_first, u_last]: [f64; 2], [tol3d, tol2d]: [f64; 2]) {
        self.multi_curves.push(curve);
        self.first_params.push(u_first);
        self.last_params.push(u_last);
        self.tolers3d.push(tol3d);
        self.tolers2d.push(tol2d);
    }
}

/// `NCollection_Sequence::Value(Index)`: Standard_OutOfRange outside `1..=Length()`.
pub(super) fn sequence_value<T: Clone>(sequence: &[T], index: i32) -> Result<T, Failure> {
    usize::try_from(index - 1)
        .ok()
        .and_then(|slot| sequence.get(slot))
        .cloned()
        .ok_or_else(|| {
            Failure::out_of_range(&format!(
                "NCollection_Sequence::Value: index {index}; expected 1..={}",
                sequence.len()
            ))
        })
}

/// One continuous least-squares fit of `line` over `[u_first, u_last]` at degree
/// `deg` with `min(2 deg + 1, 24)` Gauss points: `(IsDone, fit)`.
pub(super) fn fit_at_degree(
    line: &dyn Function,
    [u_first, u_last]: [f64; 2],
    options: &ComputeCLineOptions,
    deg: i32,
) -> Result<LeastSquare, Failure> {
    const NB_POINTS_MAX: i32 = 24;
    let nb_points: i32 = (2 * deg + 1).min(NB_POINTS_MAX);
    LeastSquare::new(
        line,
        u_first,
        u_last,
        options.first_constraint,
        options.last_constraint,
        deg,
        nb_points,
    )
}
