// SPDX-License-Identifier: GPL-2.0-only
//! Exact restriction of one polynomial; no sampled geometry or tessellation is constructed.
use super::arithmetic::{add, scale};
use super::bounds::IntegralBounds;
use super::{ExactRatio, PolynomialSideFailure as Fault, Vector, checkpoint};
use std::cmp::Ordering;

pub(super) struct Interval {
    pub controls: Vec<Vector>,
    pub start: ExactRatio,
    pub width: ExactRatio,
    pub bounds: IntegralBounds,
    pub gap: ExactRatio,
}
impl Ord for Interval {
    fn cmp(&self, other: &Self) -> Ordering {
        self.gap
            .cmp(&other.gap)
            .then_with(|| other.start.cmp(&self.start))
    }
}
impl PartialOrd for Interval {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Heap scheduling must use exactly the same total order for every comparison.
        Some(self.cmp(other))
    }
}
impl PartialEq for Interval {
    fn eq(&self, other: &Self) -> bool {
        let ordering = self.cmp(other);
        ordering == Ordering::Equal
    }
}
impl Eq for Interval {}

pub(super) fn bisect(
    controls: &[Vector],
    cancelled: &mut impl FnMut() -> bool,
) -> Result<[Vec<Vector>; 2], Fault> {
    let mut level = controls.to_vec();
    let mut first = vec![level[0].clone()];
    let mut second = vec![level.last().unwrap().clone()];
    let half = ExactRatio::from_finite(0.5);
    while level.len() > 1 {
        checkpoint(cancelled)?;
        level = level
            .windows(2)
            .map(|pair| scale(&add(&pair[0], &pair[1]), &half))
            .collect();
        first.push(level[0].clone());
        second.push(level.last().unwrap().clone());
    }
    second.reverse();
    Ok([first, second])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_restriction_preserves_nonconstant_polynomial_and_midpoint() {
        let controls: Vec<Vector> = [0., 2., 0.]
            .into_iter()
            .map(|x| [x, 0., 0.].map(ExactRatio::from_finite))
            .collect();
        let [left, right] = bisect(&controls, &mut || false).unwrap();
        let coordinates = |vectors: Vec<Vector>| {
            vectors
                .into_iter()
                .map(|v| v[0].clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(coordinates(left), [0., 1., 1.].map(ExactRatio::from_finite));
        assert_eq!(
            coordinates(right),
            [1., 1., 0.].map(ExactRatio::from_finite)
        );
        assert!(bisect(&controls, &mut || true).is_err());
    }
}
