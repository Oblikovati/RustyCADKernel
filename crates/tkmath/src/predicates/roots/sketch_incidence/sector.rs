// SPDX-License-Identifier: GPL-2.0-only
//! Exact membership of a contact direction in an angular sector of a conic.
//!
//! Membership uses only the signs of `det(ray, q)` and `dot(ray, q)` for the two
//! boundary rays, where `q` is the contact direction from the center. A contact on
//! a boundary ray is exactly `AtStart` or `AtEnd`.
use super::algebraic::IsolatedRoot;
use super::budget::Work;
use super::failure::SketchIncidenceCause as Cause;
use super::input::SketchConicSector;
use super::rational_polynomial::{RationalPolynomial, ratio};
use super::report::SketchLinePosition;
use super::root_sign::sign_at_root;
use crate::predicates::{OrientationSign, orient2d};
use num_bigint::Sign;
use std::cmp::Ordering;

/// The contact direction `q = (x - cx w, y - cy w)` (scaled by `w > 0`) at a root.
pub(super) struct ContactDirection<'a> {
    pub root: &'a mut IsolatedRoot,
    pub relative: [RationalPolynomial; 2],
}
impl ContactDirection<'_> {
    /// Exact sign of `det(direction, q)`.
    pub fn cross(&mut self, direction: [f64; 2], work: &mut Work<'_>) -> Result<Ordering, Cause> {
        let [vx, vy] = direction.map(ratio);
        let [qx, qy] = &self.relative;
        let value = qy.scale(&vx).add(&qx.scale(&vy.negated()));
        sign(self.root, &value, work)
    }
    /// Exact sign of `dot(direction, q)`.
    pub fn dot(&mut self, direction: [f64; 2], work: &mut Work<'_>) -> Result<Ordering, Cause> {
        let [vx, vy] = direction.map(ratio);
        let [qx, qy] = &self.relative;
        sign(self.root, &qx.scale(&vx).add(&qy.scale(&vy)), work)
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
    sector: &SketchConicSector,
    work: &mut Work<'_>,
) -> Result<Option<SketchLinePosition>, Cause> {
    let (first, second, labels) = counterclockwise_frame(sector);
    let after_first = direction.cross(first, work)?;
    let before_second = direction.cross(second, work)?.reverse();
    if on_ray(direction, first, after_first, work)? {
        return Ok(Some(labels[0]));
    }
    if on_ray(direction, second, before_second, work)? {
        return Ok(Some(labels[1]));
    }
    let inside = match orient2d([0., 0.], first, second).expect("admitted sector directions") {
        OrientationSign::Positive => after_first.is_gt() && before_second.is_gt(),
        OrientationSign::Negative => after_first.is_gt() || before_second.is_gt(),
        // Admission refused equal rays, so these directions are opposite (a half turn).
        OrientationSign::Zero => after_first.is_gt(),
    };
    Ok(inside.then_some(SketchLinePosition::Interior))
}
type SectorFrame = ([f64; 2], [f64; 2], [SketchLinePosition; 2]);
// A clockwise sweep from start to end is the counterclockwise sweep from end to start.
fn counterclockwise_frame(sector: &SketchConicSector) -> SectorFrame {
    use SketchLinePosition::{AtEnd, AtStart};
    if sector.counterclockwise {
        return (sector.start, sector.end, [AtStart, AtEnd]);
    }
    (sector.end, sector.start, [AtEnd, AtStart])
}
// On the boundary ray itself, not on its opposite through the center.
fn on_ray(
    direction: &mut ContactDirection<'_>,
    ray: [f64; 2],
    cross: Ordering,
    work: &mut Work<'_>,
) -> Result<bool, Cause> {
    if cross != Ordering::Equal {
        return Ok(false);
    }
    Ok(direction.dot(ray, work)? == Ordering::Greater)
}
