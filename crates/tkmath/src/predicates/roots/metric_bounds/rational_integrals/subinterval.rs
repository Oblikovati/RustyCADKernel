// SPDX-License-Identifier: GPL-2.0-only
//! ADR 0066 Taylor-model enclosure of `1/2 * integral_a^b P/Q dt` with `Q > 0`.
//!
//! On the dyadic span `[j, j+1] / 2^l` put `K = 2^(l+1)` and `t = (2j + 1 + v) / K`.
//! Then `P/Q = K^(m-n) p/q` with the integer polynomials `p = K^n P(t)`, `q = K^m Q(t)`
//! and `1/2 integral P/Q dt = K^(m-n) / (2K) * integral_{-1}^{1} p/q dv`.
//! A binary64 series only *chooses* the dyadic approximant `S`; the remainder
//! `R = p - S q` is exact, so `p/q = S + R/q`. `S` integrates exactly. With `v = 2u - 1`,
//! `R/q` is a convex combination of its Bernstein ratios `r_i/q_i` when every `q_i > 0`,
//! so `integral_{-1}^{1} R/q dv` lies in `2 [min r_i/q_i, max r_i/q_i]`.
use super::super::super::{
    RootIsolationError,
    certificate_arithmetic::{ExactRatio, convolution::raw_coefficient_product},
};
use super::integer_polynomial::{
    approximate, decompose, scaled_bernstein, shifted_difference, shifted_scaled,
    unit_from_symmetric,
};
use super::{RATIONAL_TAYLOR_ORDER, RationalIntegralFailure};
use num_bigint::{BigInt, Sign};

/// Exact dyadic parameter span `[index, index + 1] / 2^level` inside `[0,1]`.
#[derive(Clone, Debug)]
pub(super) struct DyadicSpan {
    pub index: BigInt,
    pub level: u32,
}
/// Exact lower and upper bounds of one contribution.
#[derive(Clone, Debug)]
pub(super) struct ContributionBounds {
    pub lower: ExactRatio,
    pub upper: ExactRatio,
}
impl DyadicSpan {
    pub(super) fn unit() -> Self {
        Self {
            index: BigInt::from(0),
            level: 0,
        }
    }
    pub(super) fn length(&self) -> ExactRatio {
        power_of_two(-i64::from(self.level))
    }
    pub(super) fn halves(&self) -> [Self; 2] {
        let left = &self.index * 2;
        let right = &left + 1;
        let level = self.level + 1;
        [
            Self { index: left, level },
            Self {
                index: right,
                level,
            },
        ]
    }
}
pub(super) fn power_of_two(exponent: i64) -> ExactRatio {
    let magnitude = BigInt::from(1) << exponent.unsigned_abs();
    if exponent >= 0 {
        return ExactRatio::new(magnitude, BigInt::from(1));
    }
    ExactRatio::new(BigInt::from(1), magnitude)
}

/// Enclose `1/2 * integral_span P/Q dt` for integer monomial `P` and `Q`.
pub(super) fn enclose_span(
    numerator: &[BigInt],
    denominator: &[BigInt],
    span: &DyadicSpan,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<ContributionBounds, RationalIntegralFailure> {
    let scale = BigInt::from(1) << (span.level as usize + 1);
    let offset = &span.index * 2 + 1;
    let p = shifted_scaled(numerator, &offset, &scale, cancelled)?;
    let q = shifted_scaled(denominator, &offset, &scale, cancelled)?;
    if q.first()
        .is_none_or(|constant| constant.sign() != Sign::Plus)
    {
        return Err(denominator_sign(span));
    }
    let model = TaylorModel::new(&p, &q, cancelled)?;
    let [low, high] = model.remainder_bounds(&q, span, cancelled)?;
    let degrees = denominator.len() as i64 - numerator.len() as i64;
    let level = i64::from(span.level) + 1;
    let factor = power_of_two(level * degrees - level - 1);
    let two = ExactRatio::from_finite(2.);
    Ok(ContributionBounds {
        lower: factor.multiply(&model.series_integral.add(&two.multiply(&low))),
        upper: factor.multiply(&model.series_integral.add(&two.multiply(&high))),
    })
}

/// `p = S q + R` with `S = series * 2^series_exponent` and `R = remainder * 2^remainder_exponent`.
struct TaylorModel {
    remainder: Vec<BigInt>,
    remainder_exponent: i64,
    series_integral: ExactRatio,
}
impl TaylorModel {
    fn new(
        p: &[BigInt],
        q: &[BigInt],
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<Self, RootIsolationError> {
        let (series, exponent) = approximant(p, q);
        let product = raw_coefficient_product(&series, q, cancelled)?;
        let (remainder, remainder_exponent) = if exponent >= 0 {
            (shifted_difference(p, &product, exponent as usize), 0)
        } else {
            let lifted: Vec<BigInt> = p.iter().map(|c| c << exponent.unsigned_abs()).collect();
            (shifted_difference(&lifted, &product, 0), exponent)
        };
        Ok(Self {
            remainder,
            remainder_exponent,
            series_integral: symmetric_integral(&series).multiply(&power_of_two(exponent)),
        })
    }
    /// `[min, max]` of `R(2u-1) / q(2u-1)` Bernstein ratios at a common degree, exactly.
    fn remainder_bounds(
        &self,
        q: &[BigInt],
        span: &DyadicSpan,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<[ExactRatio; 2], RationalIntegralFailure> {
        let top = unit_from_symmetric(&self.remainder, cancelled)?;
        let bottom = unit_from_symmetric(q, cancelled)?;
        let degree = top.len().max(bottom.len()).saturating_sub(1);
        let top = scaled_bernstein(&top, degree, cancelled)?;
        let bottom = scaled_bernstein(&bottom, degree, cancelled)?;
        if bottom.iter().any(|value| value.sign() != Sign::Plus) {
            return Err(denominator_sign(span));
        }
        let scale = power_of_two(self.remainder_exponent);
        let [low, high] = extreme_ratios(&top, &bottom);
        Ok([low.multiply(&scale), high.multiply(&scale)])
    }
}
/// Truncated binary64 series of `p/q`, returned as exact integers times `2^exponent`.
/// Nonfinite or vanishing terms end the series; any approximant keeps `R` exact.
fn approximant(p: &[BigInt], q: &[BigInt]) -> (Vec<BigInt>, i64) {
    let p_shift = max_bits(p);
    let q_shift = max_bits(q);
    let p_float: Vec<f64> = p.iter().map(|c| approximate(c, p_shift)).collect();
    let q_float: Vec<f64> = q.iter().map(|c| approximate(c, q_shift)).collect();
    let series = float_series(&p_float, &q_float);
    let parts: Vec<(i64, i64)> = series.iter().map(|value| decompose(*value)).collect();
    let lowest = parts.iter().filter(|(m, _)| *m != 0).map(|(_, e)| *e).min();
    let Some(lowest) = lowest else {
        return (Vec::new(), 0);
    };
    let integers = parts
        .iter()
        .map(|(m, e)| BigInt::from(*m) << (e - lowest) as usize);
    (integers.collect(), lowest + p_shift as i64 - q_shift as i64)
}
fn max_bits(polynomial: &[BigInt]) -> u64 {
    polynomial.iter().map(BigInt::bits).max().unwrap_or(0)
}
fn float_series(p: &[f64], q: &[f64]) -> Vec<f64> {
    let mut series: Vec<f64> = Vec::with_capacity(RATIONAL_TAYLOR_ORDER + 1);
    for power in 0..=RATIONAL_TAYLOR_ORDER {
        let mut value = p.get(power).copied().unwrap_or(0.);
        for (shift, term) in q.iter().enumerate().take(power + 1).skip(1) {
            value -= term * series[power - shift];
        }
        let value = value / q[0];
        if !value.is_finite() {
            break;
        }
        series.push(value);
    }
    series
}
/// Exact `integral_{-1}^{1} sum_k s_k v^k dv`; odd powers cancel.
fn symmetric_integral(series: &[BigInt]) -> ExactRatio {
    let even = series.iter().enumerate().step_by(2);
    even.fold(ExactRatio::from_finite(0.), |sum, (power, coefficient)| {
        sum.add(&ExactRatio::new(coefficient * 2, BigInt::from(power + 1)))
    })
}
/// Minimum and maximum of `top_i / bottom_i` for positive `bottom_i`, by cross-multiplication.
fn extreme_ratios(top: &[BigInt], bottom: &[BigInt]) -> [ExactRatio; 2] {
    let compare = |a: &usize, b: &usize| (&top[*a] * &bottom[*b]).cmp(&(&top[*b] * &bottom[*a]));
    let indices = 0..top.len();
    let low = indices.clone().min_by(compare).unwrap_or(0);
    let high = indices.max_by(compare).unwrap_or(0);
    let ratio = |index: usize| ExactRatio::new(top[index].clone(), bottom[index].clone());
    [ratio(low), ratio(high)]
}
fn denominator_sign(span: &DyadicSpan) -> RationalIntegralFailure {
    let width = 2f64.powi(-(span.level.min(1074) as i32));
    let start = approximate(&span.index, 0) * width;
    RationalIntegralFailure::DenominatorSign {
        start,
        end: start + width,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn polynomial_ratio_with_exact_series_has_zero_width() {
        // P/Q = (1 + t)^2 / (1 + t)^2 = 1 on [0,1]; half integral 1/2 exactly.
        let square: Vec<BigInt> = [1, 2, 1].into_iter().map(BigInt::from).collect();
        let bounds = enclose_span(&square, &square, &DyadicSpan::unit(), &mut || false).unwrap();
        assert_eq!(bounds.lower, ExactRatio::from_finite(0.5));
        assert_eq!(bounds.upper, ExactRatio::from_finite(0.5));
    }
    #[test]
    fn halves_tile_the_span_and_cancellation_is_typed() {
        let [left, right] = DyadicSpan::unit().halves();
        assert_eq!(
            left.length().add(&right.length()),
            ExactRatio::from_finite(1.)
        );
        assert_eq!(right.index, BigInt::from(1));
        let one = [BigInt::from(1)];
        let result = enclose_span(&one, &one, &left, &mut || true);
        assert!(matches!(result, Err(RationalIntegralFailure::Metric(_))));
    }
    #[test]
    fn nonpositive_denominator_is_refused_with_its_span() {
        let one = [BigInt::from(1)];
        let negative = [BigInt::from(-1)];
        let [_, right] = DyadicSpan::unit().halves();
        let result = enclose_span(&one, &negative, &right, &mut || false);
        assert!(matches!(
            result,
            Err(RationalIntegralFailure::DenominatorSign {
                start: 0.5,
                end: 1.
            })
        ));
    }
}
