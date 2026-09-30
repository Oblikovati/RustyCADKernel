use super::*;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomTrimmedCurve};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;

pub(super) const CONTROL_POINTS: i32 = 23;
pub(super) struct CurveSamples {
    pub points: Array1<'static, Point3>,
    pub parameters: Array1<'static, f64>,
    pub uv: Array1<'static, Point2>,
}

impl CurveSamples {
    pub fn generate(curve: &Handle<dyn GeomCurve>, range: (f64, f64)) -> Result<Self, Failure> {
        let count = sample_count(curve, range)?;
        let mut samples = Self::with_count(count);
        let step = (range.1 - range.0) / f64::from(count - 1);
        for index in 1..=count {
            let parameter = if index == 1 {
                range.0
            } else if index == count {
                range.1
            } else {
                range.0 + f64::from(index - 1) * step
            };
            samples.parameters[index] = parameter;
            samples.points[index] = curve.value(parameter)?;
        }
        Ok(samples)
    }

    pub(super) fn with_count(count: i32) -> Self {
        Self {
            points: Array1::with_bounds(1, count).unwrap(),
            parameters: Array1::with_bounds(1, count).unwrap(),
            uv: Array1::with_bounds(1, count).unwrap(),
        }
    }
}

pub(super) fn extracted_bspline(curve: &Handle<dyn GeomCurve>) -> Handle<GeomBSplineCurve> {
    let mut basis = curve.clone();
    loop {
        let trimmed = basis.cast::<GeomTrimmedCurve>();
        let Some(trimmed) = trimmed.get() else {
            return basis.cast();
        };
        basis = trimmed.basis_curve();
    }
}

fn sample_count(curve: &Handle<dyn GeomCurve>, range: (f64, f64)) -> Result<i32, Failure> {
    let spline = extracted_bspline(curve);
    let Some(spline) = spline.get() else {
        return Ok(CONTROL_POINTS);
    };
    let mut spans = 0;
    for index in 1..spline.nb_knots() {
        if spline.knot(index + 1)? > range.0 && spline.knot(index)? < range.1 {
            spans += 1;
        }
    }
    let mut count = CONTROL_POINTS;
    while count < spans * (spline.degree() + 1) {
        count += CONTROL_POINTS - 1;
    }
    Ok(count)
}

pub(super) fn uneven_bspline(
    curve: &Handle<dyn GeomCurve>,
    range: (f64, f64),
) -> Result<bool, Failure> {
    let spline = extracted_bspline(curve);
    if spline.is_null() {
        return Ok(false);
    }
    let adaptor = GeomAdaptorCurve::with_curve(curve)?;
    spline_speed_invalid(&spline, &adaptor, range)
}
fn spline_speed_invalid(
    spline: &GeomBSplineCurve,
    adaptor: &GeomAdaptorCurve,
    range: (f64, f64),
) -> Result<bool, Failure> {
    let mut first = range.0;
    let mut speeds = Vec::new();
    let mut minimum = precision::infinite();
    for index in 1..=spline.nb_knots() {
        if first >= range.1 {
            break;
        }
        if spline.knot(index)? <= range.0 {
            continue;
        }
        let last = std_min(range.1, spline.knot(index)?);
        let count = interval_sample_count(spline, index, (first, last))?;
        let (coefficient, smallest) = interval_speed(adaptor, (first, last), count)?;
        minimum = std_min(minimum, smallest);
        if coefficient.abs() > rustycad_tkmath::gp::resolution() {
            speeds.push(coefficient);
        }
        first = last;
    }
    Ok(speed_ratio(&speeds) > 1500. && minimum > precision::confusion())
}

fn interval_sample_count(
    spline: &GeomBSplineCurve,
    index: i32,
    range: (f64, f64),
) -> Result<i32, Failure> {
    if index <= 1 {
        return Ok(CONTROL_POINTS);
    }
    let ratio = (range.1 - range.0) / (spline.knot(index)? - spline.knot(index - 1)?);
    if ratio >= 1e-2 {
        return Ok(CONTROL_POINTS);
    }
    Ok(((ratio / 1e-2 * f64::from(CONTROL_POINTS)) as i32).max(2))
}

fn interval_speed(
    curve: &GeomAdaptorCurve,
    range: (f64, f64),
    count: i32,
) -> Result<(f64, f64), Failure> {
    let step = (range.1 - range.0) / f64::from(count - 1);
    let mut previous = curve.value(range.0)?;
    let mut length = 0.;
    let mut minimum = precision::infinite();
    for index in 1..count {
        let point = curve.value(range.0 + step * f64::from(index))?;
        let distance = point.distance(&previous);
        length += distance;
        previous = point;
        minimum = std_min(minimum, distance / step);
    }
    Ok((length / (range.1 - range.0), minimum))
}

fn speed_ratio(speeds: &[f64]) -> f64 {
    let Some(&first) = speeds.first() else {
        return 0.;
    };
    let (mut minimum, mut maximum) = (first, first);
    for &speed in &speeds[1..] {
        if speed < minimum {
            minimum = speed;
        }
        if maximum < speed {
            maximum = speed;
        }
    }
    maximum / minimum
}
