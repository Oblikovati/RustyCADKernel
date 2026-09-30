//! `BRepApprox_SurfaceTool` as the `ThePSurfaceTool` of the IntImp and ApproxInt
//! templates (the `ThePSurfaceTool` macro of every BRepApprox `_0.cxx`,
//! docs/topalgo-design.md section 12): each trait member calls the tool function of the
//! same name.
use super::surface_tool::{BRepApproxSurface, BRepApproxSurfaceTool as Tool};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::geom::GeomSurfaceD1;
use rustycad_tkgeomalgo::approx_int::ApproxIntSurfaceTool;
use rustycad_tkgeomalgo::int_imp::ThePSurfaceTool;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Cone3, Cylinder3, Plane3, Point3, Sphere3};

impl ThePSurfaceTool for Tool {
    type Surface = BRepApproxSurface;
    fn first_u_parameter(surface: &Self::Surface) -> Result<f64, Failure> {
        Tool::first_u_parameter(surface)
    }
    fn first_v_parameter(surface: &Self::Surface) -> Result<f64, Failure> {
        Tool::first_v_parameter(surface)
    }
    fn last_u_parameter(surface: &Self::Surface) -> Result<f64, Failure> {
        Tool::last_u_parameter(surface)
    }
    fn last_v_parameter(surface: &Self::Surface) -> Result<f64, Failure> {
        Tool::last_v_parameter(surface)
    }
    fn u_resolution(surface: &Self::Surface, r3d: f64) -> Result<f64, Failure> {
        Tool::u_resolution(surface, r3d)
    }
    fn v_resolution(surface: &Self::Surface, r3d: f64) -> Result<f64, Failure> {
        Tool::v_resolution(surface, r3d)
    }
    fn value(surface: &Self::Surface, u: f64, v: f64) -> Result<Point3, Failure> {
        Tool::value(surface, u, v)
    }
    fn d1(surface: &Self::Surface, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        Tool::d1(surface, u, v)
    }
}

impl ApproxIntSurfaceTool for Tool {
    fn is_u_periodic(surface: &Self::Surface) -> Result<bool, Failure> {
        Tool::is_u_periodic(surface)
    }
    fn is_v_periodic(surface: &Self::Surface) -> Result<bool, Failure> {
        Tool::is_v_periodic(surface)
    }
    fn u_period(surface: &Self::Surface) -> Result<f64, Failure> {
        Tool::u_period(surface)
    }
    fn v_period(surface: &Self::Surface) -> Result<f64, Failure> {
        Tool::v_period(surface)
    }
    fn get_type(surface: &Self::Surface) -> Result<SurfaceType, Failure> {
        Tool::get_type(surface)
    }
    fn plane(surface: &Self::Surface) -> Result<Plane3, Failure> {
        Tool::plane(surface)
    }
    fn cylinder(surface: &Self::Surface) -> Result<Cylinder3, Failure> {
        Tool::cylinder(surface)
    }
    fn sphere(surface: &Self::Surface) -> Result<Sphere3, Failure> {
        Tool::sphere(surface)
    }
    fn cone(surface: &Self::Surface) -> Result<Cone3, Failure> {
        Tool::cone(surface)
    }
}
