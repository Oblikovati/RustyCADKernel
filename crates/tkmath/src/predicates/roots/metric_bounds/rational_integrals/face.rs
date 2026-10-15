// SPDX-License-Identifier: GPL-2.0-only
//! ADR 0068: certified area of a face bounded by one outer loop and its direct holes.
//! For a Jordan loop the Green integral is its orientation times its enclosed area, so a
//! certified integral that excludes zero gives both; the face area is the outer area
//! minus the hole areas. The Jordan, disjointness and nesting facts are hypotheses
//! certified by the caller (the general loop region proof); they are not checked here.
use super::super::super::{certificate_arithmetic::ExactRatio, error::poll};
use super::super::{OrientedMetricCurve, PolynomialIntegralOperation};
use super::publish::{BoundsSum, admit_boundary, reversed_bounds};
use super::subinterval::ContributionBounds;
use super::*;

/// Certify `area(outer) - sum area(hole)`; every loop's integral must exclude zero.
/// Example: `certify_rational_face_area(&outer, &[&hole], Default::default(), || false)?`.
pub fn certify_rational_face_area(
    outer: &[OrientedMetricCurve<'_>],
    holes: &[&[OrientedMetricCurve<'_>]],
    options: RationalIntegralOptions,
    mut cancelled: impl FnMut() -> bool,
) -> Result<CertifiedRationalMeasure, RationalIntegralError> {
    let request = FaceRequest { options };
    poll(&mut cancelled).map_err(|cause| request.failure(None, cause.into()))?;
    let loops: Vec<&[OrientedMetricCurve<'_>]> = std::iter::once(outer)
        .chain(holes.iter().copied())
        .collect();
    for curves in &loops {
        admit_boundary(curves, options).map_err(|cause| request.failure(None, cause))?;
    }
    let total: usize = loops.iter().map(|curves| curves.len()).sum();
    let share = publish::width_share(options, total, &ExactRatio::from_finite(1.));
    let mut face = BoundsSum::zero();
    let mut offset = 0;
    for (index, curves) in loops.iter().enumerate() {
        let (bounds, work) = request.loop_magnitude(curves, offset, &share, &mut cancelled)?;
        face.accumulate(if index == 0 { bounds } else { negated(bounds) }, work);
        offset += curves.len();
    }
    poll(&mut cancelled).map_err(|cause| request.failure(None, cause.into()))?;
    publish::rounded_measure(face, options, PolynomialIntegralOperation::FaceArea)
        .map_err(|cause| request.failure(None, cause))
}
struct FaceRequest {
    options: RationalIntegralOptions,
}
impl FaceRequest {
    fn failure(
        &self,
        curve_index: Option<usize>,
        cause: RationalIntegralFailure,
    ) -> RationalIntegralError {
        RationalIntegralError {
            operation: PolynomialIntegralOperation::FaceArea,
            requested_options: self.options,
            curve_index,
            cause,
        }
    }
    /// Exact enclosure of one loop's enclosed area and the subintervals spent.
    fn loop_magnitude(
        &self,
        curves: &[OrientedMetricCurve<'_>],
        offset: usize,
        share: &ExactRatio,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<(ContributionBounds, u64), RationalIntegralError> {
        let zero = ExactRatio::from_finite(0.);
        let mut sum = ContributionBounds {
            lower: zero.clone(),
            upper: zero.clone(),
        };
        let mut work = 0;
        for (index, occurrence) in curves.iter().enumerate() {
            let part = curve::curve_integral(occurrence.curve, self.options, share, cancelled)
                .map_err(|cause| self.failure(Some(offset + index), cause))?;
            let bounds = reversed_bounds(part.bounds, occurrence.reversed);
            sum = ContributionBounds {
                lower: sum.lower.add(&bounds.lower),
                upper: sum.upper.add(&bounds.upper),
            };
            work += part.subintervals;
        }
        if sum.lower > zero {
            return Ok((sum, work));
        }
        if sum.upper < zero {
            return Ok((negated(sum), work));
        }
        let cause = RationalIntegralFailure::IndeterminateOrientation {
            first_curve: offset,
        };
        Err(self.failure(Some(offset), cause))
    }
}
fn negated(bounds: ContributionBounds) -> ContributionBounds {
    reversed_bounds(bounds, true)
}
