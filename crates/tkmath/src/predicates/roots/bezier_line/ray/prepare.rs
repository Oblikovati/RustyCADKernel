// SPDX-License-Identifier: GPL-2.0-only
use super::super::super::RootIsolation;
use super::super::{projection, solve};
use super::{BezierLineQuery, BezierRayError, DyadicBound, IntegerPolynomial};
use num_bigint::BigInt;
pub(in super::super) struct RegularProjection {
    pub normal: Vec<BigInt>,
    pub longitudinal: Vec<BigInt>,
    pub denominator: IntegerPolynomial,
    pub work: u16,
    pub denominator_isolation: RootIsolation,
}
pub(in super::super) fn regular_projection(
    query: BezierLineQuery<'_>,
    bounds: [DyadicBound; 2],
    width: DyadicBound,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<RegularProjection, BezierRayError> {
    regular_projection_captured(query, bounds, width, cancelled, |_, _, _, _| Ok(()))
        .map(|(projected, ())| projected)
}
pub(in super::super) fn regular_projection_captured<T, F: FnMut() -> bool>(
    query: BezierLineQuery<'_>,
    bounds: [DyadicBound; 2],
    width: DyadicBound,
    cancelled: &mut F,
    capture: impl FnOnce(
        &projection::Projection,
        &[BigInt],
        &[DyadicBound; 2],
        &mut F,
    ) -> Result<T, BezierRayError>,
) -> Result<(RegularProjection, T), BezierRayError> {
    regular_projection_accounted(query, bounds, width, cancelled, capture, &mut || {})
}
pub(in super::super) fn regular_projection_accounted<T, F: FnMut() -> bool>(
    query: BezierLineQuery<'_>,
    bounds: [DyadicBound; 2],
    width: DyadicBound,
    cancelled: &mut F,
    capture: impl FnOnce(
        &projection::Projection,
        &[BigInt],
        &[DyadicBound; 2],
        &mut F,
    ) -> Result<T, BezierRayError>,
    completed_split: &mut impl FnMut(),
) -> Result<(RegularProjection, T), BezierRayError> {
    let (projected, longitudinal) =
        projection::project_directed(query, cancelled).map_err(BezierRayError::Line)?;
    let captured = capture(&projected, &longitudinal, &bounds, cancelled)?;
    let (denominator, roots) = solve::regular_denominator_accounted(
        projected.denominator,
        bounds,
        width,
        query.options,
        cancelled,
        completed_split,
    )
    .map_err(BezierRayError::Line)?;
    let regular = completed_projection(projected.numerator, longitudinal, denominator, roots);
    Ok((regular, captured))
}

fn completed_projection(
    normal: Vec<BigInt>,
    longitudinal: Vec<BigInt>,
    denominator: IntegerPolynomial,
    denominator_isolation: RootIsolation,
) -> RegularProjection {
    RegularProjection {
        normal,
        longitudinal,
        denominator,
        work: denominator_isolation.bisections(),
        denominator_isolation,
    }
}
