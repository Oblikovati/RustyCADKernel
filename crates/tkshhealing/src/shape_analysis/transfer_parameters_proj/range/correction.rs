//! Preserve pcurve knot snapping and asymmetric native expansion near range ends.
use super::*;
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dOffsetCurve, Geom2dTrimmedCurve};
pub(super) fn correct_parameter(
    curve: &Handle<dyn Geom2dCurve>,
    parameter: f64,
) -> Result<f64, Failure> {
    if let Some(trimmed) = curve.cast::<Geom2dTrimmedCurve>().get() {
        return correct_parameter(&trimmed.basis_curve(), parameter);
    }
    if let Some(offset) = curve.cast::<Geom2dOffsetCurve>().get() {
        return correct_parameter(&offset.basis_curve(), parameter);
    }
    let spline = curve.cast::<Geom2dBSplineCurve>();
    let Some(spline) = spline.get() else {
        return Ok(parameter);
    };
    for index in spline.first_u_knot_index()?..=spline.last_u_knot_index()? {
        let knot = spline.knot(index)?;
        if (knot - parameter).abs() < precision::p_confusion() {
            return Ok(knot);
        }
    }
    Ok(parameter)
}
pub(super) fn expand_collapsed(parameters: &mut [f64; 2], bounds: (f64, f64)) {
    let tolerance = precision::p_confusion();
    if (parameters[1] - parameters[0]).partial_cmp(&tolerance) != Some(std::cmp::Ordering::Less) {
        return;
    }
    if parameters[0] - bounds.0 < tolerance {
        parameters[1] += 2. * tolerance;
    } else if bounds.1 - parameters[1] < tolerance {
        parameters[0] -= 2. * tolerance;
    } else {
        parameters[0] -= tolerance;
        parameters[1] += tolerance;
    }
}
