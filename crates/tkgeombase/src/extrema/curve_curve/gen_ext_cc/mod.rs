// Rust translation of Extrema_GGenExtCC.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Extrema_GGenExtCC` (`Extrema_ECC`, `Extrema_ECC2d`): the minima of the distance
//! between two curves by math_GlobOptMin (Evtushenko's method) on the products of
//! their C2 (or C1) intervals, with a Lipschitz constant estimated from the curves.
mod intervals;
mod lipschitz;
mod search;
mod solutions;

use crate::extrema::curve_tool::ExtremaCurveTool;
use crate::extrema::failures::{extremum_index, not_done, null_object, sequence_value};
use crate::extrema::gp_space::GpPoint;
use crate::extrema::pon::POnCurve;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;

/// `Extrema_GGenExtCC` over one [`ExtremaCurveTool`] (both curves of one tool, as in
/// every native instantiation). `Perform` appends its solutions without clearing
/// those of an earlier call, as natively;
/// `ECC::with_curves(&c1, &c2)?.perform()?`.
pub struct GGenExtCC<'a, T: ExtremaCurveTool> {
    is_find_single_solution: bool,
    parallel: bool,
    curve_min_tol: f64,
    low_border: [f64; 2],
    upp_border: [f64; 2],
    points1: Vec<f64>,
    points2: Vec<f64>,
    curves: [Option<&'a T::Curve>; 2],
    done: bool,
}

impl<T: ExtremaCurveTool> Default for GGenExtCC<'_, T> {
    fn default() -> Self {
        Self {
            is_find_single_solution: false,
            parallel: false,
            curve_min_tol: precision::p_confusion(),
            low_border: [0.0; 2],
            upp_border: [0.0; 2],
            points1: Vec::new(),
            points2: Vec::new(),
            curves: [None, None],
            done: false,
        }
    }
}

impl<'a, T: ExtremaCurveTool> GGenExtCC<'a, T> {
    /// `Extrema_GGenExtCC()`; `ECC::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Extrema_GGenExtCC(C1, C2)`: the bounds are the curve ranges;
    /// `ECC::with_curves(&c1, &c2)?`.
    pub fn with_curves(c1: &'a T::Curve, c2: &'a T::Curve) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        extrema.curves = [Some(c1), Some(c2)];
        extrema.low_border = [T::first_parameter(c1)?, T::first_parameter(c2)?];
        extrema.upp_border = [T::last_parameter(c1)?, T::last_parameter(c2)?];
        Ok(extrema)
    }
    /// `Extrema_GGenExtCC(C1, C2, Uinf, Usup, Vinf, Vsup)`;
    /// `ECC::with_bounds(&c1, &c2, (0.0, 1.0), (0.0, 1.0))`.
    pub fn with_bounds(c1: &'a T::Curve, c2: &'a T::Curve, u: (f64, f64), v: (f64, f64)) -> Self {
        let mut extrema: Self = Self::new();
        extrema.set_params(c1, c2, u, v);
        extrema
    }
    /// `SetParams(C1, C2, Uinf, Usup, Vinf, Vsup)`;
    /// `extrema.set_params(&c1, &c2, (0.0, 1.0), (0.0, 1.0))`.
    pub fn set_params(
        &mut self,
        c1: &'a T::Curve,
        c2: &'a T::Curve,
        (uinf, usup): (f64, f64),
        (vinf, vsup): (f64, f64),
    ) {
        self.curves = [Some(c1), Some(c2)];
        self.low_border = [uinf, vinf];
        self.upp_border = [usup, vsup];
    }
    /// `SetTolerance(theTol)`: the minimal parameter tolerance of the curves;
    /// `extrema.set_tolerance(1e-9)`.
    pub fn set_tolerance(&mut self, tol: f64) {
        self.curve_min_tol = tol;
    }
    /// `SetSingleSolutionFlag(theFlag)`; `extrema.set_single_solution_flag(true)`.
    pub fn set_single_solution_flag(&mut self, flag: bool) {
        self.is_find_single_solution = flag;
    }
    /// `GetSingleSolutionFlag()`; `extrema.get_single_solution_flag()`.
    pub fn get_single_solution_flag(&self) -> bool {
        self.is_find_single_solution
    }
    fn curve_pair(&self) -> Result<(&'a T::Curve, &'a T::Curve), Failure> {
        let missing = || null_object("Extrema_GGenExtCC::Perform()");
        Ok((
            self.curves[0].ok_or_else(missing)?,
            self.curves[1].ok_or_else(missing)?,
        ))
    }
    /// `Perform()`; `extrema.perform()?`.
    pub fn perform(&mut self) -> Result<(), Failure> {
        self.done = false;
        self.parallel = false;
        let (c1, c2) = self.curve_pair()?;
        let layout = intervals::interval_layout::<T>(c1, c2)?;
        let lipschitz = lipschitz::lipschitz_constant::<T>(c1, c2)?;
        let found = self.search_minima(c1, c2, &layout, lipschitz)?;
        if found.points.is_empty() {
            self.done = false;
            return Ok(());
        }
        self.done = true;
        self.keep_solutions(c1, c2, found)
    }
}

impl<T: ExtremaCurveTool> GGenExtCC<'_, T> {
    /// `IsDone()`; `extrema.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// `IsParallel()`: StdFail_NotDone when not done; `extrema.is_parallel()?`.
    pub fn is_parallel(&self) -> Result<bool, Failure> {
        if !self.done {
            return Err(not_done("Extrema_GGenExtCC::IsParallel()"));
        }
        Ok(self.parallel)
    }
    /// `NbExt()`: StdFail_NotDone when not done; `extrema.nb_ext()?`.
    pub fn nb_ext(&self) -> Result<i32, Failure> {
        if !self.done {
            return Err(not_done("Extrema_GGenExtCC::NbExt()"));
        }
        Ok(self.points1.len() as i32)
    }
    /// `SquareDistance(N)` (the native default is 1): recomputed from the curve points;
    /// `extrema.square_distance(1)?`.
    pub fn square_distance(&self, n: i32) -> Result<f64, Failure> {
        let (p1, p2) = self.points(n)?;
        Ok(p1.value().square_distance_to(p2.value()))
    }
    /// `Points(N, P1, P2)`, returned as `(P1, P2)`; `extrema.points(1)?`.
    // The native output pair (P1, P2) as one value.
    #[allow(clippy::type_complexity)]
    pub fn points(&self, n: i32) -> Result<(POnCurve<T::Point>, POnCurve<T::Point>), Failure> {
        let member: &str = "Extrema_GGenExtCC::Points()";
        extremum_index(member, n, || self.nb_ext())?;
        let (c1, c2) = self.curve_pair()?;
        let (u, v) = (
            sequence_value(member, &self.points1, n)?,
            sequence_value(member, &self.points2, n)?,
        );
        Ok((
            POnCurve::new(u, T::value(c1, u)?),
            POnCurve::new(v, T::value(c2, v)?),
        ))
    }
}
