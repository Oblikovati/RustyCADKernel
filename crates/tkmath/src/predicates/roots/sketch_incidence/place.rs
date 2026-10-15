// SPDX-License-Identifier: GPL-2.0-only
//! Exact places along the source curve, compared in source order.
//!
//! Line and spline sources order contacts by their parameter. A conic source has
//! no rational parameter, so a direction `q` from its center is placed by the
//! chart of `q` around a reference ray `r` (taken in the sweep sense) and, inside
//! the two open half turns, by the cotangent key `K = -dot(r, q) / det(r, q)`,
//! which increases with the sweep angle. Both are algebraic, so places of
//! different equations compare exactly with the shared root engine. An arc bounded
//! by parameter angles keeps its exact frame axis as the reference; places before
//! its start angle are marked `turn` and follow every other place.
use super::super::certificate_arithmetic::{ExactRatio, RatioInterval};
use super::algebraic::{IsolatedRoot, compare_roots};
use super::angle::FrameAngle;
use super::budget::Work;
use super::failure::SketchIncidenceCause as Cause;
use super::report::outward_ratio;
use super::surd_key::SurdKey;
use std::cmp::Ordering;

/// An exact key value: a certified root, a rational number, a frame angle's key,
/// or a surd ratio at an offset contact (ADR 0080).
#[derive(Clone, Debug, PartialEq)]
pub(super) enum KeyValue {
    Root(IsolatedRoot),
    Ratio(ExactRatio),
    Angle(Box<AngleKey>),
    Surd(Box<SurdKey>),
}
/// The cotangent key of a frame angle's direction in a sweep frame.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct AngleKey {
    pub angle: FrameAngle,
    pub reference: [ExactRatio; 2],
    pub counterclockwise: bool,
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
        turn: bool,
        chart: SketchSweepChart,
        cotangent: Option<KeyValue>,
    },
}
impl SourcePlace {
    /// A sweep place before any wrap normalization.
    pub fn sweep(chart: SketchSweepChart, cotangent: Option<KeyValue>) -> Self {
        Self::Sweep {
            turn: false,
            chart,
            cotangent,
        }
    }
}
fn negate(value: RatioInterval) -> RatioInterval {
    RatioInterval {
        lower: value.upper.negated(),
        upper: value.lower.negated(),
    }
}
impl AngleKey {
    /// `[o det(r, q), dot(r, q)]` enclosures of the angle direction.
    pub fn across_along(&self) -> [RatioInterval; 2] {
        let [qx, qy] = self.angle.direction();
        let [rx, ry] = self.reference.clone().map(RatioInterval::point);
        let mut across = qy.multiply(&rx).add(&negate(qx.multiply(&ry)));
        if !self.counterclockwise {
            across = negate(across);
        }
        [across, qx.multiply(&rx).add(&qy.multiply(&ry))]
    }
    /// Enclosure of `-along / across`, once `across` excludes zero.
    fn interval(&self) -> Option<RatioInterval> {
        let [across, along] = self.across_along();
        let reciprocal = across.reciprocal()?;
        Some(negate(along.multiply(&reciprocal)))
    }
}
impl KeyValue {
    /// Outward binary64 bounds of the exact value.
    pub fn outward(&self) -> [f64; 2] {
        let interval = self.enclosure().unwrap_or(RatioInterval {
            lower: ExactRatio::from_finite(-f64::MAX),
            upper: ExactRatio::from_finite(f64::MAX),
        });
        [
            outward_ratio(&interval.lower)[0],
            outward_ratio(&interval.upper)[1],
        ]
    }
    fn enclosure(&self) -> Option<RatioInterval> {
        match self {
            Self::Root(root) => {
                let [lower, upper] = root.ratio_bounds();
                Some(RatioInterval { lower, upper })
            }
            Self::Ratio(value) => Some(RatioInterval::point(value.clone())),
            Self::Angle(key) => key.interval(),
            Self::Surd(key) => key.enclosure(),
        }
    }
    fn refine(&mut self, work: &mut Work<'_>) -> Result<(), Cause> {
        match self {
            Self::Root(root) => root.bisect(work),
            Self::Ratio(_) => Ok(()),
            Self::Angle(key) => {
                key.angle.refine();
                Ok(())
            }
            Self::Surd(key) => key.refine(work),
        }
    }
    // Replace a surd key by its image root, so ties are decided by common factors.
    fn resolve_image(&mut self, work: &mut Work<'_>) -> Result<(), Cause> {
        if let Self::Surd(key) = self {
            *self = Self::Root(key.image(work)?);
        }
        Ok(())
    }
}
/// Exact order of two key values of possibly different equations.
pub(super) fn compare_values(
    first: &mut KeyValue,
    second: &mut KeyValue,
    work: &mut Work<'_>,
) -> Result<Ordering, Cause> {
    match (&mut *first, &mut *second) {
        (KeyValue::Root(left), KeyValue::Root(right)) => compare_roots(left, right, work),
        (KeyValue::Root(left), KeyValue::Ratio(right)) => left.compare_ratio(right, work),
        (KeyValue::Ratio(left), KeyValue::Root(right)) => {
            Ok(right.compare_ratio(left, work)?.reverse())
        }
        (KeyValue::Ratio(left), KeyValue::Ratio(right)) => Ok((*left).cmp(right)),
        (KeyValue::Surd(left), KeyValue::Ratio(right)) => left.compare_ratio(right, work),
        (KeyValue::Ratio(left), KeyValue::Surd(right)) => {
            Ok(right.compare_ratio(left, work)?.reverse())
        }
        (KeyValue::Angle(left), KeyValue::Angle(right))
            if left.angle.same_direction(&right.angle) =>
        {
            Ok(Ordering::Equal)
        }
        (KeyValue::Surd(_), _) | (_, KeyValue::Surd(_)) => surd_order(first, second, work),
        _ => separate(first, second, work),
    }
}
/// Refinements tried before a surd key builds its image equation.
const SURD_SEPARATIONS: u32 = 256;
// Distinct values separate by refinement; an algebraic tie needs both image roots.
fn surd_order(
    first: &mut KeyValue,
    second: &mut KeyValue,
    work: &mut Work<'_>,
) -> Result<Ordering, Cause> {
    if let Some(order) = separate_within(first, second, SURD_SEPARATIONS, work)? {
        return Ok(order);
    }
    if matches!(first, KeyValue::Angle(_)) || matches!(second, KeyValue::Angle(_)) {
        return separate(first, second, work);
    }
    first.resolve_image(work)?;
    second.resolve_image(work)?;
    compare_values(first, second, work)
}
// An angle key never equals an algebraic key or another angle's key unless they
// name the same direction (checked above), so refinement always separates them.
fn separate(
    first: &mut KeyValue,
    second: &mut KeyValue,
    work: &mut Work<'_>,
) -> Result<Ordering, Cause> {
    loop {
        if let Some(order) = separate_within(first, second, 1, work)? {
            return Ok(order);
        }
    }
}
// At most `rounds` refinements; None while the enclosures still overlap.
fn separate_within(
    first: &mut KeyValue,
    second: &mut KeyValue,
    rounds: u32,
    work: &mut Work<'_>,
) -> Result<Option<Ordering>, Cause> {
    for _ in 0..rounds {
        work.charge()?;
        if let (Some(left), Some(right)) = (first.enclosure(), second.enclosure()) {
            if left.upper < right.lower {
                return Ok(Some(Ordering::Less));
            }
            if right.upper < left.lower {
                return Ok(Some(Ordering::Greater));
            }
        }
        // Scheduling only: refine the wider enclosure (a rational never refines).
        if wider(first, second) {
            first.refine(work)?;
        } else {
            second.refine(work)?;
        }
    }
    Ok(None)
}
// Exact enclosure widths: binary64 spreads stall once both are one ulp wide, and a
// value that cannot be enclosed yet is the wider (ADR 0080).
fn wider(first: &KeyValue, second: &KeyValue) -> bool {
    match (first.enclosure(), second.enclosure()) {
        (Some(left), Some(right)) => {
            left.upper.subtract(&left.lower) >= right.upper.subtract(&right.lower)
        }
        (None, _) => true,
        (Some(_), None) => false,
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
                turn: left_turn,
                chart: left_chart,
                cotangent: left,
            },
            Sweep {
                turn: right_turn,
                chart: right_chart,
                cotangent: right,
            },
        ) => {
            let order = (*left_turn, *left_chart).cmp(&(*right_turn, *right_chart));
            match (order, left, right) {
                (Ordering::Equal, Some(left), Some(right)) => compare_values(left, right, work),
                (order, _, _) => Ok(order),
            }
        }
        _ => Err(Cause::Invariant(
            "places of one source have different kinds",
        )),
    }
}
