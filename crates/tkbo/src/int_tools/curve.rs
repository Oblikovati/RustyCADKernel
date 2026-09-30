// IntTools_Curve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! A face intersection curve, its two pcurves and tolerances.
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::{
    adaptor3d::Adaptor3dCurve,
    geom::{GeomBoundedCurve, GeomCurve},
    geom_adaptor::GeomAdaptorCurve,
};
use rustycad_tkmath::{geom_abs::CurveType, gp::Point3};
/// `IntTools_Curve`; `Curve::default().has_bounds() == false`.
#[derive(Clone, Default)]
pub struct Curve {
    curve: Handle<dyn GeomCurve>,
    first2d: Handle<dyn Geom2dCurve>,
    second2d: Handle<dyn Geom2dCurve>,
    tolerance: f64,
    tangential_tolerance: f64,
}
impl Curve {
    /// Copies curve handles and tolerances; `Curve::new(&curve, &p1, &p2, 1e-7, 0.0)`.
    pub fn new(
        curve: &Handle<dyn GeomCurve>,
        first2d: &Handle<dyn Geom2dCurve>,
        second2d: &Handle<dyn Geom2dCurve>,
        tolerance: f64,
        tangential_tolerance: f64,
    ) -> Self {
        Self {
            curve: curve.clone(),
            first2d: first2d.clone(),
            second2d: second2d.clone(),
            tolerance,
            tangential_tolerance,
        }
    }
    /// Replaces all handles; `result.set_curves(&curve, &p1, &p2)`.
    pub fn set_curves(
        &mut self,
        curve: &Handle<dyn GeomCurve>,
        first2d: &Handle<dyn Geom2dCurve>,
        second2d: &Handle<dyn Geom2dCurve>,
    ) {
        self.set_curve(curve);
        self.set_first_curve2d(first2d);
        self.set_second_curve2d(second2d);
    }
    /// Replaces the spatial curve; `result.set_curve(&curve)`.
    pub fn set_curve(&mut self, curve: &Handle<dyn GeomCurve>) {
        self.curve = curve.clone();
    }
    /// Replaces the first pcurve; `result.set_first_curve2d(&pcurve)`.
    pub fn set_first_curve2d(&mut self, curve: &Handle<dyn Geom2dCurve>) {
        self.first2d = curve.clone();
    }
    /// Replaces the second pcurve; `result.set_second_curve2d(&pcurve)`.
    pub fn set_second_curve2d(&mut self, curve: &Handle<dyn Geom2dCurve>) {
        self.second2d = curve.clone();
    }
    /// Replaces the spatial tolerance; `result.set_tolerance(1e-7)`.
    pub fn set_tolerance(&mut self, tolerance: f64) {
        self.tolerance = tolerance;
    }
    /// Replaces the tangential tolerance; `result.set_tangential_tolerance(1e-6)`.
    pub fn set_tangential_tolerance(&mut self, tolerance: f64) {
        self.tangential_tolerance = tolerance;
    }
    /// Borrows the spatial curve handle; `result.curve()`.
    pub fn curve(&self) -> &Handle<dyn GeomCurve> {
        &self.curve
    }
    /// Borrows the first pcurve handle; `result.first_curve2d()`.
    pub fn first_curve2d(&self) -> &Handle<dyn Geom2dCurve> {
        &self.first2d
    }
    /// Borrows the second pcurve handle; `result.second_curve2d()`.
    pub fn second_curve2d(&self) -> &Handle<dyn Geom2dCurve> {
        &self.second2d
    }
    /// Reads the spatial tolerance; `result.tolerance()`.
    pub fn tolerance(&self) -> f64 {
        self.tolerance
    }
    /// Reads the tangential tolerance; `result.tangential_tolerance()`.
    pub fn tangential_tolerance(&self) -> f64 {
        self.tangential_tolerance
    }
    /// Checks dynamic bounded-curve membership; `result.has_bounds()`.
    pub fn has_bounds(&self) -> bool {
        !self.curve.cast::<dyn GeomBoundedCurve>().is_null()
    }
    /// Updates outputs only for bounded curves; `result.bounds(&mut a, &mut b, &mut p, &mut q)?`.
    pub fn bounds(
        &self,
        first: &mut f64,
        last: &mut f64,
        p: &mut Point3,
        q: &mut Point3,
    ) -> Result<bool, Failure> {
        if !self.has_bounds() {
            return Ok(false);
        }
        *first = self.curve.first_parameter();
        *last = self.curve.last_parameter();
        *p = self.curve.eval_d0(*first)?;
        *q = self.curve.eval_d0(*last)?;
        Ok(true)
    }
    /// Evaluates using the native OR predicate; `result.d0(0.5, &mut point)?`.
    pub fn d0(&self, parameter: f64, point: &mut Point3) -> Result<bool, Failure> {
        // 3d097a0 uses OR, so a bounded curve is also evaluated outside its interval.
        let inside =
            parameter >= self.curve.first_parameter() || parameter <= self.curve.last_parameter();
        if inside {
            *point = self.curve.eval_d0(parameter)?;
        }
        Ok(inside)
    }
    /// Reports the adaptor's curve classification; `result.curve_type()?`.
    pub fn curve_type(&self) -> Result<CurveType, Failure> {
        GeomAdaptorCurve::with_curve(&self.curve)?.get_type()
    }
}
