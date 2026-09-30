// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepApprox/BRepApprox_SurfaceTool.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepApprox_SurfaceTool`: the static tool that the BRepApprox template instances call
//! on a `BRepAdaptor_Surface` (docs/topalgo-design.md section 12). Every member but the
//! sample counts calls the adaptor member of the same name (the `.lxx`); the sample
//! counts are 10 (the `.cxx`).
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{
    GeomBSplineSurface, GeomBezierSurface, GeomSurfaceD1, GeomSurfaceD2, GeomSurfaceD3,
};
use rustycad_tkmath::geom_abs::{Shape, SurfaceType};
use rustycad_tkmath::gp::{
    Axis1, Cone3, Cylinder3, Direction3, Plane3, Point3, Sphere3, Torus3, Vector3,
};

/// The surface of every BRepApprox tool call: the shared adaptor, as the native
/// `const BRepAdaptor_Surface&` (docs/topalgo-design.md section 12).
pub type BRepApproxSurface = Handle<BRepAdaptorSurface>;

/// The sample count of every `NbSamplesU`/`NbSamplesV` overload.
const NB_SAMPLES: i32 = 10;

/// `BRepApprox_SurfaceTool`: the surface tool of the BRepApprox instances of the
/// ApproxInt, IntImp and AppParCurves templates.
///
/// ```
/// use rustycad_tktopalgo::brep_approx::BRepApproxSurfaceTool;
/// use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
/// use rustycad_tkernel::standard::Handle;
/// let surface = Handle::new(BRepAdaptorSurface::default());
/// assert_eq!(BRepApproxSurfaceTool::nb_samples_u(&surface), 10);
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct BRepApproxSurfaceTool;

impl BRepApproxSurfaceTool {
    /// `FirstUParameter(S)`; `BRepApproxSurfaceTool::first_u_parameter(&s)?`.
    pub fn first_u_parameter(surface: &BRepApproxSurface) -> Result<f64, Failure> {
        surface.first_u_parameter()
    }
    /// `FirstVParameter(S)`; `BRepApproxSurfaceTool::first_v_parameter(&s)?`.
    pub fn first_v_parameter(surface: &BRepApproxSurface) -> Result<f64, Failure> {
        surface.first_v_parameter()
    }
    /// `LastUParameter(S)`; `BRepApproxSurfaceTool::last_u_parameter(&s)?`.
    pub fn last_u_parameter(surface: &BRepApproxSurface) -> Result<f64, Failure> {
        surface.last_u_parameter()
    }
    /// `LastVParameter(S)`; `BRepApproxSurfaceTool::last_v_parameter(&s)?`.
    pub fn last_v_parameter(surface: &BRepApproxSurface) -> Result<f64, Failure> {
        surface.last_v_parameter()
    }
    /// `NbUIntervals(S, Sh)`; `BRepApproxSurfaceTool::nb_u_intervals(&s, Shape::C2)?`.
    pub fn nb_u_intervals(surface: &BRepApproxSurface, shape: Shape) -> Result<i32, Failure> {
        surface.nb_u_intervals(shape)
    }
    /// `NbVIntervals(S, Sh)`; `BRepApproxSurfaceTool::nb_v_intervals(&s, Shape::C2)?`.
    pub fn nb_v_intervals(surface: &BRepApproxSurface, shape: Shape) -> Result<i32, Failure> {
        surface.nb_v_intervals(shape)
    }
    /// `UIntervals(S, T, Sh)`: the U bounds of the intervals into `bounds`;
    /// `BRepApproxSurfaceTool::u_intervals(&s, &mut t, Shape::C2)?`.
    pub fn u_intervals(
        surface: &BRepApproxSurface,
        bounds: &mut Array1<'_, f64>,
        shape: Shape,
    ) -> Result<(), Failure> {
        surface.u_intervals(bounds, shape)
    }
    /// `VIntervals(S, T, Sh)`: the V bounds of the intervals into `bounds`;
    /// `BRepApproxSurfaceTool::v_intervals(&s, &mut t, Shape::C2)?`.
    pub fn v_intervals(
        surface: &BRepApproxSurface,
        bounds: &mut Array1<'_, f64>,
        shape: Shape,
    ) -> Result<(), Failure> {
        surface.v_intervals(bounds, shape)
    }
    /// `UTrim(S, First, Last, Tol)`; `BRepApproxSurfaceTool::u_trim(&s, 0.0, 1.0, 1e-7)?`.
    pub fn u_trim(
        surface: &BRepApproxSurface,
        first: f64,
        last: f64,
        tolerance: f64,
    ) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
        surface.u_trim(first, last, tolerance)
    }
    /// `VTrim(S, First, Last, Tol)`; `BRepApproxSurfaceTool::v_trim(&s, 0.0, 1.0, 1e-7)?`.
    pub fn v_trim(
        surface: &BRepApproxSurface,
        first: f64,
        last: f64,
        tolerance: f64,
    ) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
        surface.v_trim(first, last, tolerance)
    }
    /// `IsUClosed(S)`; `BRepApproxSurfaceTool::is_u_closed(&s)?`.
    pub fn is_u_closed(surface: &BRepApproxSurface) -> Result<bool, Failure> {
        surface.is_u_closed()
    }
    /// `IsVClosed(S)`; `BRepApproxSurfaceTool::is_v_closed(&s)?`.
    pub fn is_v_closed(surface: &BRepApproxSurface) -> Result<bool, Failure> {
        surface.is_v_closed()
    }
    /// `IsUPeriodic(S)`; `BRepApproxSurfaceTool::is_u_periodic(&s)?`.
    pub fn is_u_periodic(surface: &BRepApproxSurface) -> Result<bool, Failure> {
        surface.is_u_periodic()
    }
    /// `UPeriod(S)`; `BRepApproxSurfaceTool::u_period(&s)?`.
    pub fn u_period(surface: &BRepApproxSurface) -> Result<f64, Failure> {
        surface.u_period()
    }
    /// `IsVPeriodic(S)`; `BRepApproxSurfaceTool::is_v_periodic(&s)?`.
    pub fn is_v_periodic(surface: &BRepApproxSurface) -> Result<bool, Failure> {
        surface.is_v_periodic()
    }
    /// `VPeriod(S)`; `BRepApproxSurfaceTool::v_period(&s)?`.
    pub fn v_period(surface: &BRepApproxSurface) -> Result<f64, Failure> {
        surface.v_period()
    }
    /// `Value(S, u, v)`; `BRepApproxSurfaceTool::value(&s, 0.5, 0.5)?`.
    pub fn value(surface: &BRepApproxSurface, u: f64, v: f64) -> Result<Point3, Failure> {
        surface.value(u, v)
    }
    /// `D0(S, u, v, P)`: the point; `BRepApproxSurfaceTool::d0(&s, 0.5, 0.5)?`.
    pub fn d0(surface: &BRepApproxSurface, u: f64, v: f64) -> Result<Point3, Failure> {
        surface.d0(u, v)
    }
    /// `D1(S, u, v, P, D1u, D1v)`: the outputs as one record;
    /// `BRepApproxSurfaceTool::d1(&s, 0.5, 0.5)?.d1u`.
    pub fn d1(surface: &BRepApproxSurface, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        surface.d1(u, v)
    }
    /// `D2(S, u, v, P, D1U, D1V, D2U, D2V, D2UV)`;
    /// `BRepApproxSurfaceTool::d2(&s, 0.5, 0.5)?.d2uv`.
    pub fn d2(surface: &BRepApproxSurface, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure> {
        surface.d2(u, v)
    }
    /// `D3(S, u, v, P, D1U, ..., D3UVV)`; `BRepApproxSurfaceTool::d3(&s, 0.5, 0.5)?.d3uuv`.
    pub fn d3(surface: &BRepApproxSurface, u: f64, v: f64) -> Result<GeomSurfaceD3, Failure> {
        surface.d3(u, v)
    }
    /// `DN(S, u, v, Nu, Nv)`; `BRepApproxSurfaceTool::dn(&s, 0.5, 0.5, 1, 1)?`.
    pub fn dn(
        surface: &BRepApproxSurface,
        u: f64,
        v: f64,
        nu: i32,
        nv: i32,
    ) -> Result<Vector3, Failure> {
        surface.dn(u, v, nu, nv)
    }
    /// `UResolution(S, R3d)`; `BRepApproxSurfaceTool::u_resolution(&s, 1e-7)?`.
    pub fn u_resolution(surface: &BRepApproxSurface, r3d: f64) -> Result<f64, Failure> {
        surface.u_resolution(r3d)
    }
    /// `VResolution(S, R3d)`; `BRepApproxSurfaceTool::v_resolution(&s, 1e-7)?`.
    pub fn v_resolution(surface: &BRepApproxSurface, r3d: f64) -> Result<f64, Failure> {
        surface.v_resolution(r3d)
    }
    /// `GetType(S)`; `BRepApproxSurfaceTool::get_type(&s)?`.
    pub fn get_type(surface: &BRepApproxSurface) -> Result<SurfaceType, Failure> {
        surface.get_type()
    }
    /// `Plane(S)`; `BRepApproxSurfaceTool::plane(&s)?`.
    pub fn plane(surface: &BRepApproxSurface) -> Result<Plane3, Failure> {
        surface.plane()
    }
    /// `Cylinder(S)`; `BRepApproxSurfaceTool::cylinder(&s)?`.
    pub fn cylinder(surface: &BRepApproxSurface) -> Result<Cylinder3, Failure> {
        surface.cylinder()
    }
    /// `Cone(S)`; `BRepApproxSurfaceTool::cone(&s)?`.
    pub fn cone(surface: &BRepApproxSurface) -> Result<Cone3, Failure> {
        surface.cone()
    }
    /// `Torus(S)`; `BRepApproxSurfaceTool::torus(&s)?`.
    pub fn torus(surface: &BRepApproxSurface) -> Result<Torus3, Failure> {
        surface.torus()
    }
    /// `Sphere(S)`; `BRepApproxSurfaceTool::sphere(&s)?`.
    pub fn sphere(surface: &BRepApproxSurface) -> Result<Sphere3, Failure> {
        surface.sphere()
    }
    /// `Bezier(S)`; `BRepApproxSurfaceTool::bezier(&s)?`.
    pub fn bezier(surface: &BRepApproxSurface) -> Result<Handle<GeomBezierSurface>, Failure> {
        surface.bezier()
    }
    /// `BSpline(S)`; `BRepApproxSurfaceTool::bspline(&s)?`.
    pub fn bspline(surface: &BRepApproxSurface) -> Result<Handle<GeomBSplineSurface>, Failure> {
        surface.bspline()
    }
    /// `AxeOfRevolution(S)`; `BRepApproxSurfaceTool::axe_of_revolution(&s)?`.
    pub fn axe_of_revolution(surface: &BRepApproxSurface) -> Result<Axis1, Failure> {
        surface.axe_of_revolution()
    }
    /// `Direction(S)`; `BRepApproxSurfaceTool::direction(&s)?`.
    pub fn direction(surface: &BRepApproxSurface) -> Result<Direction3, Failure> {
        surface.direction()
    }
    /// `BasisCurve(S)`; `BRepApproxSurfaceTool::basis_curve(&s)?`.
    pub fn basis_curve(surface: &BRepApproxSurface) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
        surface.basis_curve()
    }
    /// `NbSamplesU(S)`: 10 for every surface; `BRepApproxSurfaceTool::nb_samples_u(&s)`.
    pub fn nb_samples_u(_surface: &BRepApproxSurface) -> i32 {
        NB_SAMPLES
    }
    /// `NbSamplesV(S)`: 10 for every surface; `BRepApproxSurfaceTool::nb_samples_v(&s)`.
    pub fn nb_samples_v(_surface: &BRepApproxSurface) -> i32 {
        NB_SAMPLES
    }
    /// `NbSamplesU(S, u1, u2)`: 10 whatever the range;
    /// `BRepApproxSurfaceTool::nb_samples_u_range(&s, 0.0, 1.0)`.
    pub fn nb_samples_u_range(_surface: &BRepApproxSurface, _u1: f64, _u2: f64) -> i32 {
        NB_SAMPLES
    }
    /// `NbSamplesV(S, v1, v2)`: 10 whatever the range;
    /// `BRepApproxSurfaceTool::nb_samples_v_range(&s, 0.0, 1.0)`.
    pub fn nb_samples_v_range(_surface: &BRepApproxSurface, _v1: f64, _v2: f64) -> i32 {
        NB_SAMPLES
    }
}
