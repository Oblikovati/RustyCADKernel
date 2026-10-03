use crate::shape_analysis::ShapeAnalysisCurve;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurveOnSurface;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkmath::gp::Point3;

pub(super) fn project_pcurve(
    curve: &Handle<dyn Geom2dCurve>,
    surface: &Handle<dyn GeomSurface>,
    range: (f64, f64),
    point: Point3,
    precision: f64,
) -> Result<(f64, f64), Failure> {
    let curve = adapt_geometry(curve, surface, range)?;
    let (distance, _, parameter) =
        ShapeAnalysisCurve.project_adaptor(&curve, point, precision, false)?;
    Ok((distance, parameter))
}

pub(super) fn adapt_geometry(
    curve: &Handle<dyn Geom2dCurve>,
    surface: &Handle<dyn GeomSurface>,
    range: (f64, f64),
) -> Result<Adaptor3dCurveOnSurface, Failure> {
    let curve_adaptor: Handle<dyn Adaptor2dCurve2d> =
        Handle::new(Geom2dAdaptorCurve::with_range(curve, range.0, range.1)?).cast();
    let surface_adaptor: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(surface)?).cast();
    Adaptor3dCurveOnSurface::with_curve(&curve_adaptor, &surface_adaptor)
}

pub(super) fn midpoint(
    curve: &Handle<dyn GeomCurve>,
    range: (f64, f64),
) -> Result<Point3, Failure> {
    curve.value((range.0 + range.1) / 2.0)
}
