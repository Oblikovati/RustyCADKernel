// SPDX-License-Identifier: GPL-2.0-only
//! The exact source line `A + t (B - A)`: line positions and world construction.
use super::super::DyadicBound;
use super::super::certificate_arithmetic::{ExactRatio, RatioInterval, world::WorldEnvelope};
use super::algebraic::IsolatedRoot;
use super::budget::Work;
use super::construction::WorldLimit;
use super::failure::SketchIncidenceCause as Cause;
use super::rational_polynomial::ratio;
use super::report::SketchLinePosition;
use std::cmp::Ordering;

pub(super) struct SourceLine {
    pub origin: [ExactRatio; 2],
    pub direction: [ExactRatio; 2],
    pub squared_length: ExactRatio,
    requested: f64,
}
impl SourceLine {
    pub fn new(points: [[f64; 2]; 2], requested: f64) -> Self {
        let origin = points[0].map(ratio);
        let direction = std::array::from_fn(|axis| ratio(points[1][axis]).subtract(&origin[axis]));
        let squared_length = dot(&direction, &direction);
        Self {
            origin,
            direction,
            squared_length,
            requested,
        }
    }
    /// Caller world error limit, never widened.
    pub fn requested(&self) -> f64 {
        self.requested
    }
    /// Exact position of an isolated line parameter relative to 0 and 1.
    pub fn position(
        &self,
        root: &mut IsolatedRoot,
        work: &mut Work<'_>,
    ) -> Result<SketchLinePosition, Cause> {
        position_between(root, [0., 1.], work)
    }
    /// Narrow the parameter until the constructed point meets the requested error.
    pub fn construct(
        &self,
        root: &mut IsolatedRoot,
        work: &mut Work<'_>,
    ) -> Result<WorldEnvelope, Cause> {
        let limit = WorldLimit::new(self.requested);
        limit.construct(root, work, &|root: &IsolatedRoot| {
            Some(self.point_enclosure(root))
        })
    }
    fn point_enclosure(&self, root: &IsolatedRoot) -> [RatioInterval; 2] {
        let [lower, upper] = root.ratio_bounds();
        let parameter = RatioInterval { lower, upper };
        std::array::from_fn(|axis| {
            parameter
                .multiply(&RatioInterval::point(self.direction[axis].clone()))
                .add(&RatioInterval::point(self.origin[axis].clone()))
        })
    }
    /// Exact line parameter of a supplied rational point projected onto the line.
    pub fn projected_parameter(&self, point: [f64; 2]) -> ExactRatio {
        let offset: [ExactRatio; 2] =
            std::array::from_fn(|axis| ratio(point[axis]).subtract(&self.origin[axis]));
        dot(&offset, &self.direction).divide(&self.squared_length)
    }
}
/// Exact position of an isolated root relative to two increasing binary64 values.
pub(super) fn position_between(
    root: &mut IsolatedRoot,
    range: [f64; 2],
    work: &mut Work<'_>,
) -> Result<SketchLinePosition, Cause> {
    let [start, end] = range.map(|value| DyadicBound::from_f64(value).expect("finite range"));
    Ok(match root.split_at(&start, work)? {
        Ordering::Less => SketchLinePosition::BeforeStart,
        Ordering::Equal => SketchLinePosition::AtStart,
        Ordering::Greater => match root.split_at(&end, work)? {
            Ordering::Less => SketchLinePosition::Interior,
            Ordering::Equal => SketchLinePosition::AtEnd,
            Ordering::Greater => SketchLinePosition::AfterEnd,
        },
    })
}
/// Exact position of a rational value relative to 0 and 1.
pub(super) fn ratio_position(value: &ExactRatio) -> SketchLinePosition {
    match (value.cmp(&ratio(0.)), value.cmp(&ratio(1.))) {
        (Ordering::Less, _) => SketchLinePosition::BeforeStart,
        (Ordering::Equal, _) => SketchLinePosition::AtStart,
        (_, Ordering::Less) => SketchLinePosition::Interior,
        (_, Ordering::Equal) => SketchLinePosition::AtEnd,
        _ => SketchLinePosition::AfterEnd,
    }
}
pub(super) fn dot(left: &[ExactRatio; 2], right: &[ExactRatio; 2]) -> ExactRatio {
    left[0]
        .multiply(&right[0])
        .add(&left[1].multiply(&right[1]))
}
