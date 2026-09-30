// Rust translation of Extrema_GLocateExtPC.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Extrema_GLocateExtPC`: the extremum of the distance between a point and a curve
//! nearest to a start parameter: a local search on the C2 interval of the start (then
//! on the neighbouring intervals) for B-spline, offset and other curves, a local
//! search on a Bezier curve, and the closed-form extremum nearest to the start for
//! the conics.
mod intervals;

use super::ext_pc::GGExtPC;
use super::gen_locate_ext_pc::GenLocateExtPC;
use crate::extrema::curve_tool::ExtremaCurveTool;
use crate::extrema::failures::null_object;
use crate::extrema::gp_space::{GpPoint, GpVector};
use crate::extrema::pon::POnCurve;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkmath::geom_abs::{CurveType, Shape};

/// `Extrema_GLocateExtPC` over one [`ExtremaCurveTool`] (`Extrema_LocateExtPC` and
/// `Extrema_LocateExtPC2d`); `LocateExtPC::with_curve(point, &adaptor, 0.5, 1e-9)?`.
pub struct GLocateExtPC<'a, T: ExtremaCurveTool> {
    pp: POnCurve<T::Point>,
    curve: Option<&'a T::Curve>,
    dist2: f64,
    is_min: bool,
    done: bool,
    umin: f64,
    usup: f64,
    tol: f64,
    loc_ext_pc: GenLocateExtPC<'a, T>,
    extrem_pc: GGExtPC<'a, T>,
    kind: CurveType,
    number_ext: i32,
}

impl<T: ExtremaCurveTool> Default for GLocateExtPC<'_, T> {
    fn default() -> Self {
        Self {
            pp: POnCurve::default(),
            curve: None,
            dist2: 0.0,
            is_min: false,
            done: false,
            umin: 0.0,
            usup: 0.0,
            tol: 0.0,
            loc_ext_pc: GenLocateExtPC::new(),
            extrem_pc: GGExtPC::new(),
            kind: CurveType::OtherCurve,
            number_ext: 0,
        }
    }
}

// Whether a curve kind is searched locally (`myLocExtPC`) rather than in closed form.
fn is_numeric_kind(kind: CurveType) -> bool {
    matches!(
        kind,
        CurveType::BSplineCurve
            | CurveType::BezierCurve
            | CurveType::OffsetCurve
            | CurveType::OtherCurve
    )
}

impl<'a, T: ExtremaCurveTool> GLocateExtPC<'a, T> {
    /// `Extrema_GLocateExtPC()`; `LocateExtPC::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Extrema_GLocateExtPC(P, C, U0, TolF)`: the whole curve range;
    /// `LocateExtPC::with_curve(point, &adaptor, 0.5, 1e-9)?`.
    pub fn with_curve(
        point: T::Point,
        curve: &'a T::Curve,
        u0: f64,
        tol_f: f64,
    ) -> Result<Self, Failure> {
        let range: (f64, f64) = (T::first_parameter(curve)?, T::last_parameter(curve)?);
        Self::with_range(point, curve, u0, range, tol_f)
    }
    /// `Extrema_GLocateExtPC(P, C, U0, Umin, Usup, TolF)`;
    /// `LocateExtPC::with_range(point, &adaptor, 0.5, (0.0, 1.0), 1e-9)?`.
    pub fn with_range(
        point: T::Point,
        curve: &'a T::Curve,
        u0: f64,
        range: (f64, f64),
        tol_f: f64,
    ) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        extrema.initialize(curve, range, tol_f)?;
        extrema.perform(point, u0)?;
        Ok(extrema)
    }
    /// `Initialize(C, Umin, Usup, TolF)`: the local or closed-form search gets the
    /// curve resolution of Precision::Confusion() as tolerance (as its TolF for the
    /// conics); `extrema.initialize(&adaptor, (0.0, 1.0), 1e-9)?`.
    pub fn initialize(
        &mut self,
        curve: &'a T::Curve,
        (umin, usup): (f64, f64),
        tol_f: f64,
    ) -> Result<(), Failure> {
        self.curve = Some(curve);
        self.tol = tol_f;
        self.umin = umin;
        self.usup = usup;
        self.kind = T::get_type(curve)?;
        let tol_u: f64 = T::resolution(curve, precision::confusion())?;
        if is_numeric_kind(self.kind) {
            return self.loc_ext_pc.initialize(curve, (umin, usup), tol_u);
        }
        self.extrem_pc.initialize(curve, umin, usup, tol_u)
    }
    /// `Perform(P, U0)`; `extrema.perform(point, 0.5)?`.
    pub fn perform(&mut self, point: T::Point, u0: f64) -> Result<(), Failure> {
        match self.kind {
            CurveType::OtherCurve | CurveType::OffsetCurve | CurveType::BSplineCurve => {
                self.perform_intervals(point, u0)
            }
            CurveType::BezierCurve => {
                self.loc_ext_pc.perform(point, u0)?;
                self.done = self.loc_ext_pc.is_done();
                Ok(())
            }
            _ => self.perform_closed_form(point, u0),
        }
    }
    // The closed-form extremum whose parameter is nearest to `u0` (the last of equals).
    fn perform_closed_form(&mut self, point: T::Point, u0: f64) -> Result<(), Failure> {
        self.extrem_pc.perform(point)?;
        self.number_ext = 0;
        if self.extrem_pc.is_done() {
            let mut best: f64 = f64::MAX;
            for i in 1..=self.extrem_pc.nb_ext()? {
                let val_u: f64 = (self.extrem_pc.point(i)?.parameter() - u0).abs();
                if val_u <= best {
                    best = val_u;
                    self.number_ext = i;
                    self.done = true;
                }
            }
        }
        if self.number_ext == 0 {
            self.done = false;
        }
        Ok(())
    }
    fn curve(&self) -> Result<&'a T::Curve, Failure> {
        self.curve
            .ok_or_else(|| null_object("Extrema_GLocateExtPC::Perform()"))
    }
    // The C2 bounds of the curve (`theInter`).
    fn c2_bounds(curve: &T::Curve) -> Result<Vec<f64>, Failure> {
        let n: i32 = T::nb_intervals(curve, Shape::C2)?;
        let mut bounds: Array1<'static, f64> =
            Array1::with_bounds(1, n + 1).map_err(|error| error.into_failure("Intervals"))?;
        T::intervals(curve, &mut bounds, Shape::C2)?;
        Ok((1..=n + 1).map(|i| bounds[i]).collect())
    }
    // The interval `index` (1-based) of `bounds`, clipped to `[myumin, myusup]`.
    fn clipped(&self, bounds: &[f64], index: i32) -> (f64, f64) {
        let i: usize = (index - 1) as usize;
        (
            std_max(bounds[i], self.umin),
            std_min(bounds[i + 1], self.usup),
        )
    }
    // `TheVector(P, C(u)) * C'(u)` with the point.
    fn projection_sign(&self, point: T::Point, u: f64) -> Result<(f64, T::Point), Failure> {
        let (p1, v1) = T::d1(self.curve()?, u)?;
        Ok((point.vector_to(&p1).dot_with(&v1), p1))
    }
    // Keeps the local search result.
    fn take_local(&mut self) -> Result<(), Failure> {
        self.pp = self.loc_ext_pc.point()?;
        self.is_min = self.loc_ext_pc.is_min()?;
        self.dist2 = self.loc_ext_pc.square_distance()?;
        Ok(())
    }
}
