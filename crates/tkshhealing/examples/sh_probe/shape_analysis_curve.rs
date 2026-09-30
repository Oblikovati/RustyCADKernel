//! Original ShapeAnalysis_Curve projection paths, including mutable outputs on failure.
use super::brep_lib_common::geom_curve;
use super::common::{Reader, attempt, end, pnt, real};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkmath::gp::Point3;
use rustycad_tkshhealing::shape_analysis::{
    CurveProjection, ShapeAnalysisCurve, adjust_by_period, adjust_to_period,
};
struct ProjectionRequest {
    operation: i32,
    curve: Handle<dyn GeomCurve>,
    point: Point3,
    tolerance: f64,
    bounds: (f64, f64),
    previous: f64,
    adjust: bool,
}
fn read_projection(r: &mut Reader) -> Result<ProjectionRequest, Failure> {
    Ok(ProjectionRequest {
        operation: r.integer(),
        curve: geom_curve(r)?,
        point: r.pnt(),
        tolerance: r.real(),
        bounds: (r.real(), r.real()),
        previous: r.real(),
        adjust: r.flag(),
    })
}
fn run_projection(a: &ProjectionRequest, output: &mut CurveProjection) -> Result<f64, Failure> {
    let tool = ShapeAnalysisCurve;
    match a.operation {
        0 => tool.project_full_into(&a.curve, a.point, a.tolerance, a.adjust, output),
        1 => tool.project_into(&a.curve, a.point, a.tolerance, a.bounds, a.adjust, output),
        4 => tool.next_project_into(
            a.previous,
            &a.curve,
            a.point,
            a.tolerance,
            a.bounds,
            a.adjust,
            output,
        ),
        _ => run_adaptor_projection(a, output),
    }
}
fn run_adaptor_projection(
    a: &ProjectionRequest,
    output: &mut CurveProjection,
) -> Result<f64, Failure> {
    let adaptor = GeomAdaptorCurve::with_range(&a.curve, a.bounds.0, a.bounds.1)?;
    let tool = ShapeAnalysisCurve;
    match a.operation {
        2 => tool.project_adaptor_into(&adaptor, a.point, a.tolerance, a.adjust, output),
        3 => tool.project_act(&adaptor, a.point, a.tolerance, output),
        _ => tool.next_project_adaptor_into(a.previous, &adaptor, a.point, a.tolerance, output),
    }
}
/// Print original distance and outputs, including after failure; `projection_case(numbers)`.
pub fn projection_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut output = CurveProjection {
        point: Point3::new(91.0, -37.0, 18.0),
        parameter: 123.0,
    };
    attempt(|| {
        let request = read_projection(&mut r)?;
        real(run_projection(&request, &mut output)?);
        Ok(())
    });
    pnt(output.point);
    real(output.parameter);
    end();
}
/// Native namespace period shifts; `period_case(&[7.0,0.0,6.0])`.
pub fn period_case(values: &[f64]) {
    real(adjust_by_period(values[0], values[1], values[2]));
    real(adjust_to_period(values[0], values[1], values[2]));
    end();
}
