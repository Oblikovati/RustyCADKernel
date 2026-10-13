// SPDX-License-Identifier: GPL-2.0-only
use super::super::super::{DyadicBound, error::poll};
use super::super::{BezierRayError, projection::Projection};
use num_bigint::BigInt;
#[derive(Clone)]
pub(super) struct Endpoint([BigInt; 3]);
impl Endpoint {
    pub fn matches(&self, other: &Self) -> bool {
        &self.0[0] * &other.0[2] == &other.0[0] * &self.0[2]
            && &self.0[1] * &other.0[2] == &other.0[1] * &self.0[2]
    }
}
pub(super) fn capture(
    projection: &Projection,
    longitudinal: &[BigInt],
    bounds: &[DyadicBound; 2],
    cancelled: &mut impl FnMut() -> bool,
) -> Result<[Endpoint; 2], BezierRayError> {
    let controls: Vec<[BigInt; 3]> = projection
        .numerator
        .iter()
        .zip(longitudinal)
        .zip(&projection.denominator)
        .map(|((n, l), d)| [n.clone(), l.clone(), d.clone()])
        .collect();
    let lower = evaluate(controls.clone(), &bounds[0], cancelled)?;
    let upper = if bounds[0] == bounds[1] {
        lower.clone()
    } else {
        evaluate(controls, &bounds[1], cancelled)?
    };
    Ok([lower, upper])
}
fn evaluate(
    mut controls: Vec<[BigInt; 3]>,
    parameter: &DyadicBound,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<Endpoint, BezierRayError> {
    let (numerator, denominator) = parameter.ratio();
    let complement = &denominator - &numerator;
    for length in (1..controls.len()).rev() {
        for index in 0..length {
            poll(cancelled).map_err(BezierRayError::Endpoint)?;
            controls[index] = std::array::from_fn(|axis| {
                &controls[index][axis] * &complement + &controls[index + 1][axis] * &numerator
            });
        }
    }
    Ok(Endpoint(controls.remove(0)))
}
