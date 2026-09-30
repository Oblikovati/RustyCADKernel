//! Global extrema followed by OCCT's elementary or sampled fallback.
use super::projection_elementary::elementary_projection;
use super::projection_segments::sample_projection;
use super::projection_state::{CurveProjection, ProjectionSearch};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkgeombase::extrema::ExtPC;
use rustycad_tkmath::gp::Point3;

fn extrema_projection(
    curve: &dyn Adaptor3dCurve,
    point: Point3,
    output: &mut CurveProjection,
) -> Result<bool, Failure> {
    let extrema = ExtPC::with_curve(point, curve, 1e-10)?;
    if !extrema.is_done() {
        return Ok(false);
    }
    let index = nearest_minimum(&extrema)?;
    if index == 0 {
        return Ok(false);
    }
    output.parameter = extrema.point(index)?.parameter();
    output.point = *extrema.point(index)?.value();
    Ok(true)
}
pub(super) fn project_act(
    curve: &dyn Adaptor3dCurve,
    point: Point3,
    tolerance: f64,
    output: &mut CurveProjection,
) -> Result<f64, Failure> {
    output.parameter = 0.0;
    // Native catches Standard_Failure only around the global extrema attempt.
    let found = extrema_projection(curve, point, output).unwrap_or(false);
    let mut search = ProjectionSearch::new(curve, *output)?;
    if found {
        search.remember(curve, point)?;
    }
    if !found || search.minimum > tolerance {
        output.parameter = 0.0;
        if !elementary_projection(curve, point, &mut search, output)? {
            return sample_projection(curve, point, tolerance, &mut search, output);
        }
    }
    finish_projection(point, &search, output)
}
fn finish_projection(
    point: Point3,
    search: &ProjectionSearch,
    output: &mut CurveProjection,
) -> Result<f64, Failure> {
    let (first, last) = search.bounds;
    if search.closed && (output.parameter < first || output.parameter > last) {
        output.parameter += super::periodic::adjust_by_period(
            output.parameter,
            0.5 * (first + last),
            search.period,
        );
    }
    if let Some(old) = search.old
        && old.point.square_distance(&point) < output.point.square_distance(&point)
    {
        *output = old;
    }
    Ok(output.point.distance(&point))
}

fn nearest_minimum(extrema: &ExtPC) -> Result<i32, Failure> {
    let (mut minimum, mut index) = (f64::MAX, 0);
    for i in 1..=extrema.nb_ext()? {
        if !extrema.is_min(i)? {
            continue;
        }
        let distance = extrema.square_distance(i)?;
        if distance < minimum {
            minimum = distance;
            index = i;
        }
    }
    Ok(index)
}
