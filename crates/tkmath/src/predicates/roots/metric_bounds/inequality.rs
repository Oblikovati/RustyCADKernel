// SPDX-License-Identifier: GPL-2.0-only
use super::arithmetic::{accumulate, difference, product};
use super::coefficients::{embedded, place, scalar};
use super::{PlanarResidualQuery, ResidualError};
use num_bigint::{BigInt, Sign};
pub(super) fn threshold(
    query: &PlanarResidualQuery<'_>,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<(Vec<BigInt>, bool, BigInt), ResidualError> {
    mapped_threshold(query, [super::MetricParameterMap::Identity; 2], cancelled)
}
pub(super) fn mapped_threshold(
    query: &PlanarResidualQuery<'_>,
    maps: [super::MetricParameterMap; 2],
    cancelled: &mut impl FnMut() -> bool,
) -> Result<(Vec<BigInt>, bool, BigInt), ResidualError> {
    let first = embedded(query.curve, query.curve_embedding, "curve", cancelled)?;
    let second = embedded(query.pcurve, query.plane_embedding, "pcurve", cancelled)?;
    let second = place(second, query.plane_placement)?;
    let first = super::parameter::substitute(first, maps[0], cancelled)?;
    let second = super::parameter::substitute(second, maps[1], cancelled)?;
    let denominator = product(&first[3], &second[3], cancelled)?;
    let residual = residual_square(&first, &second, cancelled)?;
    let zero = residual.iter().all(|n| n.sign() == Sign::NoSign);
    let limit = scalar(query.spatial_limit, "spatial_limit", 0)?;
    let square = &limit * &limit;
    Ok((
        threshold_difference(&denominator, residual, &square, cancelled)?,
        zero,
        square,
    ))
}
fn threshold_difference(
    denominator: &[BigInt],
    mut residual: Vec<BigInt>,
    squared_limit: &BigInt,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<Vec<BigInt>, ResidualError> {
    let mut permitted = product(denominator, denominator, cancelled)?;
    for coefficient in &mut permitted {
        *coefficient *= squared_limit;
    }
    for coefficient in &mut residual {
        *coefficient <<= 2148usize;
    }
    Ok(difference(permitted, residual))
}

fn residual_square(
    first: &super::coefficients::HomogeneousCurve,
    second: &super::coefficients::HomogeneousCurve,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<Vec<BigInt>, ResidualError> {
    let mut residual = vec![BigInt::from(0)];
    for axis in 0..3 {
        let numerator = difference(
            product(&first[axis], &second[3], cancelled)?,
            product(&second[axis], &first[3], cancelled)?,
        );
        accumulate(&mut residual, product(&numerator, &numerator, cancelled)?);
    }
    Ok(residual)
}
