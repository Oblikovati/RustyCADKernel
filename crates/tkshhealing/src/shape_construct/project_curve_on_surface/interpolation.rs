use super::*;
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkgeomalgo::geom2d_api::Interpolate;

impl ShapeConstructProjectCurveOnSurface {
    pub(super) fn interpolate_pcurve(
        &self,
        points: &Array1<'_, Point2>,
        parameters: &Array1<'_, f64>,
    ) -> Handle<dyn Geom2dCurve> {
        let tolerance = self.fields.read().precision / (100. * f64::from(points.length()));
        interpolate_points(points, parameters, tolerance).unwrap_or_default()
    }
}

fn interpolate_points(
    points: &Array1<'_, Point2>,
    parameters: &Array1<'_, f64>,
    mut tolerance: f64,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let (points, parameters) = checked_points(points, parameters, &mut tolerance);
    if points.length() < 2 {
        return Ok(Handle::default());
    }
    let points = Handle::new(HArray1::from_owned(points));
    let parameters = Handle::new(HArray1::from_owned(parameters));
    let mut interpolation = Interpolate::with_parameters(&points, &parameters, false, tolerance)?;
    interpolation.perform()?;
    if !interpolation.is_done() {
        return Ok(Handle::default());
    }
    Ok(interpolation.curve()?.cast())
}

fn checked_points(
    points: &Array1<'_, Point2>,
    parameters: &Array1<'_, f64>,
    tolerance: &mut f64,
) -> (Array1<'static, Point2>, Array1<'static, f64>) {
    let (mut keep, minimum, dropped) = point_retention(points);
    if minimum < f64::MAX {
        *tolerance = 0.9 * minimum.sqrt();
    }
    let mut selected: Vec<Point2> = (points.lower()..=points.upper())
        .map(|i| points[i])
        .collect();
    if dropped > 0 && points.length() - dropped < 2 {
        keep[0] = true;
        *keep.last_mut().unwrap() = true;
        let last = selected.last_mut().unwrap();
        *last = Point2::new(last.x() + *tolerance, last.y() + *tolerance);
    }
    collect_retained(&selected, parameters, &keep)
}

fn point_retention(points: &Array1<'_, Point2>) -> (Vec<bool>, f64, i32) {
    let mut keep = vec![true; points.length() as usize];
    let (mut previous, mut valid, mut minimum, mut dropped) =
        (points[points.lower()], points.lower(), f64::MAX, 0);
    for index in points.lower() + 1..=points.upper() {
        let distance = previous.square_distance(points[index]);
        if distance < rustycad_tkmath::gp::resolution() {
            dropped += 1;
            let remove = if index == points.upper() {
                valid
            } else {
                index
            };
            keep[(remove - points.lower()) as usize] = false;
        } else {
            minimum = rustycad_tkg2d::geom2d::native::std_min(minimum, distance);
            valid = index;
            previous = points[index];
        }
    }
    (keep, minimum, dropped)
}

fn collect_retained(
    points: &[Point2],
    parameters: &Array1<'_, f64>,
    keep: &[bool],
) -> (Array1<'static, Point2>, Array1<'static, f64>) {
    let count = keep.iter().filter(|&&value| value).count() as i32;
    let lower = parameters.lower();
    let mut result = Array1::with_bounds(lower, lower + count - 1).unwrap();
    let mut knots = Array1::with_bounds(lower, lower + count - 1).unwrap();
    let mut next = lower;
    for (index, &point) in points.iter().enumerate() {
        if !keep[index] {
            continue;
        }
        result[next] = point;
        knots[next] = parameters[lower + index as i32];
        next += 1;
    }
    (result, knots)
}
