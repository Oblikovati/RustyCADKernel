//! ShapeAnalysis_Curve.cxx's deliberately ordered 25/40/20/25/40 probes.
use super::projection_state::{CurveProjection, ProjectionSearch};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkgeombase::extrema::LocateExtPC;
use rustycad_tkmath::gp::Point3;

pub(super) fn project_segments(
    curve: &dyn Adaptor3dCurve,
    point: Point3,
    count: i32,
    bounds: &mut (f64, f64),
    distance: &mut f64,
    output: &mut CurveProjection,
) -> Result<(), Failure> {
    if count <= 0 {
        return Ok(());
    }
    let step = (bounds.1 - bounds.0) / f64::from(count);
    let mut best = *distance * *distance;
    let mut changed = false;
    for i in 0..=count {
        let parameter = bounds.0 + step * f64::from(i);
        let candidate = curve.value(parameter)?;
        let squared = candidate.square_distance(&point);
        if squared < best {
            best = squared;
            *output = CurveProjection {
                point: candidate,
                parameter,
            };
            changed = true;
        }
    }
    if changed {
        *distance = best.sqrt();
    }
    bounds.1 = std_min(bounds.1, output.parameter + step);
    bounds.0 = std_max(bounds.0, output.parameter - step);
    Ok(())
}
pub(super) fn local_projection(
    curve: &dyn Adaptor3dCurve,
    point: Point3,
    previous: f64,
    bounds: (f64, f64),
    tolerance: f64,
    output: &mut CurveProjection,
) -> Result<Option<f64>, Failure> {
    let local = LocateExtPC::with_range(point, curve, previous, bounds, tolerance)?;
    if !local.is_done() {
        return Ok(None);
    }
    let result = local.point()?;
    output.parameter = result.parameter();
    output.point = *result.value();
    Ok(Some(point.distance(&output.point)))
}
pub(super) fn sample_projection(
    curve: &dyn Adaptor3dCurve,
    point: Point3,
    tolerance: f64,
    search: &mut ProjectionSearch,
    output: &mut CurveProjection,
) -> Result<f64, Failure> {
    let mut distance = precision::infinite();
    project_segments(curve, point, 25, &mut search.bounds, &mut distance, output)?;
    if distance <= tolerance {
        return Ok(distance);
    }
    if let Some(newton) = local_projection(
        curve,
        point,
        output.parameter,
        search.bounds,
        tolerance,
        output,
    )? && newton < search.minimum
    {
        return Ok(newton);
    }
    refine_projection(curve, point, tolerance, search, output, distance)
}
fn refine_projection(
    curve: &dyn Adaptor3dCurve,
    point: Point3,
    tolerance: f64,
    search: &mut ProjectionSearch,
    output: &mut CurveProjection,
    mut distance: f64,
) -> Result<f64, Failure> {
    // These native counts are empirical; changing them changes which minimum wins.
    for count in [40, 20, 25, 40] {
        project_segments(
            curve,
            point,
            count,
            &mut search.bounds,
            &mut distance,
            output,
        )?;
        if distance <= tolerance {
            return Ok(distance);
        }
    }
    if distance > search.minimum {
        distance = search.minimum;
        *output = search.computed;
    }
    Ok(distance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
    #[test]
    fn no_segments_preserves_all_outputs_without_evaluating_curve() {
        let curve = GeomAdaptorCurve::default();
        let mut bounds = (2.0, 3.0);
        let mut distance = 8.0;
        let mut output = CurveProjection {
            point: Point3::new(1.0, 2.0, 3.0),
            parameter: 5.0,
        };
        let original = output;
        project_segments(
            &curve,
            Point3::default(),
            0,
            &mut bounds,
            &mut distance,
            &mut output,
        )
        .unwrap();
        assert_eq!(bounds, (2.0, 3.0));
        assert_eq!(distance, 8.0);
        assert_eq!(output, original);
    }
}
