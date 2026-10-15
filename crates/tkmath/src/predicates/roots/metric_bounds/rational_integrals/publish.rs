// SPDX-License-Identifier: GPL-2.0-only
//! Admission, error-budget division and final outward rounding of the exact bounds.
use super::super::super::certificate_arithmetic::{ExactRatio, rounding::outward_binary64};
use super::super::{OrientedMetricCurve, PolynomialIntegralOperation};
use super::subinterval::ContributionBounds;
use super::{
    CertifiedRationalMeasure, RationalIntegralEvidence, RationalIntegralFailure,
    RationalIntegralOptions,
};
use std::cmp::Ordering;

pub(super) fn admit_boundary(
    curves: &[OrientedMetricCurve<'_>],
    options: RationalIntegralOptions,
) -> Result<(), RationalIntegralFailure> {
    admit_options(options)?;
    if curves.is_empty() {
        return Err(RationalIntegralFailure::EmptyBoundary);
    }
    if curves.len() > usize::from(options.max_curves) {
        return Err(RationalIntegralFailure::CurveCount {
            actual: curves.len(),
            limit: options.max_curves,
        });
    }
    Ok(())
}
fn admit_options(options: RationalIntegralOptions) -> Result<(), RationalIntegralFailure> {
    if options.behavior_version != 1
        || options.max_degree > 16
        || options.max_curves == 0
        || options.max_curves > 64
        || options.max_subintervals == 0
        || options.max_subintervals > 1 << 16
        || !options.absolute_error_limit.is_finite()
        || options.absolute_error_limit < 0.
    {
        return Err(RationalIntegralFailure::Policy(options));
    }
    Ok(())
}
/// Per-curve width share `cap / (count * abs(factor))`; a zero factor needs only `cap / count`.
pub(super) fn width_share(
    options: RationalIntegralOptions,
    count: usize,
    factor: &ExactRatio,
) -> ExactRatio {
    let cap = ExactRatio::from_finite(options.absolute_error_limit);
    let curves = ExactRatio::new((count.max(1) as u64).into(), 1.into());
    let magnitude = match factor.sign() {
        Ordering::Less => factor.negated(),
        Ordering::Equal => ExactRatio::from_finite(1.),
        Ordering::Greater => factor.clone(),
    };
    cap.divide(&curves.multiply(&magnitude))
}
/// Reverse traversal negates the integral, so the bounds swap.
pub(super) fn reversed_bounds(bounds: ContributionBounds, reversed: bool) -> ContributionBounds {
    if !reversed {
        return bounds;
    }
    ContributionBounds {
        lower: bounds.upper.negated(),
        upper: bounds.lower.negated(),
    }
}
/// Exact running sum of every curve's bounds and the integration work spent.
pub(super) struct BoundsSum {
    bounds: ContributionBounds,
    subintervals: u64,
}
impl BoundsSum {
    pub(super) fn zero() -> Self {
        Self {
            bounds: ContributionBounds {
                lower: ExactRatio::from_finite(0.),
                upper: ExactRatio::from_finite(0.),
            },
            subintervals: 0,
        }
    }
    pub(super) fn accumulate(&mut self, part: ContributionBounds, subintervals: u64) {
        self.bounds.lower = self.bounds.lower.add(&part.lower);
        self.bounds.upper = self.bounds.upper.add(&part.upper);
        self.subintervals += subintervals;
    }
    /// Multiply by an exact factor, swapping the bounds when it is negative.
    pub(super) fn scaled(self, factor: &ExactRatio) -> Self {
        let lower = self.bounds.lower.multiply(factor);
        let upper = self.bounds.upper.multiply(factor);
        let (lower, upper) = if lower <= upper {
            (lower, upper)
        } else {
            (upper, lower)
        };
        Self {
            bounds: ContributionBounds { lower, upper },
            subintervals: self.subintervals,
        }
    }
}
pub(super) fn rounded_measure(
    sum: BoundsSum,
    options: RationalIntegralOptions,
    operation: PolynomialIntegralOperation,
) -> Result<CertifiedRationalMeasure, RationalIntegralFailure> {
    let ContributionBounds { lower, upper } = &sum.bounds;
    let middle = lower.add(upper).multiply(&ExactRatio::from_finite(0.5));
    let representative = finite_enclosure(&middle)?[0];
    let exact_representative = ExactRatio::from_finite(representative);
    let error = upper
        .subtract(&exact_representative)
        .max(exact_representative.subtract(lower));
    let absolute_error_bound = finite_enclosure(&error)?[1];
    if absolute_error_bound > options.absolute_error_limit {
        return Err(RationalIntegralFailure::ErrorLimit {
            requested: options.absolute_error_limit,
            achieved: absolute_error_bound,
        });
    }
    Ok(CertifiedRationalMeasure {
        enclosure: [finite_enclosure(lower)?[0], finite_enclosure(upper)?[1]],
        representative,
        absolute_error_bound,
        options,
        operation,
        evidence: evidence(sum.subintervals),
        subintervals: sum.subintervals,
    })
}
fn evidence(subintervals: u64) -> RationalIntegralEvidence {
    if subintervals == 0 {
        return RationalIntegralEvidence::ExactRational;
    }
    RationalIntegralEvidence::ErrorBoundedIntegration
}
fn finite_enclosure(exact: &ExactRatio) -> Result<[f64; 2], RationalIntegralFailure> {
    outward_binary64(exact).ok_or_else(|| RationalIntegralFailure::Unrepresentable {
        exact_value: format!("{exact:?}"),
    })
}
