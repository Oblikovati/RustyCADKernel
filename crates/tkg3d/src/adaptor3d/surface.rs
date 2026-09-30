// Rust translation of ModelingData/TKG3d/Adaptor3d/Adaptor3d_Surface.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Adaptor3d_Surface`: the interface that surface algorithms evaluate through.
use super::curve::Adaptor3dCurve;
use crate::geom::errors::not_implemented;
use crate::geom::{
    GeomBSplineSurface, GeomBezierSurface, GeomSurfaceD1, GeomSurfaceD2, GeomSurfaceD3,
};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient, Transient};
use rustycad_tkernel::{impl_transient, impl_transient_interface};
use rustycad_tkmath::geom_abs::{Shape, SurfaceType};
use rustycad_tkmath::gp::{
    Axis1, Cone3, Cylinder3, Direction3, Plane3, Point3, Sphere3, Torus3, Vector3,
};

/// A surface seen by geometric algorithms (`Adaptor3d_Surface`).
///
/// Every virtual member of the native base class raises Standard_NotImplemented;
/// the provided methods here do the same, and each concrete adaptor overrides them.
/// The non-virtual `Value`, `D0`..`D3` and `DN` forward to the `EvalD*` members,
/// `D1`..`D3` returning the result records instead of output parameters;
/// `let point: Point3 = adaptor.value(0.5, 0.5)?;`.
pub trait Adaptor3dSurface: Transient {
    /// A new adaptor sharing the surface (`ShallowCopy`); `adaptor.shallow_copy()?`.
    fn shallow_copy(&self) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
        Err(not_implemented("Adaptor3d_Surface::ShallowCopy"))
    }
    /// First U parameter (`FirstUParameter`); `adaptor.first_u_parameter()?`.
    fn first_u_parameter(&self) -> Result<f64, Failure> {
        Err(not_implemented("Adaptor3d_Surface::FirstUParameter"))
    }
    /// Last U parameter (`LastUParameter`); `adaptor.last_u_parameter()?`.
    fn last_u_parameter(&self) -> Result<f64, Failure> {
        Err(not_implemented("Adaptor3d_Surface::LastUParameter"))
    }
    /// First V parameter (`FirstVParameter`); `adaptor.first_v_parameter()?`.
    fn first_v_parameter(&self) -> Result<f64, Failure> {
        Err(not_implemented("Adaptor3d_Surface::FirstVParameter"))
    }
    /// Last V parameter (`LastVParameter`); `adaptor.last_v_parameter()?`.
    fn last_v_parameter(&self) -> Result<f64, Failure> {
        Err(not_implemented("Adaptor3d_Surface::LastVParameter"))
    }
    /// Continuity in U (`UContinuity`); `adaptor.u_continuity()?`.
    fn u_continuity(&self) -> Result<Shape, Failure> {
        Err(not_implemented("Adaptor3d_Surface::UContinuity"))
    }
    /// Continuity in V (`VContinuity`); `adaptor.v_continuity()?`.
    fn v_continuity(&self) -> Result<Shape, Failure> {
        Err(not_implemented("Adaptor3d_Surface::VContinuity"))
    }
    /// Number of U intervals of continuity `shape` (`NbUIntervals`); `adaptor.nb_u_intervals(Shape::C2)?`.
    fn nb_u_intervals(&self, _shape: Shape) -> Result<i32, Failure> {
        Err(not_implemented("Adaptor3d_Surface::NbUIntervals"))
    }
    /// Number of V intervals of continuity `shape` (`NbVIntervals`); `adaptor.nb_v_intervals(Shape::C2)?`.
    fn nb_v_intervals(&self, _shape: Shape) -> Result<i32, Failure> {
        Err(not_implemented("Adaptor3d_Surface::NbVIntervals"))
    }
    /// The U interval bounds, `NbUIntervals + 1` values (`UIntervals`);
    /// `adaptor.u_intervals(&mut bounds, Shape::C2)?`.
    fn u_intervals(&self, _bounds: &mut Array1<'_, f64>, _shape: Shape) -> Result<(), Failure> {
        Err(not_implemented("Adaptor3d_Surface::UIntervals"))
    }
    /// The V interval bounds, `NbVIntervals + 1` values (`VIntervals`);
    /// `adaptor.v_intervals(&mut bounds, Shape::C2)?`.
    fn v_intervals(&self, _bounds: &mut Array1<'_, f64>, _shape: Shape) -> Result<(), Failure> {
        Err(not_implemented("Adaptor3d_Surface::VIntervals"))
    }
    /// The surface with U in `[first, last]` (`UTrim`); `adaptor.u_trim(0.0, 1.0, 1e-7)?`.
    fn u_trim(
        &self,
        _first: f64,
        _last: f64,
        _tolerance: f64,
    ) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
        Err(not_implemented("Adaptor3d_Surface::UTrim"))
    }
    /// The surface with V in `[first, last]` (`VTrim`); `adaptor.v_trim(0.0, 1.0, 1e-7)?`.
    fn v_trim(
        &self,
        _first: f64,
        _last: f64,
        _tolerance: f64,
    ) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
        Err(not_implemented("Adaptor3d_Surface::VTrim"))
    }
    /// Whether closed in U (`IsUClosed`); `adaptor.is_u_closed()?`.
    fn is_u_closed(&self) -> Result<bool, Failure> {
        Err(not_implemented("Adaptor3d_Surface::IsUClosed"))
    }
    /// Whether closed in V (`IsVClosed`); `adaptor.is_v_closed()?`.
    fn is_v_closed(&self) -> Result<bool, Failure> {
        Err(not_implemented("Adaptor3d_Surface::IsVClosed"))
    }
    /// Whether periodic in U (`IsUPeriodic`); `adaptor.is_u_periodic()?`.
    fn is_u_periodic(&self) -> Result<bool, Failure> {
        Err(not_implemented("Adaptor3d_Surface::IsUPeriodic"))
    }
    /// The U period (`UPeriod`); `adaptor.u_period()?`.
    fn u_period(&self) -> Result<f64, Failure> {
        Err(not_implemented("Adaptor3d_Surface::UPeriod"))
    }
    /// Whether periodic in V (`IsVPeriodic`); `adaptor.is_v_periodic()?`.
    fn is_v_periodic(&self) -> Result<bool, Failure> {
        Err(not_implemented("Adaptor3d_Surface::IsVPeriodic"))
    }
    /// The V period (`VPeriod`); `adaptor.v_period()?`.
    fn v_period(&self) -> Result<f64, Failure> {
        Err(not_implemented("Adaptor3d_Surface::VPeriod"))
    }
    /// Point of parameters `(u, v)` (`Value`, non-virtual); `adaptor.value(0.5, 0.5)?`.
    fn value(&self, u: f64, v: f64) -> Result<Point3, Failure> {
        self.eval_d0(u, v)
    }
    /// Point of parameters `(u, v)` (`D0`, non-virtual); `adaptor.d0(0.5, 0.5)?`.
    fn d0(&self, u: f64, v: f64) -> Result<Point3, Failure> {
        self.eval_d0(u, v)
    }
    /// Point and first derivatives (`D1`, non-virtual); `adaptor.d1(0.5, 0.5)?.d1u`.
    fn d1(&self, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        self.eval_d1(u, v)
    }
    /// Point and derivatives up to order 2 (`D2`, non-virtual); `adaptor.d2(0.5, 0.5)?.d2uv`.
    fn d2(&self, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure> {
        self.eval_d2(u, v)
    }
    /// Point and derivatives up to order 3 (`D3`, non-virtual); `adaptor.d3(0.5, 0.5)?.d3uuv`.
    fn d3(&self, u: f64, v: f64) -> Result<GeomSurfaceD3, Failure> {
        self.eval_d3(u, v)
    }
    /// Derivative of orders `(nu, nv)` (`DN`, non-virtual); `adaptor.dn(0.5, 0.5, 1, 1)?`.
    fn dn(&self, u: f64, v: f64, nu: i32, nv: i32) -> Result<Vector3, Failure> {
        self.eval_dn(u, v, nu, nv)
    }
    /// U resolution of the space resolution `r3d` (`UResolution`); `adaptor.u_resolution(1e-7)?`.
    fn u_resolution(&self, _r3d: f64) -> Result<f64, Failure> {
        Err(not_implemented("Adaptor3d_Surface::UResolution"))
    }
    /// V resolution of the space resolution `r3d` (`VResolution`); `adaptor.v_resolution(1e-7)?`.
    fn v_resolution(&self, _r3d: f64) -> Result<f64, Failure> {
        Err(not_implemented("Adaptor3d_Surface::VResolution"))
    }
    /// Kind of the underlying surface (`GetType`); `adaptor.get_type()?`.
    fn get_type(&self) -> Result<SurfaceType, Failure> {
        Err(not_implemented("Adaptor3d_Surface::GetType"))
    }
    /// The plane, for a `Plane` type (`Plane`); `adaptor.plane()?`.
    fn plane(&self) -> Result<Plane3, Failure> {
        Err(not_implemented("Adaptor3d_Surface::Plane"))
    }
    /// The cylinder, for a `Cylinder` type (`Cylinder`); `adaptor.cylinder()?`.
    fn cylinder(&self) -> Result<Cylinder3, Failure> {
        Err(not_implemented("Adaptor3d_Surface::Cylinder"))
    }
    /// The cone, for a `Cone` type (`Cone`); `adaptor.cone()?`.
    fn cone(&self) -> Result<Cone3, Failure> {
        Err(not_implemented("Adaptor3d_Surface::Cone"))
    }
    /// The sphere, for a `Sphere` type (`Sphere`); `adaptor.sphere()?`.
    fn sphere(&self) -> Result<Sphere3, Failure> {
        Err(not_implemented("Adaptor3d_Surface::Sphere"))
    }
    /// The torus, for a `Torus` type (`Torus`); `adaptor.torus()?`.
    fn torus(&self) -> Result<Torus3, Failure> {
        Err(not_implemented("Adaptor3d_Surface::Torus"))
    }
    /// U degree (`UDegree`); `adaptor.u_degree()?`.
    fn u_degree(&self) -> Result<i32, Failure> {
        Err(not_implemented("Adaptor3d_Surface::UDegree"))
    }
    /// Number of U poles (`NbUPoles`); `adaptor.nb_u_poles()?`.
    fn nb_u_poles(&self) -> Result<i32, Failure> {
        Err(not_implemented("Adaptor3d_Surface::NbUPoles"))
    }
    /// V degree (`VDegree`); `adaptor.v_degree()?`.
    fn v_degree(&self) -> Result<i32, Failure> {
        Err(not_implemented("Adaptor3d_Surface::VDegree"))
    }
    /// Number of V poles (`NbVPoles`); `adaptor.nb_v_poles()?`.
    fn nb_v_poles(&self) -> Result<i32, Failure> {
        Err(not_implemented("Adaptor3d_Surface::NbVPoles"))
    }
    /// Number of U knots (`NbUKnots`); `adaptor.nb_u_knots()?`.
    fn nb_u_knots(&self) -> Result<i32, Failure> {
        Err(not_implemented("Adaptor3d_Surface::NbUKnots"))
    }
    /// Number of V knots (`NbVKnots`); `adaptor.nb_v_knots()?`.
    fn nb_v_knots(&self) -> Result<i32, Failure> {
        Err(not_implemented("Adaptor3d_Surface::NbVKnots"))
    }
    /// Whether rational in U (`IsURational`); `adaptor.is_u_rational()?`.
    fn is_u_rational(&self) -> Result<bool, Failure> {
        Err(not_implemented("Adaptor3d_Surface::IsURational"))
    }
    /// Whether rational in V (`IsVRational`); `adaptor.is_v_rational()?`.
    fn is_v_rational(&self) -> Result<bool, Failure> {
        Err(not_implemented("Adaptor3d_Surface::IsVRational"))
    }
    /// The Bezier surface (`Bezier`); `adaptor.bezier()?`.
    fn bezier(&self) -> Result<Handle<GeomBezierSurface>, Failure> {
        Err(not_implemented("Adaptor3d_Surface::Bezier"))
    }
    /// The B-spline surface (`BSpline`); `adaptor.bspline()?`.
    fn bspline(&self) -> Result<Handle<GeomBSplineSurface>, Failure> {
        Err(not_implemented("Adaptor3d_Surface::BSpline"))
    }
    /// The axis of a surface of revolution (`AxeOfRevolution`); `adaptor.axe_of_revolution()?`.
    fn axe_of_revolution(&self) -> Result<Axis1, Failure> {
        Err(not_implemented("Adaptor3d_Surface::AxeOfRevolution"))
    }
    /// The direction of a surface of extrusion (`Direction`); `adaptor.direction()?`.
    fn direction(&self) -> Result<Direction3, Failure> {
        Err(not_implemented("Adaptor3d_Surface::Direction"))
    }
    /// The basis curve of a swept surface (`BasisCurve`); `adaptor.basis_curve()?`.
    fn basis_curve(&self) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
        Err(not_implemented("Adaptor3d_Surface::BasisCurve"))
    }
    /// The basis surface of an offset surface (`BasisSurface`); `adaptor.basis_surface()?`.
    fn basis_surface(&self) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
        Err(not_implemented("Adaptor3d_Surface::BasisSurface"))
    }
    /// The offset of an offset surface (`OffsetValue`); `adaptor.offset_value()?`.
    fn offset_value(&self) -> Result<f64, Failure> {
        Err(not_implemented("Adaptor3d_Surface::OffsetValue"))
    }
    /// Point of parameters `(u, v)` (`EvalD0`); `adaptor.eval_d0(0.5, 0.5)?`.
    fn eval_d0(&self, _u: f64, _v: f64) -> Result<Point3, Failure> {
        Err(not_implemented("Adaptor3d_Surface::EvalD0"))
    }
    /// Point and first derivatives (`EvalD1`); `adaptor.eval_d1(0.5, 0.5)?.d1u`.
    fn eval_d1(&self, _u: f64, _v: f64) -> Result<GeomSurfaceD1, Failure> {
        Err(not_implemented("Adaptor3d_Surface::EvalD1"))
    }
    /// Point and derivatives up to order 2 (`EvalD2`); `adaptor.eval_d2(0.5, 0.5)?.d2uv`.
    fn eval_d2(&self, _u: f64, _v: f64) -> Result<GeomSurfaceD2, Failure> {
        Err(not_implemented("Adaptor3d_Surface::EvalD2"))
    }
    /// Point and derivatives up to order 3 (`EvalD3`); `adaptor.eval_d3(0.5, 0.5)?.d3v`.
    fn eval_d3(&self, _u: f64, _v: f64) -> Result<GeomSurfaceD3, Failure> {
        Err(not_implemented("Adaptor3d_Surface::EvalD3"))
    }
    /// Derivative of orders `(nu, nv)` (`EvalDN`); `adaptor.eval_dn(0.5, 0.5, 1, 1)?`.
    fn eval_dn(&self, _u: f64, _v: f64, _nu: i32, _nv: i32) -> Result<Vector3, Failure> {
        Err(not_implemented("Adaptor3d_Surface::EvalDN"))
    }
}
impl_transient_interface!(dyn Adaptor3dSurface, "Adaptor3d_Surface", StandardTransient);

/// An instance of the non-abstract native base class itself, whose members all raise
/// Standard_NotImplemented; `Handle::new(Adaptor3dSurfaceBase::default())`.
///
/// Its dynamic type is `Adaptor3d_Surface`, as for `new Adaptor3d_Surface()`.
#[derive(Default)]
pub struct Adaptor3dSurfaceBase {
    parent: StandardTransient,
}
impl_transient!(
    Adaptor3dSurfaceBase,
    "Adaptor3d_Surface",
    StandardTransient,
    parent,
    interfaces: [dyn Adaptor3dSurface]
);
impl Adaptor3dSurface for Adaptor3dSurfaceBase {}
