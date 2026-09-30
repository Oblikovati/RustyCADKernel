//! Closed-form fallbacks retain OCCT's parameter-only periodic correction.
use super::projection_state::{CurveProjection, ProjectionSearch};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::elclib::{self, ElementaryCurve};
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Point3, resolution};
use std::f64::consts::TAU;

fn project_conic<C: ElementaryCurve<Point = Point3>>(
    curve: &C,
    point: Point3,
    output: &mut CurveProjection,
) -> Result<(), Failure> {
    output.parameter = elclib::parameter(curve, point)?;
    output.point = elclib::value(output.parameter, curve);
    Ok(())
}
fn project_circle(
    curve: &dyn Adaptor3dCurve,
    point: Point3,
    output: &mut CurveProjection,
) -> Result<(), Failure> {
    let circle = curve.circle()?;
    output.point = circle.position().location();
    if circle.radius() <= resolution() || point.square_distance(&output.point) <= resolution() {
        output.parameter = curve.first_parameter()?;
        let xyz = output.point.xyz() + circle.x_axis().direction().xyz() * circle.radius();
        output.point = Point3::new(xyz.x(), xyz.y(), xyz.z());
        return Ok(());
    }
    project_conic(&circle, point, output)
}
pub(super) fn elementary_projection(
    curve: &dyn Adaptor3dCurve,
    point: Point3,
    search: &mut ProjectionSearch,
    output: &mut CurveProjection,
) -> Result<bool, Failure> {
    match curve.get_type()? {
        CurveType::Circle => {
            project_circle(curve, point, output)?;
            search.closed = true;
            search.period = TAU;
        }
        CurveType::Ellipse => {
            project_conic(&curve.ellipse()?, point, output)?;
            search.closed = true;
            search.period = TAU;
        }
        CurveType::Line => project_conic(&curve.line()?, point, output)?,
        CurveType::Hyperbola => project_conic(&curve.hyperbola()?, point, output)?,
        CurveType::Parabola => project_conic(&curve.parabola()?, point, output)?,
        _ => return Ok(false),
    }
    Ok(true)
}
