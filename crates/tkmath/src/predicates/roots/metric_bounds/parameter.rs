// SPDX-License-Identifier: GPL-2.0-only
//! Native endpoints remain separate exact operands; never evaluate a rounded b-a.
use super::super::{DyadicBound, error::poll};
use super::arithmetic::product;
use super::coefficients::{HomogeneousCurve, scalar};
use super::{MetricCurve, ResidualError};
use num_bigint::BigInt;

/// Parameter transport into the stored curve's native basis.
/// Example: `MetricParameterMap::Endpoints([2., 5.])` means exactly 2+(5-2)*u.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MetricParameterMap {
    /// The declared query parameter is already the native parameter.
    Identity,
    /// Endpoints at u=0 and u=1, preserved as two independent binary64 operands.
    Endpoints([f64; 2]),
}
impl MetricParameterMap {
    /// Lift an exact bracket without division or binary64 rounding.
    /// Example: `map.image_bounds(&root_bounds)?` yields native parameter evidence.
    pub fn image_bounds(
        self,
        bounds: &[DyadicBound; 2],
    ) -> Result<[DyadicBound; 2], ResidualError> {
        if bounds[0] > bounds[1] {
            return Err(ResidualError::ParameterBracket {
                supplied: bounds.clone(),
            });
        }
        if matches!(self, Self::Identity) {
            return Ok(bounds.clone());
        }
        let [a, b] = self.exact_coefficients()?;
        let first = image(&a, &b, &bounds[0]);
        let last = image(&a, &b, &bounds[1]);
        Ok(if first <= last {
            [first, last]
        } else {
            [last, first]
        })
    }
    pub(in crate::predicates::roots) fn exact_coefficients(
        self,
    ) -> Result<[BigInt; 2], ResidualError> {
        let endpoints = match self {
            Self::Identity => [0., 1.],
            Self::Endpoints(e) => e,
        };
        let a = scalar(endpoints[0], "parameter endpoint", 0)?;
        let b = scalar(endpoints[1], "parameter endpoint", 1)?;
        Ok([a.clone(), b - a])
    }
}
fn image(a: &BigInt, span: &BigInt, u: &DyadicBound) -> DyadicBound {
    let (numerator, denominator) = u.ratio();
    let exponent = -1074 - (denominator.bits() as i32 - 1);
    DyadicBound::new(a * denominator + span * numerator, exponent)
}
pub(super) fn substitute(
    curve: HomogeneousCurve,
    map: MetricParameterMap,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<HomogeneousCurve, ResidualError> {
    if matches!(map, MetricParameterMap::Identity) {
        return Ok(curve);
    }
    let linear = map.exact_coefficients()?;
    let degree = curve.iter().map(Vec::len).max().unwrap_or(1) - 1;
    let mut result: HomogeneousCurve = std::array::from_fn(|_| Vec::new());
    for (target, polynomial) in result.iter_mut().zip(curve) {
        *target = compose(&polynomial, &linear, degree, cancelled)?;
    }
    Ok(result)
}
fn compose(
    coefficients: &[BigInt],
    linear: &[BigInt; 2],
    degree: usize,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<Vec<BigInt>, ResidualError> {
    let mut result = vec![BigInt::from(0)];
    let mut power = vec![BigInt::from(1)];
    for (index, coefficient) in coefficients.iter().enumerate() {
        poll(cancelled)?;
        result.resize(power.len(), BigInt::from(0));
        let scaled = coefficient << (1074 * (degree - index));
        for (target, factor) in result.iter_mut().zip(&power) {
            *target += &scaled * factor;
        }
        power = product(&power, linear, cancelled)?;
    }
    Ok(result)
}
pub(super) fn admit_map<const DIM: usize>(
    curve: MetricCurve<'_, DIM>,
    map: MetricParameterMap,
    bounds: &[DyadicBound; 2],
) -> Result<(), ResidualError> {
    let native = map.image_bounds(bounds)?;
    if matches!(curve, MetricCurve::Bezier(_))
        && (native[0].compare_f64(0.).unwrap().is_lt()
            || native[1].compare_f64(1.).unwrap().is_gt())
    {
        return Err(ResidualError::ParameterDomain { supplied: native });
    }
    Ok(())
}
