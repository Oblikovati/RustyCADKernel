//! Native automatic limits grow geometrically; explicit limits are clamped directly.
use rustycad_tkernel::{precision, standard::Failure};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use std::cmp::Ordering;

pub(super) fn find_limits(curve: &dyn Adaptor3dCurve, limit: f64) -> Result<[f64; 2], Failure> {
    let bounds = [curve.first_parameter()?, curve.last_parameter()?];
    match (
        precision::is_negative_infinite(bounds[0]),
        precision::is_positive_infinite(bounds[1]),
    ) {
        (true, true) => expand_both(curve, limit),
        (true, false) => expand_one(curve, limit, bounds[1], true),
        (false, true) => expand_one(curve, limit, bounds[0], false),
        _ => Ok(bounds),
    }
}
fn expand_both(curve: &dyn Adaptor3dCurve, limit: f64) -> Result<[f64; 2], Failure> {
    let mut delta = 1.;
    loop {
        delta *= 2.;
        let distance = curve.d0(-delta)?.distance(&curve.d0(delta)?);
        if distance.partial_cmp(&limit) != Some(Ordering::Less) {
            return Ok([-delta, delta]);
        }
    }
}
fn expand_one(
    curve: &dyn Adaptor3dCurve,
    limit: f64,
    anchor: f64,
    first: bool,
) -> Result<[f64; 2], Failure> {
    // OCCT evaluates the finite endpoint once, before growing the infinite endpoint.
    let fixed = curve.d0(anchor)?;
    let mut delta = 1.;
    loop {
        delta *= 2.;
        let parameter = anchor + if first { -delta } else { delta };
        let distance = curve.d0(parameter)?.distance(&fixed);
        if distance.partial_cmp(&limit) != Some(Ordering::Less) {
            return Ok(if first {
                [parameter, anchor]
            } else {
                [anchor, parameter]
            });
        }
    }
}
pub(super) fn explicit_limits(mut bounds: [f64; 2], limit: f64) -> [f64; 2] {
    if precision::is_negative_infinite(bounds[0]) {
        bounds[0] = -limit;
    }
    if precision::is_positive_infinite(bounds[1]) {
        bounds[1] = limit;
    }
    bounds
}
