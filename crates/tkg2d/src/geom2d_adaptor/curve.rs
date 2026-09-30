// Rust translation of ModelingData/TKG2d/Geom2dAdaptor/Geom2dAdaptor_Curve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dAdaptor_Curve`: a Geom2d curve seen through the Adaptor2d interface.
use super::curve_state::{self as state, AdaptorState};
use super::{curve_eval as eval, curve_query as query, curve_shape as shape};
use crate::adaptor2d::Adaptor2dCurve2d;
use crate::geom2d::offset_curve_utils::OffsetBasisCurve;
use crate::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCurve, Geom2dCurveD1, Geom2dCurveD2, Geom2dCurveD3,
};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{Circle2, Ellipse2, Hyperbola2, Line2, Parabola2, Point2, Vector2};

/// A Geom2d curve on `[UFirst, ULast]` for the algorithms that take an adaptor
/// (`Geom2dAdaptor_Curve`).
///
/// Loading records the curve kind and the elementary geometry; a trimmed curve is
/// replaced by its basis, and an offset curve gets a nested adaptor on its basis.
/// Bezier and B-spline curves are evaluated through a cached span polynomial
/// (`BSplCLib_Cache`), rebuilt under this object's lock;
/// `Geom2dAdaptorCurve::with_range(&curve, 0.0, 1.0)?`.
#[derive(Default)]
pub struct Geom2dAdaptorCurve {
    parent: StandardTransient,
    state: SharedState<AdaptorState>,
}
impl_transient!(
    Geom2dAdaptorCurve,
    "Geom2dAdaptor_Curve",
    dyn Adaptor2dCurve2d,
    parent,
    interfaces: [dyn Adaptor2dCurve2d]
);

impl Geom2dAdaptorCurve {
    /// An adaptor on `curve` over its own bounds (`Geom2dAdaptor_Curve(C)`);
    /// Standard_NullObject for a null curve; `Geom2dAdaptorCurve::with_curve(&curve)?`.
    pub fn with_curve(curve: &Handle<dyn Geom2dCurve>) -> Result<Self, Failure> {
        let adaptor = Self::default();
        adaptor.load(curve)?;
        Ok(adaptor)
    }
    /// An adaptor on `curve` over `[u_first, u_last]`
    /// (`Geom2dAdaptor_Curve(C, UFirst, ULast)`); the `Load` checks apply;
    /// `Geom2dAdaptorCurve::with_range(&curve, 0.0, 1.0)?`.
    pub fn with_range(
        curve: &Handle<dyn Geom2dCurve>,
        u_first: f64,
        u_last: f64,
    ) -> Result<Self, Failure> {
        let adaptor = Self::default();
        adaptor.load_range(curve, u_first, u_last)?;
        Ok(adaptor)
    }
    /// Unload the curve (`Reset`); `adaptor.reset()`.
    pub fn reset(&self) {
        state::reset(&mut self.state.write());
    }
    /// Load `curve` over its own bounds (`Load(C)`); Standard_NullObject for a null
    /// curve; `adaptor.load(&curve)?`.
    pub fn load(&self, curve: &Handle<dyn Geom2dCurve>) -> Result<(), Failure> {
        state::load_whole(&mut self.state.write(), curve)
    }
    /// Load `curve` over `[u_first, u_last]` (`Load(C, UFirst, ULast)`);
    /// Standard_NullObject for a null curve, Standard_ConstructionError when
    /// `u_first > u_last + Precision::Confusion()`; `adaptor.load_range(&curve, 0.0, 1.0)?`.
    pub fn load_range(
        &self,
        curve: &Handle<dyn Geom2dCurve>,
        u_first: f64,
        u_last: f64,
    ) -> Result<(), Failure> {
        state::load_range(&mut self.state.write(), curve, u_first, u_last)
    }
    /// Whether a curve is loaded (`IsInitialized`); `adaptor.is_initialized()`.
    pub fn is_initialized(&self) -> bool {
        self.state.read().curve.is_some()
    }
    /// The loaded curve, the basis of a loaded trimmed curve (`Curve`); `adaptor.curve()`.
    pub fn curve(&self) -> Handle<dyn Geom2dCurve> {
        self.state.read().curve.clone()
    }
    /// A copy sharing the curve, without the caches (`ShallowCopy`, as a value);
    /// `adaptor.shallow_adaptor()?`.
    pub fn shallow_adaptor(&self) -> Result<Self, Failure> {
        let copy: AdaptorState = state::shallow_state(&self.state.read())?;
        Ok(Self {
            parent: StandardTransient::default(),
            state: SharedState::new(copy),
        })
    }
    fn bounds(&self) -> (f64, f64) {
        let state = self.state.read();
        (state.first, state.last)
    }
}

impl Adaptor2dCurve2d for Geom2dAdaptorCurve {
    fn shallow_copy(&self) -> Result<Handle<dyn Adaptor2dCurve2d>, Failure> {
        Ok(Handle::new(self.shallow_adaptor()?).cast())
    }
    fn first_parameter(&self) -> Result<f64, Failure> {
        Ok(self.bounds().0)
    }
    fn last_parameter(&self) -> Result<f64, Failure> {
        Ok(self.bounds().1)
    }
    fn continuity(&self) -> Result<Shape, Failure> {
        shape::continuity(&self.state.read())
    }
    fn nb_intervals(&self, continuity: Shape) -> Result<i32, Failure> {
        shape::nb_intervals(&self.state.read(), continuity)
    }
    fn intervals(&self, bounds: &mut Array1<'_, f64>, continuity: Shape) -> Result<(), Failure> {
        shape::intervals(&self.state.read(), bounds, continuity)
    }
    fn trim(
        &self,
        first: f64,
        last: f64,
        _tolerance: f64,
    ) -> Result<Handle<dyn Adaptor2dCurve2d>, Failure> {
        Ok(Handle::new(Self::with_range(&self.curve(), first, last)?).cast())
    }
    fn is_closed(&self) -> Result<bool, Failure> {
        let (first, last) = self.bounds();
        if precision::is_positive_infinite(last) || precision::is_negative_infinite(first) {
            return Ok(false);
        }
        let start: Point2 = self.value(first)?;
        let end: Point2 = self.value(last)?;
        Ok(start.distance(end) <= precision::confusion())
    }
    fn is_periodic(&self) -> Result<bool, Failure> {
        Ok(self.state.read().curve()?.is_periodic())
    }
    fn period(&self) -> Result<f64, Failure> {
        // Unlike Geom2d_Curve::Period, no periodicity check: the curve's range.
        let state = self.state.read();
        let curve: &dyn Geom2dCurve = state.curve()?;
        Ok(curve.last_parameter() - curve.first_parameter())
    }
    fn resolution(&self, r_uv: f64) -> Result<f64, Failure> {
        query::resolution(&self.state.read(), r_uv)
    }
    fn get_type(&self) -> Result<CurveType, Failure> {
        Ok(self.state.read().type_curve)
    }
    fn line(&self) -> Result<Line2, Failure> {
        query::line(&self.state.read())
    }
    fn circle(&self) -> Result<Circle2, Failure> {
        query::circle(&self.state.read())
    }
    fn ellipse(&self) -> Result<Ellipse2, Failure> {
        query::ellipse(&self.state.read())
    }
    fn hyperbola(&self) -> Result<Hyperbola2, Failure> {
        query::hyperbola(&self.state.read())
    }
    fn parabola(&self) -> Result<Parabola2, Failure> {
        query::parabola(&self.state.read())
    }
    fn degree(&self) -> Result<i32, Failure> {
        query::degree(&self.state.read())
    }
    fn is_rational(&self) -> Result<bool, Failure> {
        Ok(query::is_rational(&self.state.read()))
    }
    fn nb_poles(&self) -> Result<i32, Failure> {
        query::nb_poles(&self.state.read())
    }
    fn nb_knots(&self) -> Result<i32, Failure> {
        query::nb_knots(&self.state.read())
    }
    fn nb_samples(&self) -> Result<i32, Failure> {
        query::nb_samples(self.state.read().curve()?)
    }
    fn bezier(&self) -> Result<Handle<Geom2dBezierCurve>, Failure> {
        Ok(self.state.read().curve.cast())
    }
    fn bspline(&self) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
        Ok(query::bspline(&self.state.read()))
    }
    fn eval_d0(&self, u: f64) -> Result<Point2, Failure> {
        eval::d0(&mut self.state.write(), u)
    }
    fn eval_d1(&self, u: f64) -> Result<Geom2dCurveD1, Failure> {
        eval::d1(&mut self.state.write(), u)
    }
    fn eval_d2(&self, u: f64) -> Result<Geom2dCurveD2, Failure> {
        eval::d2(&mut self.state.write(), u)
    }
    fn eval_d3(&self, u: f64) -> Result<Geom2dCurveD3, Failure> {
        eval::d3(&mut self.state.write(), u)
    }
    fn eval_dn(&self, u: f64, n: i32) -> Result<Vector2, Failure> {
        eval::dn(&self.state.read(), u, n)
    }
}

// The template parameter of Geom2d_OffsetCurveUtils::Evaluate*: the adaptor bounds
// and evaluators. Bounds cannot fail; an evaluation failure propagates.
impl OffsetBasisCurve for Geom2dAdaptorCurve {
    fn basis_first_parameter(&self) -> f64 {
        self.bounds().0
    }
    fn basis_last_parameter(&self) -> f64 {
        self.bounds().1
    }
    fn basis_d0(&self, u: f64) -> Result<Point2, Failure> {
        self.eval_d0(u)
    }
    fn basis_d1(&self, u: f64) -> Result<Geom2dCurveD1, Failure> {
        self.eval_d1(u)
    }
    fn basis_d2(&self, u: f64) -> Result<Geom2dCurveD2, Failure> {
        self.eval_d2(u)
    }
    fn basis_d3(&self, u: f64) -> Result<Geom2dCurveD3, Failure> {
        self.eval_d3(u)
    }
    fn basis_dn(&self, u: f64, n: i32) -> Result<Vector2, Failure> {
        self.eval_dn(u, n)
    }
}
