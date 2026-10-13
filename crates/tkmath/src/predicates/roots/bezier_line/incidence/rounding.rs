// SPDX-License-Identifier: GPL-2.0-only
use super::rational::ExactRatio;
use std::cmp::Ordering;

const MAX_FINITE_BITS: u64 = 0x7fef_ffff_ffff_ffff;

pub(super) fn outward_binary64(value: &ExactRatio) -> Option<[f64; 2]> {
    if value.sign() != Ordering::Less {
        return positive_bounds(value);
    }
    let positive = positive_bounds(&value.negated())?;
    Some([-positive[1], -positive[0]])
}
fn positive_bounds(value: &ExactRatio) -> Option<[f64; 2]> {
    if value > &ExactRatio::from_finite(f64::MAX) {
        return None;
    }
    let floor = floor_bits(value);
    let lower = f64::from_bits(floor);
    if &ExactRatio::from_finite(lower) == value {
        return Some([lower, lower]);
    }
    Some([lower, f64::from_bits(floor + 1)])
}
fn floor_bits(value: &ExactRatio) -> u64 {
    // Positive finite binary64 bit patterns are ordered; every comparison is exact.
    // Unlike a floating quotient, this also certifies underflow and extreme ratios.
    greatest_bits_where(|candidate| ExactRatio::from_finite(candidate) <= *value)
}
pub(super) fn outward_sqrt(value: &ExactRatio) -> Option<f64> {
    if value.sign() == Ordering::Less {
        return None;
    }
    let maximum = ExactRatio::from_finite(f64::MAX);
    if value > &maximum.multiply(&maximum) {
        return None;
    }
    let floor = greatest_bits_where(|candidate| {
        let exact = ExactRatio::from_finite(candidate);
        exact.multiply(&exact) <= *value
    });
    let lower = ExactRatio::from_finite(f64::from_bits(floor));
    let exact = lower.multiply(&lower) == *value;
    Some(f64::from_bits(floor + u64::from(!exact)))
}
fn greatest_bits_where(mut accepted: impl FnMut(f64) -> bool) -> u64 {
    let mut lower = 0;
    let mut upper = MAX_FINITE_BITS;
    while lower < upper {
        let middle = lower + (upper - lower).div_ceil(2);
        if accepted(f64::from_bits(middle)) {
            lower = middle;
        } else {
            upper = middle - 1;
        }
    }
    lower
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn directed_rounding_contains_nonbinary_fraction_without_widening_exact_values() {
        let third = ExactRatio::new(1.into(), 3.into());
        let bounds = outward_binary64(&third).unwrap();
        assert!(ExactRatio::from_finite(bounds[0]) < third);
        assert!(ExactRatio::from_finite(bounds[1]) > third);
        assert_eq!(bounds[1].to_bits(), bounds[0].to_bits() + 1);
        assert_eq!(
            outward_binary64(&third.negated()),
            Some([-bounds[1], -bounds[0]])
        );
        assert_eq!(
            outward_binary64(&ExactRatio::from_finite(0.5)),
            Some([0.5, 0.5])
        );
    }
    #[test]
    fn directed_rounding_distinguishes_subnormal_enclosure_from_overflow() {
        let tiny = ExactRatio::from_finite(f64::from_bits(1));
        let half = tiny.divide(&ExactRatio::from_finite(2.));
        assert_eq!(outward_binary64(&half), Some([0., f64::from_bits(1)]));
        let too_large = ExactRatio::from_finite(f64::MAX).multiply(&ExactRatio::from_finite(2.));
        assert_eq!(outward_binary64(&too_large), None);
    }
    #[test]
    fn square_root_bound_is_the_smallest_binary64_proven_not_below_root() {
        let two = ExactRatio::from_finite(2.);
        let upper = outward_sqrt(&two).unwrap();
        let exact = ExactRatio::from_finite(upper);
        let predecessor = ExactRatio::from_finite(f64::from_bits(upper.to_bits() - 1));
        assert!(exact.multiply(&exact) >= two);
        assert!(predecessor.multiply(&predecessor) < two);
        assert_eq!(outward_sqrt(&ExactRatio::from_finite(4.)), Some(2.));
        assert!(outward_sqrt(&two.negated()).is_none());
    }
}
