//! HLRBRep_BSurfaceTool.hxx/.lxx/.cxx, OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkernel::{
    ncollection::Array1,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::{
    adaptor3d::{Adaptor3dCurve, Adaptor3dSurface},
    geom::{GeomBSplineSurface, GeomBezierSurface, GeomSurfaceD1, GeomSurfaceD2, GeomSurfaceD3},
};
use rustycad_tkmath::{
    geom_abs::{Shape, SurfaceType},
    gp::{Axis1, Cone3, Cylinder3, Direction3, Plane3, Point3, Sphere3, Torus3, Vector3},
};
mod sampling;
/// Exact-HLR access to a face's surface; `BSurfaceTool::value(&surface,0.5,0.5)?`.
pub struct BSurfaceTool;
impl BSurfaceTool {
    /// Native adaptor query; `BSurfaceTool::first_u_parameter(&surface)?`.
    pub fn first_u_parameter(surface: &BRepAdaptorSurface) -> Result<f64, Failure> {
        surface.first_u_parameter()
    }
    /// Native adaptor query; `BSurfaceTool::last_u_parameter(&surface)?`.
    pub fn last_u_parameter(surface: &BRepAdaptorSurface) -> Result<f64, Failure> {
        surface.last_u_parameter()
    }
    /// Native adaptor query; `BSurfaceTool::first_v_parameter(&surface)?`.
    pub fn first_v_parameter(surface: &BRepAdaptorSurface) -> Result<f64, Failure> {
        surface.first_v_parameter()
    }
    /// Native adaptor query; `BSurfaceTool::last_v_parameter(&surface)?`.
    pub fn last_v_parameter(surface: &BRepAdaptorSurface) -> Result<f64, Failure> {
        surface.last_v_parameter()
    }
    /// Native adaptor query; `BSurfaceTool::u_continuity(&surface)?`.
    pub fn u_continuity(surface: &BRepAdaptorSurface) -> Result<Shape, Failure> {
        surface.u_continuity()
    }
    /// Native adaptor query; `BSurfaceTool::v_continuity(&surface)?`.
    pub fn v_continuity(surface: &BRepAdaptorSurface) -> Result<Shape, Failure> {
        surface.v_continuity()
    }
    /// Native adaptor query; `BSurfaceTool::nb_u_intervals(&surface, shape)?`.
    pub fn nb_u_intervals(surface: &BRepAdaptorSurface, shape: Shape) -> Result<i32, Failure> {
        surface.nb_u_intervals(shape)
    }
    /// Native adaptor query; `BSurfaceTool::nb_v_intervals(&surface, shape)?`.
    pub fn nb_v_intervals(surface: &BRepAdaptorSurface, shape: Shape) -> Result<i32, Failure> {
        surface.nb_v_intervals(shape)
    }
    /// Native adaptor query; `BSurfaceTool::u_intervals(&surface, &mut bounds, shape)?`.
    pub fn u_intervals(
        surface: &BRepAdaptorSurface,
        bounds: &mut Array1<'_, f64>,
        shape: Shape,
    ) -> Result<(), Failure> {
        surface.u_intervals(bounds, shape)
    }
    /// Native adaptor query; `BSurfaceTool::v_intervals(&surface, &mut bounds, shape)?`.
    pub fn v_intervals(
        surface: &BRepAdaptorSurface,
        bounds: &mut Array1<'_, f64>,
        shape: Shape,
    ) -> Result<(), Failure> {
        surface.v_intervals(bounds, shape)
    }
    /// Native adaptor query; `BSurfaceTool::u_trim(&surface, first, last, tolerance)?`.
    pub fn u_trim(
        surface: &BRepAdaptorSurface,
        first: f64,
        last: f64,
        tolerance: f64,
    ) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
        surface.u_trim(first, last, tolerance)
    }
    /// Native adaptor query; `BSurfaceTool::v_trim(&surface, first, last, tolerance)?`.
    pub fn v_trim(
        surface: &BRepAdaptorSurface,
        first: f64,
        last: f64,
        tolerance: f64,
    ) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
        surface.v_trim(first, last, tolerance)
    }
    /// Native adaptor query; `BSurfaceTool::is_u_closed(&surface)?`.
    pub fn is_u_closed(surface: &BRepAdaptorSurface) -> Result<bool, Failure> {
        surface.is_u_closed()
    }
    /// Native adaptor query; `BSurfaceTool::is_v_closed(&surface)?`.
    pub fn is_v_closed(surface: &BRepAdaptorSurface) -> Result<bool, Failure> {
        surface.is_v_closed()
    }
    /// Native adaptor query; `BSurfaceTool::is_u_periodic(&surface)?`.
    pub fn is_u_periodic(surface: &BRepAdaptorSurface) -> Result<bool, Failure> {
        surface.is_u_periodic()
    }
    /// Native adaptor query; `BSurfaceTool::u_period(&surface)?`.
    pub fn u_period(surface: &BRepAdaptorSurface) -> Result<f64, Failure> {
        surface.u_period()
    }
    /// Native adaptor query; `BSurfaceTool::is_v_periodic(&surface)?`.
    pub fn is_v_periodic(surface: &BRepAdaptorSurface) -> Result<bool, Failure> {
        surface.is_v_periodic()
    }
    /// Native adaptor query; `BSurfaceTool::v_period(&surface)?`.
    pub fn v_period(surface: &BRepAdaptorSurface) -> Result<f64, Failure> {
        surface.v_period()
    }
    /// Native adaptor query; `BSurfaceTool::value(&surface, u, v)?`.
    pub fn value(surface: &BRepAdaptorSurface, u: f64, v: f64) -> Result<Point3, Failure> {
        surface.value(u, v)
    }
    /// Native adaptor query; `BSurfaceTool::d0(&surface, u, v)?`.
    pub fn d0(surface: &BRepAdaptorSurface, u: f64, v: f64) -> Result<Point3, Failure> {
        surface.d0(u, v)
    }
    /// Native adaptor query; `BSurfaceTool::d1(&surface, u, v)?`.
    pub fn d1(surface: &BRepAdaptorSurface, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        surface.d1(u, v)
    }
    /// Native adaptor query; `BSurfaceTool::d2(&surface, u, v)?`.
    pub fn d2(surface: &BRepAdaptorSurface, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure> {
        surface.d2(u, v)
    }
    /// Native adaptor query; `BSurfaceTool::d3(&surface, u, v)?`.
    pub fn d3(surface: &BRepAdaptorSurface, u: f64, v: f64) -> Result<GeomSurfaceD3, Failure> {
        surface.d3(u, v)
    }
    /// Native adaptor query; `BSurfaceTool::dn(&surface, u, v, nu, nv)?`.
    pub fn dn(
        surface: &BRepAdaptorSurface,
        u: f64,
        v: f64,
        nu: i32,
        nv: i32,
    ) -> Result<Vector3, Failure> {
        surface.dn(u, v, nu, nv)
    }
    /// Native adaptor query; `BSurfaceTool::u_resolution(&surface, r3d)?`.
    pub fn u_resolution(surface: &BRepAdaptorSurface, r3d: f64) -> Result<f64, Failure> {
        surface.u_resolution(r3d)
    }
    /// Native adaptor query; `BSurfaceTool::v_resolution(&surface, r3d)?`.
    pub fn v_resolution(surface: &BRepAdaptorSurface, r3d: f64) -> Result<f64, Failure> {
        surface.v_resolution(r3d)
    }
    /// Native adaptor query; `BSurfaceTool::get_type(&surface)?`.
    pub fn get_type(surface: &BRepAdaptorSurface) -> Result<SurfaceType, Failure> {
        surface.get_type()
    }
    /// Native adaptor query; `BSurfaceTool::plane(&surface)?`.
    pub fn plane(surface: &BRepAdaptorSurface) -> Result<Plane3, Failure> {
        surface.plane()
    }
    /// Native adaptor query; `BSurfaceTool::cylinder(&surface)?`.
    pub fn cylinder(surface: &BRepAdaptorSurface) -> Result<Cylinder3, Failure> {
        surface.cylinder()
    }
    /// Native adaptor query; `BSurfaceTool::cone(&surface)?`.
    pub fn cone(surface: &BRepAdaptorSurface) -> Result<Cone3, Failure> {
        surface.cone()
    }
    /// Native adaptor query; `BSurfaceTool::sphere(&surface)?`.
    pub fn sphere(surface: &BRepAdaptorSurface) -> Result<Sphere3, Failure> {
        surface.sphere()
    }
    /// Native adaptor query; `BSurfaceTool::torus(&surface)?`.
    pub fn torus(surface: &BRepAdaptorSurface) -> Result<Torus3, Failure> {
        surface.torus()
    }
    /// Native adaptor query; `BSurfaceTool::u_degree(&surface)?`.
    pub fn u_degree(surface: &BRepAdaptorSurface) -> Result<i32, Failure> {
        surface.u_degree()
    }
    /// Native adaptor query; `BSurfaceTool::nb_u_poles(&surface)?`.
    pub fn nb_u_poles(surface: &BRepAdaptorSurface) -> Result<i32, Failure> {
        surface.nb_u_poles()
    }
    /// Native adaptor query; `BSurfaceTool::v_degree(&surface)?`.
    pub fn v_degree(surface: &BRepAdaptorSurface) -> Result<i32, Failure> {
        surface.v_degree()
    }
    /// Native adaptor query; `BSurfaceTool::nb_v_poles(&surface)?`.
    pub fn nb_v_poles(surface: &BRepAdaptorSurface) -> Result<i32, Failure> {
        surface.nb_v_poles()
    }
    /// Native adaptor query; `BSurfaceTool::nb_u_knots(&surface)?`.
    pub fn nb_u_knots(surface: &BRepAdaptorSurface) -> Result<i32, Failure> {
        surface.nb_u_knots()
    }
    /// Native adaptor query; `BSurfaceTool::nb_v_knots(&surface)?`.
    pub fn nb_v_knots(surface: &BRepAdaptorSurface) -> Result<i32, Failure> {
        surface.nb_v_knots()
    }
    /// Native adaptor query; `BSurfaceTool::is_u_rational(&surface)?`.
    pub fn is_u_rational(surface: &BRepAdaptorSurface) -> Result<bool, Failure> {
        surface.is_u_rational()
    }
    /// Native adaptor query; `BSurfaceTool::is_v_rational(&surface)?`.
    pub fn is_v_rational(surface: &BRepAdaptorSurface) -> Result<bool, Failure> {
        surface.is_v_rational()
    }
    /// Native adaptor query; `BSurfaceTool::bezier(&surface)?`.
    pub fn bezier(surface: &BRepAdaptorSurface) -> Result<Handle<GeomBezierSurface>, Failure> {
        surface.bezier()
    }
    /// Native adaptor query; `BSurfaceTool::bspline(&surface)?`.
    pub fn bspline(surface: &BRepAdaptorSurface) -> Result<Handle<GeomBSplineSurface>, Failure> {
        surface.bspline()
    }
    /// Native adaptor query; `BSurfaceTool::axe_of_revolution(&surface)?`.
    pub fn axe_of_revolution(surface: &BRepAdaptorSurface) -> Result<Axis1, Failure> {
        surface.axe_of_revolution()
    }
    /// Native adaptor query; `BSurfaceTool::direction(&surface)?`.
    pub fn direction(surface: &BRepAdaptorSurface) -> Result<Direction3, Failure> {
        surface.direction()
    }
    /// Native adaptor query; `BSurfaceTool::basis_curve(&surface)?`.
    pub fn basis_curve(
        surface: &BRepAdaptorSurface,
    ) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
        surface.basis_curve()
    }
}
