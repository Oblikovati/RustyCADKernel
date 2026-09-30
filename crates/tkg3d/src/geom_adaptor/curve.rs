// Rust translation of ModelingData/TKG3d/GeomAdaptor/GeomAdaptor_Curve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomAdaptor_Curve`: a Geom curve seen through the Adaptor3d interface.
use super::curve_state::{self as state, AdaptorState};
use super::{curve_eval as eval, curve_query as query, curve_shape as shape};
use crate::adaptor3d::Adaptor3dCurve;
use crate::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomCurve, GeomCurveD1, GeomCurveD2, GeomCurveD3,
    GeomOffsetCurve,
};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::SharedState;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{Circle3, Ellipse3, Hyperbola3, Line3, Parabola3, Point3, Vector3};

/// A Geom curve on `[UFirst, ULast]` for the algorithms that take an adaptor
/// (`GeomAdaptor_Curve`).
///
/// Loading records the curve kind and the elementary geometry; a trimmed curve is
/// replaced by its basis, and an offset curve gets a nested adaptor on its whole
/// basis. Bezier and B-spline curves are evaluated through a cached span polynomial
/// (`BSplCLib_Cache`), rebuilt under this object's lock;
/// `GeomAdaptorCurve::with_range(&curve, 0.0, 1.0)?`.
#[derive(Default)]
pub struct GeomAdaptorCurve {
    parent: StandardTransient,
    state: SharedState<AdaptorState>,
}
impl_transient!(
    GeomAdaptorCurve,
    "GeomAdaptor_Curve",
    dyn Adaptor3dCurve,
    parent,
    interfaces: [dyn Adaptor3dCurve]
);

impl GeomAdaptorCurve {
    /// An adaptor on `curve` over its own bounds (`GeomAdaptor_Curve(C)`);
    /// Standard_NullObject for a null curve; `GeomAdaptorCurve::with_curve(&curve)?`.
    pub fn with_curve(curve: &Handle<dyn GeomCurve>) -> Result<Self, Failure> {
        let adaptor = Self::default();
        adaptor.load(curve)?;
        Ok(adaptor)
    }
    /// An adaptor on `curve` over `[u_first, u_last]`
    /// (`GeomAdaptor_Curve(C, UFirst, ULast)`); the `Load` checks apply;
    /// `GeomAdaptorCurve::with_range(&curve, 0.0, 1.0)?`.
    pub fn with_range(
        curve: &Handle<dyn GeomCurve>,
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
    pub fn load(&self, curve: &Handle<dyn GeomCurve>) -> Result<(), Failure> {
        state::load_whole(&mut self.state.write(), curve)
    }
    /// Load `curve` over `[u_first, u_last]` (`Load(C, UFirst, ULast)`);
    /// Standard_NullObject for a null curve, Standard_ConstructionError when
    /// `u_first > u_last + Precision::Confusion()`; `adaptor.load_range(&curve, 0.0, 1.0)?`.
    pub fn load_range(
        &self,
        curve: &Handle<dyn GeomCurve>,
        u_first: f64,
        u_last: f64,
    ) -> Result<(), Failure> {
        state::load_range(&mut self.state.write(), curve, u_first, u_last)
    }
    /// The loaded curve, the basis of a loaded trimmed curve (`Curve`); `adaptor.curve()`.
    pub fn curve(&self) -> Handle<dyn GeomCurve> {
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

impl Adaptor3dCurve for GeomAdaptorCurve {
    fn shallow_copy(&self) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
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
    ) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
        Ok(Handle::new(Self::with_range(&self.curve(), first, last)?).cast())
    }
    fn is_closed(&self) -> Result<bool, Failure> {
        let (first, last) = self.bounds();
        if precision::is_positive_infinite(last) || precision::is_negative_infinite(first) {
            return Ok(false);
        }
        let start: Point3 = self.value(first)?;
        let end: Point3 = self.value(last)?;
        Ok(start.distance(&end) <= precision::confusion())
    }
    fn is_periodic(&self) -> Result<bool, Failure> {
        Ok(self.state.read().curve()?.is_periodic())
    }
    fn period(&self) -> Result<f64, Failure> {
        // Unlike Geom_Curve::Period, no periodicity check: the curve's range.
        let state = self.state.read();
        let curve: &dyn GeomCurve = state.curve()?;
        Ok(curve.last_parameter() - curve.first_parameter())
    }
    fn resolution(&self, r3d: f64) -> Result<f64, Failure> {
        query::resolution(&self.state.read(), r3d)
    }
    fn get_type(&self) -> Result<CurveType, Failure> {
        Ok(self.state.read().type_curve)
    }
    fn line(&self) -> Result<Line3, Failure> {
        query::line(&self.state.read())
    }
    fn circle(&self) -> Result<Circle3, Failure> {
        query::circle(&self.state.read())
    }
    fn ellipse(&self) -> Result<Ellipse3, Failure> {
        query::ellipse(&self.state.read())
    }
    fn hyperbola(&self) -> Result<Hyperbola3, Failure> {
        query::hyperbola(&self.state.read())
    }
    fn parabola(&self) -> Result<Parabola3, Failure> {
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
    fn bezier(&self) -> Result<Handle<GeomBezierCurve>, Failure> {
        query::bezier(&self.state.read())
    }
    fn bspline(&self) -> Result<Handle<GeomBSplineCurve>, Failure> {
        query::bspline(&self.state.read())
    }
    fn offset_curve(&self) -> Result<Handle<GeomOffsetCurve>, Failure> {
        query::offset_curve(&self.state.read())
    }
    fn eval_d0(&self, u: f64) -> Result<Point3, Failure> {
        eval::d0(&mut self.state.write(), u)
    }
    fn eval_d1(&self, u: f64) -> Result<GeomCurveD1, Failure> {
        eval::d1(&mut self.state.write(), u)
    }
    fn eval_d2(&self, u: f64) -> Result<GeomCurveD2, Failure> {
        eval::d2(&mut self.state.write(), u)
    }
    fn eval_d3(&self, u: f64) -> Result<GeomCurveD3, Failure> {
        eval::d3(&mut self.state.write(), u)
    }
    fn eval_dn(&self, u: f64, n: i32) -> Result<Vector3, Failure> {
        eval::dn(&self.state.read(), u, n)
    }
}
