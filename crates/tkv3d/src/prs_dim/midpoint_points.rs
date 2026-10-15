//! `PrsDim_MidPointRelation` attach points: the projection of the mid point on a line,
//! segment, circle or ellipse with the ends of the mark segment around it.
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elclib::{self, ElementaryCurve};
use rustycad_tkmath::gp::{Direction3, Line3, Point3, Vector3};
use std::f64::consts::PI;

/// An attach point and the two ends of its mark segment.
pub type PrsDimMidPointAttach = (Point3, Point3, Point3);

/// `ComputePointsOnLine(line, first)`: the mid point projected on an infinite line, the
/// marks a tenth of the distance (10 on the line) on both sides.
/// Example: `points_on_line(&line, &mid)?`.
pub fn points_on_line(line: &Line3, mid: &Point3) -> Result<PrsDimMidPointAttach, Failure> {
    let parameter = elclib::parameter(line, *mid)?;
    let attach = elclib::value(parameter, line);
    let mut distance = attach.distance(mid) / 10.0;
    if distance < precision::confusion() {
        distance = 10.0;
    }
    Ok((
        attach,
        elclib::value(parameter + distance, line),
        elclib::value(parameter - distance, line),
    ))
}

/// `ComputePointsOnLine(p1, p2, first)`: the mid point projected on a segment, the marks
/// toward the ends (from the nearest end when outside).
/// Example: `points_on_segment(&a, &b, &mid)?`.
pub fn points_on_segment(
    first: &Point3,
    second: &Point3,
    mid: &Point3,
) -> Result<PrsDimMidPointAttach, Failure> {
    let line = Line3::new(
        *first,
        Direction3::from_vector(Vector3::between(*first, *second))?,
    );
    let first_par = elclib::parameter(&line, *first)?;
    let second_par = elclib::parameter(&line, *second)?;
    let mid_par = elclib::parameter(&line, *mid)?;
    let projected = elclib::value(mid_par, &line);
    let distance = mid.distance(&projected);
    let length = first.distance(second);
    let mut segment = distance.min(length) * 0.75;
    if distance < precision::confusion() {
        segment = length * 0.75;
    }
    let toward = |to: &Point3, from: &Point3, size: f64| -> Result<Point3, Failure> {
        Ok(projected.translated(Vector3::between(*from, *to).normalized()? * size))
    };
    let (p1, p2) = if mid_par <= first_par {
        (toward(first, second, segment)?, *first)
    } else if mid_par >= second_par {
        (*second, toward(second, first, segment)?)
    } else {
        let d1 = projected.distance(first);
        let d2 = projected.distance(second);
        (
            toward(first, &projected, distance.min(d1) * 0.75)?,
            toward(second, &projected, distance.min(d2) * 0.75)?,
        )
    };
    Ok((projected, p1, p2))
}

/// `ComputePointsOnCirc` / `ComputePointsOnElips`: the mid point parameter on a closed curve
/// (moved off the center), the marks a fifth of pi around it within the arc, or at the
/// nearest arc end when outside.
/// Example: `points_on_conic(&circle, center, &mid, &a, &b)?`.
pub fn points_on_conic<C: ElementaryCurve<Point = Point3>>(
    curve: &C,
    center: Point3,
    mid: &Point3,
    first: &Point3,
    second: &Point3,
) -> Result<PrsDimMidPointAttach, Failure> {
    let confusion = precision::confusion();
    let mut current = *mid;
    if center.distance(&current) <= confusion {
        let shift = Vector3::between(center, *first).normalized()?;
        current = current.translated(shift * 1e-5);
    }
    let mut parameter = elclib::parameter(curve, current)?;
    let rad = PI / 5.0;
    let (first_mark, second_mark);
    if first.is_equal(second, confusion) {
        first_mark = parameter - rad;
        second_mark = parameter + rad;
    } else {
        let start = elclib::parameter(curve, *first)?;
        let end = elclib::parameter(curve, *second)?;
        let mut end_m = end;
        let mut delta = end_m - start;
        if delta < 0.0 {
            delta += 2.0 * PI;
            end_m += 2.0 * PI;
        }
        end_m -= start;
        let middle_out = end_m / 2.0 + PI;
        let mut relative = parameter;
        let outside = |relative: f64, parameter: &mut f64| -> (f64, f64) {
            let segment = rad.min(delta * 0.75);
            if relative > middle_out {
                *parameter = start;
                (start, start + segment)
            } else {
                *parameter = end;
                (end - segment, end)
            }
        };
        let on_arc = |dp1: f64, dp2: f64, parameter: f64| -> (f64, f64) {
            (
                parameter - rad.min(dp1 * 0.75),
                parameter + rad.min(dp2 * 0.75),
            )
        };
        let marks = if relative < start {
            relative = relative + 2.0 * PI - start;
            if relative > end_m {
                outside(relative, &mut parameter)
            } else {
                on_arc(relative - start, end_m - relative, parameter)
            }
        } else if relative > start + delta {
            relative -= start;
            outside(relative, &mut parameter)
        } else {
            on_arc(relative - start, end - relative, parameter)
        };
        (first_mark, second_mark) = marks;
    }
    Ok((
        elclib::value(parameter, curve),
        elclib::value(first_mark, curve),
        elclib::value(second_mark, curve),
    ))
}
