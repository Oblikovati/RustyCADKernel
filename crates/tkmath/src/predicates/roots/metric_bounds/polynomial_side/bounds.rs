// SPDX-License-Identifier: GPL-2.0-only
use super::super::super::certificate_arithmetic::rounding::{
    outward_binary64, outward_sqrt_bounds,
};
use super::arithmetic::{add, scale, zero_vector};
use super::{ExactRatio, PolynomialSideFailure as Fault, Vector, checkpoint};

#[derive(Clone)]
pub(super) struct IntegralBounds {
    pub lower: ExactRatio,
    pub upper: ExactRatio,
}
impl IntegralBounds {
    pub fn add(&self, other: &Self) -> Self {
        Self {
            lower: self.lower.add(&other.lower),
            upper: self.upper.add(&other.upper),
        }
    }
    pub fn replace(&self, old: &Self, first: &Self, second: &Self) -> Self {
        let children = first.add(second);
        Self {
            lower: self.lower.subtract(&old.lower).add(&children.lower),
            upper: self.upper.subtract(&old.upper).add(&children.upper),
        }
    }
    pub fn intersect(&self, other: &Self) -> Self {
        Self {
            lower: self.lower.clone().max(other.lower.clone()),
            upper: self.upper.clone().min(other.upper.clone()),
        }
    }
    pub fn publish(&self) -> Result<([f64; 2], f64), Fault> {
        let lower = finite_bounds(&self.lower, "integral lower bound")?[0];
        let upper = finite_bounds(&self.upper, "integral upper bound")?[1];
        let error = ExactRatio::from_finite(upper).subtract(&ExactRatio::from_finite(lower));
        Ok((
            [lower, upper],
            finite_bounds(&error, "absolute area error")?[1],
        ))
    }
}
pub(super) fn interval_bounds(
    controls: &[Vector],
    width: &ExactRatio,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<IntegralBounds, Fault> {
    let factor = width.divide(&ExactRatio::new(controls.len().into(), 1.into()));
    let mut integrated = zero_vector();
    let mut upper = ExactRatio::from_finite(0.);
    for control in controls {
        checkpoint(cancelled)?;
        // Scale before sqrt: an unweighted norm can overflow while its integral is finite.
        let contribution = scale(control, &factor);
        integrated = add(&integrated, &contribution);
        upper = upper.add(&ExactRatio::from_finite(norm(&contribution)?[1]));
    }
    Ok(IntegralBounds {
        lower: ExactRatio::from_finite(norm(&integrated)?[0]),
        upper,
    })
}
fn norm(vector: &Vector) -> Result<[f64; 2], Fault> {
    let squared = vector
        .iter()
        .fold(ExactRatio::from_finite(0.), |sum, coordinate| {
            sum.add(&coordinate.multiply(coordinate))
        });
    outward_sqrt_bounds(&squared).ok_or_else(|| Fault::Unrepresentable {
        quantity: "squared weighted coefficient norm",
        exact_value: format!("{squared:?}"),
    })
}
fn finite_bounds(value: &ExactRatio, quantity: &'static str) -> Result<[f64; 2], Fault> {
    outward_binary64(value).ok_or_else(|| Fault::Unrepresentable {
        quantity,
        exact_value: format!("{value:?}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scaled_bounds_do_not_overflow_unscaled_norm_or_hide_subnormals() {
        let huge = ExactRatio::from_finite(f64::MAX).multiply(&ExactRatio::from_finite(2.));
        let controls = vec![[
            huge,
            ExactRatio::from_finite(0.),
            ExactRatio::from_finite(0.),
        ]];
        let bounds =
            interval_bounds(&controls, &ExactRatio::from_finite(0.25), &mut || false).unwrap();
        assert_eq!(bounds.publish().unwrap(), ([f64::MAX / 2.; 2], 0.));
        let tiny = vec![[f64::from_bits(1), 0., 0.].map(ExactRatio::from_finite)];
        let bounds = interval_bounds(&tiny, &ExactRatio::from_finite(0.5), &mut || false).unwrap();
        assert_eq!(
            bounds.publish().unwrap(),
            ([0., f64::from_bits(1)], f64::from_bits(1))
        );
    }
}
