// SPDX-License-Identifier: GPL-2.0-only
//! One occurrence's half Green integral: exact for constant denominators, otherwise an
//! adaptive bisection of ADR 0066 subinterval enclosures until the width share is met.
use super::super::super::{certificate_arithmetic::ExactRatio, error::poll};
use super::super::polynomial_integrals::arithmetic::{
    green_numerator, integrate_unit, native_plane,
};
use super::super::{MetricCurve, arithmetic::product, coefficients};
use super::subinterval::{ContributionBounds, DyadicSpan, enclose_span};
use super::{RationalIntegralFailure, RationalIntegralOptions};
use num_bigint::{BigInt, Sign};

/// Exact bounds of one curve's half Green integral and the subintervals examined.
pub(super) struct CurveIntegral {
    pub bounds: ContributionBounds,
    pub subintervals: u64,
}

/// Integrate one admitted curve; `width_share` bounds `upper - lower` over all of `[0,1]`.
pub(super) fn curve_integral(
    source: MetricCurve<'_, 2>,
    options: RationalIntegralOptions,
    width_share: &ExactRatio,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<CurveIntegral, RationalIntegralFailure> {
    poll(cancelled)?;
    admit_degree(source, options)?;
    let curve = coefficients::embedded(source, native_plane(), "integral curve", cancelled)?;
    let numerator = green_numerator(&curve, cancelled)?;
    let weight = trimmed(&curve[3]);
    if weight.len() == 1 {
        let denominator = BigInt::from(2) * &weight[0] * &weight[0];
        let exact = integrate_unit(&numerator, denominator, cancelled)?;
        return Ok(exact_curve(exact));
    }
    let squared = product(&weight, &weight, cancelled)?;
    let bisection = Bisection {
        numerator,
        denominator: squared,
        width_share,
        limit: options.max_subintervals,
    };
    bisection.integrate(cancelled)
}
fn exact_curve(value: ExactRatio) -> CurveIntegral {
    CurveIntegral {
        bounds: ContributionBounds {
            lower: value.clone(),
            upper: value,
        },
        subintervals: 0,
    }
}
fn admit_degree(
    curve: MetricCurve<'_, 2>,
    options: RationalIntegralOptions,
) -> Result<(), RationalIntegralFailure> {
    let degree = match curve {
        MetricCurve::Bezier(curve) => curve.poles.len().saturating_sub(1),
        MetricCurve::Line { .. } => 1,
    };
    if degree > usize::from(options.max_degree) {
        return Err(RationalIntegralFailure::Degree {
            actual: degree,
            limit: options.max_degree,
        });
    }
    Ok(())
}
/// Drop vanishing leading terms: equal weights give a constant, exactly integrable denominator.
fn trimmed(coefficients: &[BigInt]) -> Vec<BigInt> {
    let length = coefficients
        .iter()
        .rposition(|c| c.sign() != Sign::NoSign)
        .map_or(1, |last| last + 1);
    coefficients[..length].to_vec()
}

struct Bisection<'a> {
    numerator: Vec<BigInt>,
    denominator: Vec<BigInt>,
    width_share: &'a ExactRatio,
    limit: u32,
}
impl Bisection<'_> {
    /// Depth-first, left before right; the exact sum makes the order value-neutral.
    fn integrate(
        &self,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<CurveIntegral, RationalIntegralFailure> {
        let mut pending = vec![DyadicSpan::unit()];
        let mut total = exact_curve(ExactRatio::from_finite(0.));
        let mut covered = ExactRatio::from_finite(0.);
        while let Some(span) = pending.pop() {
            self.charge(&mut total, &covered)?;
            let bounds = enclose_span(&self.numerator, &self.denominator, &span, cancelled)?;
            if self.accepts(&bounds, &span) {
                accumulate(&mut total.bounds, &bounds);
                covered = covered.add(&span.length());
                continue;
            }
            let [left, right] = span.halves();
            pending.push(right);
            pending.push(left);
        }
        Ok(total)
    }
    fn charge(
        &self,
        total: &mut CurveIntegral,
        covered: &ExactRatio,
    ) -> Result<(), RationalIntegralFailure> {
        if total.subintervals >= u64::from(self.limit) {
            return Err(RationalIntegralFailure::WorkLimit {
                limit: self.limit,
                certified_length: downward(covered),
            });
        }
        total.subintervals += 1;
        Ok(())
    }
    /// Accept when the width is within the span's proportional share of the curve budget.
    fn accepts(&self, bounds: &ContributionBounds, span: &DyadicSpan) -> bool {
        let width = bounds.upper.subtract(&bounds.lower);
        width <= self.width_share.multiply(&span.length())
    }
}
fn accumulate(total: &mut ContributionBounds, part: &ContributionBounds) {
    total.lower = total.lower.add(&part.lower);
    total.upper = total.upper.add(&part.upper);
}
fn downward(value: &ExactRatio) -> f64 {
    super::super::super::certificate_arithmetic::rounding::outward_binary64(value)
        .map_or(0., |bounds| bounds[0])
}
