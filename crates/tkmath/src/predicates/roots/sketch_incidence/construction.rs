// SPDX-License-Identifier: GPL-2.0-only
//! Refine an isolated root until its exact world enclosure certifies the error limit.
use super::super::certificate_arithmetic::{
    ExactRatio, RatioInterval, rounding,
    world::{WorldEnvelope, finish_envelope},
};
use super::algebraic::IsolatedRoot;
use super::budget::Work;
use super::failure::SketchIncidenceCause as Cause;
use super::rational_polynomial::ratio;

/// Exact world point enclosure for a root bracket; None while it cannot be enclosed.
pub(super) type PointEnclosure<'a> = dyn Fn(&IsolatedRoot) -> Option<[RatioInterval; 2]> + 'a;

/// Requested Euclidean construction error, never widened.
pub(super) struct WorldLimit {
    requested: f64,
    squared: ExactRatio,
}
impl WorldLimit {
    pub fn new(requested: f64) -> Self {
        let limit = ratio(requested);
        Self {
            requested,
            squared: limit.multiply(&limit),
        }
    }
    /// Bisect until the nearest binary64 point is certified within the limit.
    pub fn construct(
        &self,
        root: &mut IsolatedRoot,
        work: &mut Work<'_>,
        enclosure: &PointEnclosure<'_>,
    ) -> Result<WorldEnvelope, Cause> {
        loop {
            work.check()?;
            let intervals = enclosure(root);
            let achieved = match intervals.clone().map(finish_envelope) {
                Some(Ok(candidate)) if candidate.error_squared <= self.squared => {
                    return Ok(candidate);
                }
                Some(Ok(candidate)) => Some(candidate.achieved_error),
                _ => None,
            };
            let floor = intervals.as_ref().map_or(ratio(0.), resolution_floor);
            if root.exact().is_some() || floor > self.squared {
                return Err(Cause::ConstructionResolution {
                    requested: self.requested,
                    achieved,
                });
            }
            for _ in 0..self.halvings(achieved) {
                root.bisect(work)?;
            }
        }
    }
    // Scheduling only: the enclosure width roughly halves the error per bisection,
    // so skip envelope checks that cannot yet succeed. Certification is unchanged.
    fn halvings(&self, achieved: Option<f64>) -> u32 {
        let ratio = achieved.map_or(2., |value| value / self.requested);
        (ratio.log2().floor() as u32).clamp(1, 64)
    }
}
// Inside one binary64 gap per axis, no refinement can bring any representable
// point closer than the distance from the enclosure to the gap ends.
fn resolution_floor(intervals: &[RatioInterval; 2]) -> ExactRatio {
    intervals.iter().fold(ratio(0.), |total, axis| {
        let distance = gap_floor(axis);
        total.add(&distance.multiply(&distance))
    })
}
fn gap_floor(axis: &RatioInterval) -> ExactRatio {
    let (Some(low), Some(high)) = (
        rounding::outward_binary64(&axis.lower),
        rounding::outward_binary64(&axis.upper),
    ) else {
        return ratio(0.);
    };
    if low != high || low[0] == low[1] {
        return ratio(0.);
    }
    let below = axis.lower.subtract(&ratio(low[0]));
    let above = ratio(low[1]).subtract(&axis.upper);
    below.min(above)
}
