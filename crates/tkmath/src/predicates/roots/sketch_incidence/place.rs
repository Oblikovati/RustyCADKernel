// SPDX-License-Identifier: GPL-2.0-only
//! Exact places along the source curve, compared in source order.
//!
//! Line and spline sources order contacts by their parameter. A conic source has
//! no rational parameter, so a direction `q` from its center is placed by the
//! chart of `q` around a reference ray `r` (taken in the sweep sense) and, inside
//! the two open half turns, by the cotangent key `K = -dot(r, q) / det(r, q)`,
//! which increases with the sweep angle. Both are algebraic, so places of
//! different equations compare exactly with the shared root engine.
use super::super::certificate_arithmetic::ExactRatio;
use super::algebraic::{IsolatedRoot, compare_roots};
use super::budget::Work;
use super::failure::SketchIncidenceCause as Cause;
use super::report::outward_ratio;
use std::cmp::Ordering;

/// An exact key value: a certified isolated root or a rational number.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum KeyValue {
    Root(IsolatedRoot),
    Ratio(ExactRatio),
}
/// Chart of a direction around a conic center, in sweep order from the reference ray.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum SketchSweepChart {
    /// On the reference ray itself (sweep angle 0).
    Reference,
    /// Strictly inside the first half turn.
    Upper,
    /// On the ray opposite the reference (sweep angle one half turn).
    Opposite,
    /// Strictly inside the second half turn.
    Lower,
}
/// Exact place of a contact or bound along the source.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum SourcePlace {
    /// Line or spline parameter.
    Parameter(KeyValue),
    /// Conic sweep chart; the cotangent key is present exactly in the open half turns.
    Sweep {
        chart: SketchSweepChart,
        cotangent: Option<KeyValue>,
    },
}
impl KeyValue {
    /// Outward binary64 bounds of the exact value.
    pub fn outward(&self) -> [f64; 2] {
        match self {
            Self::Root(root) => {
                let [lower, upper] = root.ratio_bounds();
                [outward_ratio(&lower)[0], outward_ratio(&upper)[1]]
            }
            Self::Ratio(value) => outward_ratio(value),
        }
    }
}
/// Exact order of two key values of possibly different equations.
pub(super) fn compare_values(
    first: &mut KeyValue,
    second: &mut KeyValue,
    work: &mut Work<'_>,
) -> Result<Ordering, Cause> {
    match (first, second) {
        (KeyValue::Root(left), KeyValue::Root(right)) => compare_roots(left, right, work),
        (KeyValue::Root(left), KeyValue::Ratio(right)) => left.compare_ratio(right, work),
        (KeyValue::Ratio(left), KeyValue::Root(right)) => {
            Ok(right.compare_ratio(left, work)?.reverse())
        }
        (KeyValue::Ratio(left), KeyValue::Ratio(right)) => Ok((*left).cmp(right)),
    }
}
/// Exact order of two places of the same source.
pub(super) fn compare_places(
    first: &mut SourcePlace,
    second: &mut SourcePlace,
    work: &mut Work<'_>,
) -> Result<Ordering, Cause> {
    use SourcePlace::{Parameter, Sweep};
    match (first, second) {
        (Parameter(left), Parameter(right)) => compare_values(left, right, work),
        (
            Sweep {
                chart: left_chart,
                cotangent: left,
            },
            Sweep {
                chart: right_chart,
                cotangent: right,
            },
        ) => match ((*left_chart).cmp(right_chart), left, right) {
            (Ordering::Equal, Some(left), Some(right)) => compare_values(left, right, work),
            (order, _, _) => Ok(order),
        },
        _ => Err(Cause::Invariant(
            "places of one source have different kinds",
        )),
    }
}
