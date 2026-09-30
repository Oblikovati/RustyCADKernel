//! End tangents preserve the native derivative fallback and oriented chord order.
use super::*;
use rustycad_tkernel::precision;

fn tiny_tangent(tangent: &Vector2) -> bool {
    tangent.square_magnitude() < precision::p_confusion() * precision::p_confusion()
}

pub(super) fn end_tangent(
    curve: &dyn Geom2dCurve,
    range: (f64, f64),
    at_end: bool,
    fraction: f64,
    orientation: Orientation,
) -> Result<(bool, Point2, Vector2), Failure> {
    if let Some((point, tangent)) = requested_chord(curve, range, at_end, fraction)? {
        return Ok((true, point, tangent));
    }
    let (found, point, tangent) = derivative_tangent(curve, range.0, range.1, at_end)?;
    if !found {
        return Ok((false, point, tangent));
    }
    let tangent = if orientation == Orientation::Reversed {
        tangent.reversed()
    } else {
        tangent
    };
    Ok((true, point, tangent))
}

fn requested_chord(
    curve: &dyn Geom2dCurve,
    range: (f64, f64),
    at_end: bool,
    fraction: f64,
) -> Result<Option<(Point2, Vector2)>, Failure> {
    if fraction <= precision::confusion() {
        return Ok(None);
    }
    let delta = (range.1 - range.0) * fraction;
    if delta.abs() < precision::p_confusion() {
        return Ok(None);
    }
    let sample = chord_tangent(curve, range.0, range.1, delta, at_end)?;
    Ok((!tiny_tangent(&sample.1)).then_some(sample))
}

fn third_derivative_or_chord(
    curve: &dyn Geom2dCurve,
    range: (f64, f64),
    at_end: bool,
) -> Result<(bool, Point2, Vector2), Failure> {
    let parameter = if at_end { range.1 } else { range.0 };
    let (point, _, _, tangent) = curve.d3(parameter)?;
    if !tiny_tangent(&tangent) {
        return Ok((true, point, tangent));
    }
    let opposite = curve.d0(if at_end { range.0 } else { range.1 })?;
    let tangent = Vector2::from(opposite.xy() - point.xy());
    Ok((!tiny_tangent(&tangent), point, tangent))
}

/// The chord tangent of `GetEndTangent2d`: from the end to the point at `delta` inside.
fn chord_tangent(
    c2d: &dyn Geom2dCurve,
    cf: f64,
    cl: f64,
    delta: f64,
    at_end: bool,
) -> Result<(Point2, Vector2), Failure> {
    if at_end {
        let (pnt, ptmp) = (c2d.d0(cl)?, c2d.d0(cl - delta)?);
        return Ok((pnt, Vector2::from(pnt.xy() - ptmp.xy())));
    }
    let (pnt, ptmp) = (c2d.d0(cf)?, c2d.d0(cf + delta)?);
    Ok((pnt, Vector2::from(ptmp.xy() - pnt.xy())))
}

/// The derivative tangent of `GetEndTangent2d`: D1, D2, D3, then the chord between the
/// ends; false (with the last point and tangent) when every one is null.
fn derivative_tangent(
    c2d: &dyn Geom2dCurve,
    cf: f64,
    cl: f64,
    at_end: bool,
) -> Result<(bool, Point2, Vector2), Failure> {
    let parameter = if at_end { cl } else { cf };
    let (point, tangent) = c2d.d1(parameter)?;
    if !tiny_tangent(&tangent) {
        return Ok((true, point, tangent));
    }
    let (point, _, tangent) = c2d.d2(parameter)?;
    if !tiny_tangent(&tangent) {
        return Ok((true, point, tangent));
    }
    third_derivative_or_chord(c2d, (cf, cl), at_end)
}
