// Rust translation of Extrema_GenLocateExtPC.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Extrema_GenLocateExtPC`: the root of [`GFuncExtPC`] nearest to a start parameter,
//! found by math_FunctionRoot.
use super::func_ext_pc::GFuncExtPC;
use crate::extrema::curve_tool::ExtremaCurveTool;
use crate::extrema::failures::not_done;
use crate::extrema::pon::POnCurve;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::FunctionRoot;

/// `Extrema_GenLocateExtPC` over one [`ExtremaCurveTool`] (the aliases
/// `Extrema_LocEPCOfLocateExtPC` and `Extrema_LocEPCOfLocateExtPC2d`);
/// `GenLocateExtPC::<CurveTool>::with_range(point, &adaptor, 0.5, (0.0, 1.0), 1e-9)?`.
pub struct GenLocateExtPC<'a, T: ExtremaCurveTool> {
    done: bool,
    tol_u: f64,
    umin: f64,
    usup: f64,
    function: GFuncExtPC<'a, T>,
}

impl<T: ExtremaCurveTool> Default for GenLocateExtPC<'_, T> {
    fn default() -> Self {
        Self {
            done: false,
            tol_u: 0.0,
            umin: 0.0,
            usup: 0.0,
            function: GFuncExtPC::new(),
        }
    }
}

impl<'a, T: ExtremaCurveTool> GenLocateExtPC<'a, T> {
    /// `Extrema_GenLocateExtPC()`; `GenLocateExtPC::<CurveTool>::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Extrema_GenLocateExtPC(P, C, U0, TolU)`: the whole curve range;
    /// `GenLocateExtPC::<CurveTool>::with_curve(point, &adaptor, 0.5, 1e-9)?`.
    pub fn with_curve(
        point: T::Point,
        curve: &'a T::Curve,
        u0: f64,
        tol_u: f64,
    ) -> Result<Self, Failure> {
        let range: (f64, f64) = (T::first_parameter(curve)?, T::last_parameter(curve)?);
        Self::with_range(point, curve, u0, range, tol_u)
    }
    /// `Extrema_GenLocateExtPC(P, C, U0, Umin, Usup, TolU)`;
    /// `GenLocateExtPC::<CurveTool>::with_range(point, &adaptor, 0.5, (0.0, 1.0), 1e-9)?`.
    pub fn with_range(
        point: T::Point,
        curve: &'a T::Curve,
        u0: f64,
        range: (f64, f64),
        tol_u: f64,
    ) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        extrema.initialize(curve, range, tol_u)?;
        extrema.perform(point, u0)?;
        Ok(extrema)
    }
    /// `Initialize(C, Umin, Usup, TolU)`; `extrema.initialize(&adaptor, (0.0, 1.0), 1e-9)?`.
    pub fn initialize(
        &mut self,
        curve: &'a T::Curve,
        (umin, usup): (f64, f64),
        tol_u: f64,
    ) -> Result<(), Failure> {
        self.done = false;
        self.function.initialize(curve)?;
        self.umin = umin;
        self.usup = usup;
        self.tol_u = tol_u;
        Ok(())
    }
    /// `Perform(P, U0)`: math_FunctionRoot from `U0` in `[Umin, Usup]`; the root counts
    /// only when F at the saved point is below 1e-7 in absolute value;
    /// `extrema.perform(point, 0.5)?`.
    // The negated test keeps the native NaN semantics.
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    pub fn perform(&mut self, point: T::Point, u0: f64) -> Result<(), Failure> {
        self.function.set_point(point);
        let (tol, umin, usup) = (self.tol_u, self.umin, self.usup);
        let solver = FunctionRoot::bounded(&mut self.function, u0, tol, umin, usup, 100);
        self.function.take_failure()?;
        self.done = solver?.is_done();
        if !self.done {
            return Ok(());
        }
        let uu: f64 = self.point()?.parameter();
        let mut ff: f64 = 0.0;
        self.done = self.function.value(uu, &mut ff)? && !(ff.abs() >= 1.0e-07);
        Ok(())
    }
    /// `IsDone()`; `extrema.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// `SquareDistance()`: StdFail_NotDone when not done; `extrema.square_distance()?`.
    pub fn square_distance(&self) -> Result<f64, Failure> {
        self.check_done("SquareDistance")?;
        self.function.square_distance(1)
    }
    /// `IsMin()`; `extrema.is_min()?`.
    pub fn is_min(&self) -> Result<bool, Failure> {
        self.check_done("IsMin")?;
        self.function.is_min(1)
    }
    /// `Point()`; `extrema.point()?.parameter()`.
    pub fn point(&self) -> Result<POnCurve<T::Point>, Failure> {
        self.check_done("Point")?;
        self.function.point(1)
    }
    fn check_done(&self, member: &str) -> Result<(), Failure> {
        if self.done {
            return Ok(());
        }
        Err(not_done(&format!("Extrema_GenLocateExtPC::{member}()")))
    }
}
