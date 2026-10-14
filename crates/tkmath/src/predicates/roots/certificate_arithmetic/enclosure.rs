// SPDX-License-Identifier: GPL-2.0-only
use super::rational::ExactRatio;
use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub(in crate::predicates::roots) struct RatioInterval {
    pub lower: ExactRatio,
    pub upper: ExactRatio,
}
impl RatioInterval {
    pub(in crate::predicates::roots) fn point(value: ExactRatio) -> Self {
        Self {
            lower: value.clone(),
            upper: value,
        }
    }
    pub(in crate::predicates::roots) fn add(&self, other: &Self) -> Self {
        Self {
            lower: self.lower.add(&other.lower),
            upper: self.upper.add(&other.upper),
        }
    }
    pub(in crate::predicates::roots) fn multiply(&self, other: &Self) -> Self {
        let products = [
            self.lower.multiply(&other.lower),
            self.lower.multiply(&other.upper),
            self.upper.multiply(&other.lower),
            self.upper.multiply(&other.upper),
        ];
        Self {
            lower: products.iter().min().unwrap().clone(),
            upper: products.iter().max().unwrap().clone(),
        }
    }
    pub(in crate::predicates::roots) fn reciprocal(&self) -> Option<Self> {
        if self.lower.sign() != self.upper.sign() || self.lower.sign() == Ordering::Equal {
            return None;
        }
        let one = ExactRatio::from_finite(1.);
        Some(Self {
            lower: one.divide(&self.upper),
            upper: one.divide(&self.lower),
        })
    }
}
pub(in crate::predicates::roots) fn bernstein_enclosure(
    coefficients: &[ExactRatio],
    parameter: &RatioInterval,
    cancelled: &mut impl FnMut() -> bool,
) -> Option<RatioInterval> {
    let mut level: Vec<RatioInterval> = coefficients
        .iter()
        .cloned()
        .map(RatioInterval::point)
        .collect();
    let complement = complement(parameter);
    while level.len() > 1 {
        if cancelled() {
            return None;
        }
        level = next_level(&level, parameter, &complement);
    }
    level.pop()
}
fn complement(parameter: &RatioInterval) -> RatioInterval {
    let one = ExactRatio::from_finite(1.);
    RatioInterval {
        lower: one.subtract(&parameter.upper),
        upper: one.subtract(&parameter.lower),
    }
}
fn next_level(
    level: &[RatioInterval],
    parameter: &RatioInterval,
    complement: &RatioInterval,
) -> Vec<RatioInterval> {
    level
        .windows(2)
        .map(|pair| {
            let left = pair[0].multiply(complement);
            let right = pair[1].multiply(parameter);
            left.add(&right)
        })
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    fn interval(lower: f64, upper: f64) -> RatioInterval {
        RatioInterval {
            lower: ExactRatio::from_finite(lower),
            upper: ExactRatio::from_finite(upper),
        }
    }
    #[test]
    fn arithmetic_bounds_all_endpoint_products_and_refuses_zero_denominator() {
        let product = interval(-2., 3.).multiply(&interval(-4., 5.));
        assert_eq!(product.lower, ExactRatio::from_finite(-12.));
        assert_eq!(product.upper, ExactRatio::from_finite(15.));
        assert!(interval(-1., 2.).reciprocal().is_none());
        assert!(interval(0., 2.).reciprocal().is_none());
        assert_eq!(
            interval(-4., -2.).reciprocal().unwrap().lower,
            ExactRatio::from_finite(-0.5)
        );
    }
    #[test]
    fn bernstein_interval_is_conservative_inside_and_outside_unit_domain() {
        let coefficients: Vec<ExactRatio> = [1., -2., 1.]
            .into_iter()
            .map(ExactRatio::from_finite)
            .collect();
        for (lower, upper) in [(0., 1.), (-2., -1.), (1., 2.), (0.2, 0.3)] {
            let enclosure =
                bernstein_enclosure(&coefficients, &interval(lower, upper), &mut || false).unwrap();
            for parameter in [lower, upper, (lower + upper) / 2.] {
                let exact = bernstein_enclosure(
                    &coefficients,
                    &interval(parameter, parameter),
                    &mut || false,
                )
                .unwrap();
                assert!(enclosure.lower <= exact.lower && enclosure.upper >= exact.upper);
            }
        }
        assert!(bernstein_enclosure(&coefficients, &interval(0., 1.), &mut || true).is_none());
    }
}
