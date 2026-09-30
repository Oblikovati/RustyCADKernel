// Rust translation of Extrema_GGenExtPC.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Extrema_GGenExtPC`: every root of [`GFuncExtPC`] on a parameter range, found by
//! math_FunctionRoots on a sampling.
use super::curve_reference::CurveReference;
use super::func_ext_pc::GFuncExtPC;
use crate::extrema::curve_tool::ExtremaCurveTool;
use crate::extrema::failures::{extremum_index, not_done};
use crate::extrema::pon::POnCurve;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::FunctionRoots;

/// `Extrema_GGenExtPC` over one [`ExtremaCurveTool`]; its write-only `myInit` flag
/// has no counterpart (the aliases
/// `Extrema_EPCOfExtPC`, `Extrema_EPCOfExtPC2d`, `Extrema_EPCOfELPCOfLocateExtPC` and
/// `Extrema_EPCOfELPCOfLocateExtPC2d` are this type);
/// `GGenExtPC::<CurveTool>::with_range(point, &adaptor, 20, 0.0, 1.0, 1e-9, 1e-9)?`.
pub struct GGenExtPC<'a, T: ExtremaCurveTool> {
    done: bool,
    nb_sample: i32,
    umin: f64,
    usup: f64,
    tol_u: f64,
    tol_f: f64,
    function: GFuncExtPC<'a, T>,
}

impl<T: ExtremaCurveTool> Default for GGenExtPC<'_, T> {
    fn default() -> Self {
        Self {
            done: false,
            nb_sample: 0,
            umin: 0.0,
            usup: 0.0,
            tol_u: 0.0,
            tol_f: 0.0,
            function: GFuncExtPC::new(),
        }
    }
}

impl<'a, T: ExtremaCurveTool> GGenExtPC<'a, T> {
    /// `Extrema_GGenExtPC()`; `GGenExtPC::<CurveTool>::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Extrema_GGenExtPC(P, C, NbSample, TolU, TolF)`: the whole curve range;
    /// `GGenExtPC::<CurveTool>::with_curve(point, &adaptor, 20, 1e-9, 1e-9)?`.
    pub fn with_curve(
        point: T::Point,
        curve: &'a T::Curve,
        nb_sample: i32,
        tol_u: f64,
        tol_f: f64,
    ) -> Result<Self, Failure> {
        let mut extrema: Self = Self {
            function: GFuncExtPC::with_point_curve(point, curve)?,
            ..Self::default()
        };
        extrema.initialize(curve, nb_sample, tol_u, tol_f)?;
        extrema.perform(point)?;
        Ok(extrema)
    }
    /// `Extrema_GGenExtPC(P, C, NbSample, Umin, Usup, TolU, TolF)`;
    /// `GGenExtPC::<CurveTool>::with_range(point, &adaptor, 20, 0.0, 1.0, 1e-9, 1e-9)?`.
    pub fn with_range(
        point: T::Point,
        curve: &'a T::Curve,
        nb_sample: i32,
        (umin, usup): (f64, f64),
        tol_u: f64,
        tol_f: f64,
    ) -> Result<Self, Failure> {
        let mut extrema: Self = Self {
            function: GFuncExtPC::with_point_curve(point, curve)?,
            ..Self::default()
        };
        extrema.initialize_range(curve, nb_sample, (umin, usup), tol_u, tol_f)?;
        extrema.perform(point)?;
        Ok(extrema)
    }
    /// `Initialize(C, NbU, TolU, TolF)`: the whole curve range;
    /// `extrema.initialize(&adaptor, 20, 1e-9, 1e-9)?`.
    pub fn initialize(
        &mut self,
        curve: &'a T::Curve,
        nb_u: i32,
        tol_u: f64,
        tol_f: f64,
    ) -> Result<(), Failure> {
        self.set_sampling(nb_u, tol_u, tol_f);
        self.function.initialize(curve)?;
        self.umin = T::first_parameter(curve)?;
        self.usup = T::last_parameter(curve)?;
        Ok(())
    }
    /// `Initialize(C, NbU, Umin, Usup, TolU, TolF)`;
    /// `extrema.initialize_range(&adaptor, 20, (0.0, 1.0), 1e-9, 1e-9)?`.
    pub fn initialize_range(
        &mut self,
        curve: &'a T::Curve,
        nb_u: i32,
        (umin, usup): (f64, f64),
        tol_u: f64,
        tol_f: f64,
    ) -> Result<(), Failure> {
        self.set_sampling(nb_u, tol_u, tol_f);
        self.function.initialize(curve)?;
        self.umin = umin;
        self.usup = usup;
        Ok(())
    }
    /// `Initialize(NbU, Umin, Usup, TolU, TolF)`: the range and tolerances only;
    /// `extrema.initialize_sampling(20, (0.0, 1.0), 1e-9, 1e-9)`.
    pub fn initialize_sampling(
        &mut self,
        nb_u: i32,
        (umin, usup): (f64, f64),
        tol_u: f64,
        tol_f: f64,
    ) {
        self.set_sampling(nb_u, tol_u, tol_f);
        self.umin = umin;
        self.usup = usup;
    }
    /// `Initialize(C)`: the curve of the function only; `extrema.initialize_curve(&adaptor)?`.
    pub fn initialize_curve(&mut self, curve: &'a T::Curve) -> Result<(), Failure> {
        self.function.initialize(curve)
    }
    pub(super) fn initialize_reference(
        &mut self,
        curve: CurveReference<'a, T::Curve>,
    ) -> Result<(), Failure> {
        self.function.initialize_reference(curve)
    }
    fn set_sampling(&mut self, nb_u: i32, tol_u: f64, tol_f: f64) {
        self.nb_sample = nb_u;
        self.tol_u = tol_u;
        self.tol_f = tol_f;
    }
    /// `Perform(P)`: math_FunctionRoots on `[Umin, Usup]` with `NbSample` samples; not
    /// done when it fails or finds F null everywhere; `extrema.perform(point)?`.
    pub fn perform(&mut self, point: T::Point) -> Result<(), Failure> {
        self.function.set_point(point);
        self.function.sub_interval_initialize(self.umin, self.usup);
        self.done = false;
        let eps: [f64; 3] = [self.tol_u, self.tol_f, self.tol_f];
        let (umin, usup, nb) = (self.umin, self.usup, self.nb_sample);
        let roots = FunctionRoots::with_options(&mut self.function, umin, usup, nb, eps, 0.0);
        self.function.take_failure()?;
        if !roots.is_done() || roots.is_all_null()? {
            return Ok(());
        }
        self.done = true;
        Ok(())
    }
    /// `IsDone()`; `extrema.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// `NbExt()`: StdFail_NotDone when not done; `extrema.nb_ext()?`.
    pub fn nb_ext(&self) -> Result<i32, Failure> {
        if !self.done {
            return Err(not_done("Extrema_GGenExtPC::NbExt()"));
        }
        Ok(self.function.nb_ext())
    }
    /// `SquareDistance(N)`; `extrema.square_distance(1)?`.
    pub fn square_distance(&self, n: i32) -> Result<f64, Failure> {
        extremum_index("Extrema_GGenExtPC::SquareDistance()", n, || self.nb_ext())?;
        self.function.square_distance(n)
    }
    /// `IsMin(N)`; `extrema.is_min(1)?`.
    pub fn is_min(&self, n: i32) -> Result<bool, Failure> {
        extremum_index("Extrema_GGenExtPC::IsMin()", n, || self.nb_ext())?;
        self.function.is_min(n)
    }
    /// `Point(N)`; `extrema.point(1)?`.
    pub fn point(&self, n: i32) -> Result<POnCurve<T::Point>, Failure> {
        extremum_index("Extrema_GGenExtPC::Point()", n, || self.nb_ext())?;
        self.function.point(n)
    }
}
