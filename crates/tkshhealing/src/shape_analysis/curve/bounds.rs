//! Native sampled bounds with the ten-step coordinate-extremum refinement.
use super::ShapeAnalysisCurve;
use rustycad_tkernel::{
    ncollection::Array1,
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::{
    adaptor2d::Adaptor2dCurve2d, geom2d::Geom2dCurve, geom2d_adaptor::Geom2dAdaptorCurve,
};
use rustycad_tkmath::{
    bnd::BoundingBox2,
    geom_abs::Shape,
    gp::{Point2, Vector2},
};

fn sample_parameters(
    curve: &Handle<dyn Geom2dCurve>,
    bounds: (f64, f64),
    count: i32,
) -> Result<Array1<'static, f64>, Failure> {
    let adaptor = Geom2dAdaptorCurve::with_range(curve, bounds.0, bounds.1)?;
    let intervals = adaptor.nb_intervals(Shape::C2)?;
    let samples = if intervals < 2 { count - 1 } else { intervals };
    // Native negative sample counts produce an unused empty array and no box changes.
    let mut parameters = Array1::with_bounds(1, (samples + 1).max(0))
        .map_err(|e| e.into_failure("ShapeAnalysis_Curve::FillBndBox"))?;
    if samples == intervals {
        adaptor.intervals(&mut parameters, Shape::C2)?;
    } else {
        let step = (bounds.1 - bounds.0) / f64::from(samples);
        for i in 0..=samples {
            parameters[i + 1] = bounds.0 + f64::from(i) * step;
        }
    }
    Ok(parameters)
}
fn clip_extremum(parameter: &mut f64, previous: f64, bound: f64, outside: &mut i32) -> bool {
    let old = *outside;
    *outside += 1;
    if old > 2 || previous == bound {
        return false;
    }
    *parameter = bound;
    true
}
fn extremum_inside(
    parameter: &mut f64,
    previous: f64,
    bounds: (f64, f64),
    outside: &mut i32,
) -> bool {
    if *parameter < bounds.0 && !clip_extremum(parameter, previous, bounds.0, outside) {
        return false;
    }
    if *parameter > bounds.1 && !clip_extremum(parameter, previous, bounds.1, outside) {
        return false;
    }
    true
}
fn coordinate_extremum(
    curve: &dyn Geom2dCurve,
    bounds: (f64, f64),
    direction: Vector2,
) -> Result<Option<Point2>, Failure> {
    let mut parameter = (bounds.0 + bounds.1) * 0.5;
    let mut outside = 0;
    let mut point = Point2::default();
    for _ in 0..10 {
        let previous = parameter;
        let (candidate, first, second) = curve.d2(parameter)?;
        point = candidate;
        let determinant = second.dot(direction);
        if determinant.abs() < 1e-10 {
            return Ok(Some(point));
        }
        parameter -= first.dot(direction) / determinant;
        // Native returns the previous iterate's point without reevaluating D2.
        if (parameter - previous).abs() < precision::p_confusion() {
            return Ok(Some(point));
        }
        if !extremum_inside(&mut parameter, previous, bounds, &mut outside) {
            return Ok(None);
        }
    }
    Ok(Some(point))
}
fn add_extrema(
    curve: &dyn Geom2dCurve,
    bounds: (f64, f64),
    output: &mut BoundingBox2,
) -> Result<(), Failure> {
    for direction in [Vector2::new(1.0, 0.0), Vector2::new(0.0, 1.0)] {
        if let Some(point) = coordinate_extremum(curve, bounds, direction)? {
            output.add_point(point);
        }
    }
    Ok(())
}
fn fill_sampled(
    curve: &dyn Geom2dCurve,
    bounds: (f64, f64),
    count: i32,
    output: &mut BoundingBox2,
) -> Result<(), Failure> {
    let segments = if count < 2 { 1 } else { count - 1 };
    let step = (bounds.1 - bounds.0) / f64::from(segments);
    for i in 0..=segments {
        output.add_point(curve.value(bounds.0 + f64::from(i) * step)?);
    }
    Ok(())
}
impl ShapeAnalysisCurve {
    /// Accumulate native sampled or Newton-refined bounds; `ShapeAnalysisCurve.fill_bnd_box(&curve,0.0,1.0,10,true,&mut bounds)?`.
    pub fn fill_bnd_box(
        &self,
        curve: &Handle<dyn Geom2dCurve>,
        first: f64,
        last: f64,
        count: i32,
        exact: bool,
        output: &mut BoundingBox2,
    ) -> Result<(), Failure> {
        let geometry = curve.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "ShapeAnalysis_Curve: null curve; expected live Geom2d_Curve",
            )
        })?;
        if !exact {
            return fill_sampled(geometry, (first, last), count, output);
        }
        let parameters = sample_parameters(curve, (first, last), count)?;
        for i in 1..=parameters.upper() {
            let first = parameters[i];
            output.add_point(geometry.value(first)?);
            if i < parameters.upper() {
                add_extrema(geometry, (first, parameters[i + 1]), output)?;
            }
        }
        Ok(())
    }
}
