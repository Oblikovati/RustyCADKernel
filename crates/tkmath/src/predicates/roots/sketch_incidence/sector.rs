// SPDX-License-Identifier: GPL-2.0-only
//! Exact membership of a contact direction in an angular sector of a conic.
//!
//! Membership uses only the signs of `det(boundary, q)` and `dot(boundary, q)` for
//! the two boundaries, where `q` is the contact direction from the center. A ray
//! boundary is an exact vector, so a contact on it is exactly `AtStart` or `AtEnd`.
//! An angle boundary `q(theta)` is enclosed and refined together with the contact
//! root until the sign separates, which always happens for `theta != 0`.
use super::super::certificate_arithmetic::{ExactRatio, RatioInterval};
use super::algebraic::IsolatedRoot;
use super::angle::{AngleFrame, FrameAngle};
use super::angle_admission::compare_half_turn;
use super::budget::Work;
use super::failure::SketchIncidenceCause as Cause;
use super::input::{SketchConicBounds, SketchConicSector};
use super::rational_polynomial::{RationalPolynomial, ratio};
use super::report::SketchLinePosition;
use super::root_sign::sign_at_root;
use crate::predicates::{OrientationSign, orient2d};
use num_bigint::Sign;
use std::cmp::Ordering;

/// A conic arc's bounds with the radii that scale an angle frame.
#[derive(Clone, Copy)]
pub(super) struct ArcBounds<'a> {
    pub bounds: &'a SketchConicBounds,
    pub radii: [f64; 2],
}
/// One sector boundary: an exact ray or a frame angle.
enum Boundary {
    Ray([f64; 2]),
    Angle(FrameAngle),
}
#[derive(Clone, Copy, PartialEq)]
enum Turn {
    Convex,
    Reflex,
    Half,
}
/// The contact direction `q = (x - cx w, y - cy w)` (scaled by `w > 0`) at a root.
pub(super) struct ContactDirection<'a> {
    pub root: &'a mut IsolatedRoot,
    pub relative: [RationalPolynomial; 2],
}
impl ContactDirection<'_> {
    fn exact_cross(
        &mut self,
        vector: &[ExactRatio; 2],
        work: &mut Work<'_>,
    ) -> Result<Ordering, Cause> {
        let [vx, vy] = vector;
        let [qx, qy] = &self.relative;
        let value = qy.scale(vx).add(&qx.scale(&vy.negated()));
        sign(self.root, &value, work)
    }
    fn exact_dot(
        &mut self,
        vector: &[ExactRatio; 2],
        work: &mut Work<'_>,
    ) -> Result<Ordering, Cause> {
        let [vx, vy] = vector;
        let [qx, qy] = &self.relative;
        sign(self.root, &qx.scale(vx).add(&qy.scale(vy)), work)
    }
    /// Exact sign of `det(boundary, q)`.
    fn cross(&mut self, boundary: &mut Boundary, work: &mut Work<'_>) -> Result<Ordering, Cause> {
        match boundary {
            Boundary::Ray(vector) => self.exact_cross(&vector.map(ratio), work),
            Boundary::Angle(angle) => match angle.exact_direction() {
                Some(vector) => self.exact_cross(&vector, work),
                None => self.enclosed_cross(angle, work),
            },
        }
    }
    /// Exact sign of `dot(boundary, q)`, needed only on an exact boundary line.
    fn dot(&mut self, boundary: &Boundary, work: &mut Work<'_>) -> Result<Ordering, Cause> {
        match boundary {
            Boundary::Ray(vector) => self.exact_dot(&vector.map(ratio), work),
            Boundary::Angle(angle) => match angle.exact_direction() {
                Some(vector) => self.exact_dot(&vector, work),
                None => Err(Cause::Invariant(
                    "a nonzero angle met an algebraic direction",
                )),
            },
        }
    }
    // Refine the contact root and the angle until the determinant enclosure separates.
    fn enclosed_cross(
        &mut self,
        angle: &mut FrameAngle,
        work: &mut Work<'_>,
    ) -> Result<Ordering, Cause> {
        let mut step = 0u32;
        loop {
            work.charge()?;
            let [lower, upper] = self.root.ratio_bounds();
            let parameter = RatioInterval { lower, upper };
            let q = self.relative.clone().map(|value| value.enclose(&parameter));
            let v = angle.direction();
            let negated = v[1].multiply(&q[0]);
            let value = v[0].multiply(&q[1]).add(&RatioInterval {
                lower: negated.upper.negated(),
                upper: negated.lower.negated(),
            });
            if value.lower.sign() == Ordering::Greater {
                return Ok(Ordering::Greater);
            }
            if value.upper.sign() == Ordering::Less {
                return Ok(Ordering::Less);
            }
            // Scheduling only: bisect the root every step and add angle bits every
            // fourth step (or every step once the root is exact).
            step += 1;
            if self.root.exact().is_some() || step.is_multiple_of(4) {
                angle.refine();
            }
            self.root.bisect(work)?;
        }
    }
}
fn sign(
    root: &mut IsolatedRoot,
    polynomial: &RationalPolynomial,
    work: &mut Work<'_>,
) -> Result<Ordering, Cause> {
    Ok(match sign_at_root(root, polynomial, work)? {
        Sign::Minus => Ordering::Less,
        Sign::NoSign => Ordering::Equal,
        Sign::Plus => Ordering::Greater,
    })
}
/// Exact sector position, or None outside the sector.
pub(super) fn sector_position(
    direction: &mut ContactDirection<'_>,
    arc: ArcBounds<'_>,
    work: &mut Work<'_>,
) -> Result<Option<SketchLinePosition>, Cause> {
    let (mut first, mut second, labels, turn) = counterclockwise_frame(arc);
    let after_first = direction.cross(&mut first, work)?;
    let before_second = direction.cross(&mut second, work)?.reverse();
    if on_boundary(direction, &first, after_first, work)? {
        return Ok(Some(labels[0]));
    }
    if on_boundary(direction, &second, before_second, work)? {
        return Ok(Some(labels[1]));
    }
    let inside = match turn {
        Turn::Convex => after_first.is_gt() && before_second.is_gt(),
        Turn::Reflex => after_first.is_gt() || before_second.is_gt(),
        // Admission refused equal rays, so these directions are opposite (a half turn).
        Turn::Half => after_first.is_gt(),
    };
    Ok(inside.then_some(SketchLinePosition::Interior))
}
type SectorFrame = (Boundary, Boundary, [SketchLinePosition; 2], Turn);
// A clockwise sweep from start to end is the counterclockwise sweep from end to start.
fn counterclockwise_frame(arc: ArcBounds<'_>) -> SectorFrame {
    use SketchLinePosition::{AtEnd, AtStart};
    let (start, end, counterclockwise, turn) = match arc.bounds {
        SketchConicBounds::Rays(sector) => rays(sector),
        SketchConicBounds::Angles(angles) => {
            let frame = AngleFrame {
                axis: angles.axis,
                counterclockwise: angles.counterclockwise,
                radii: arc.radii,
            };
            let [start, end] = angles
                .bounds
                .map(|theta| Boundary::Angle(FrameAngle::new(frame.clone(), theta)));
            let turn = match compare_half_turn(angles.bounds) {
                Ordering::Less => Turn::Convex,
                _ => Turn::Reflex,
            };
            (start, end, angles.counterclockwise, turn)
        }
    };
    if counterclockwise {
        return (start, end, [AtStart, AtEnd], turn);
    }
    (end, start, [AtEnd, AtStart], turn)
}
fn rays(sector: &SketchConicSector) -> (Boundary, Boundary, bool, Turn) {
    let (first, second) = match sector.counterclockwise {
        true => (sector.start, sector.end),
        false => (sector.end, sector.start),
    };
    let turn = match orient2d([0., 0.], first, second).expect("admitted sector directions") {
        OrientationSign::Positive => Turn::Convex,
        OrientationSign::Negative => Turn::Reflex,
        OrientationSign::Zero => Turn::Half,
    };
    (
        Boundary::Ray(sector.start),
        Boundary::Ray(sector.end),
        sector.counterclockwise,
        turn,
    )
}
// On the boundary itself, not on its opposite through the center.
fn on_boundary(
    direction: &mut ContactDirection<'_>,
    boundary: &Boundary,
    cross: Ordering,
    work: &mut Work<'_>,
) -> Result<bool, Cause> {
    if cross != Ordering::Equal {
        return Ok(false);
    }
    Ok(direction.dot(boundary, work)? == Ordering::Greater)
}
