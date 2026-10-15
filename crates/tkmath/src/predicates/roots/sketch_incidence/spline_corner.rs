// SPDX-License-Identifier: GPL-2.0-only
//! Crossing or touching at a target interior knot, from the four half tangents.
//!
//! At the contact the source arrives along `-P'_L` and leaves along `P'_R`; these
//! two rays split a neighbourhood into two sides. The target arrives along `-Q'_L`
//! and leaves along `Q'_R`, both exact rational vectors at the binary64 knot. The
//! target crosses exactly when its two rays lie on different sides. A ray along a
//! source ray (a tangential piece) or a stationary or cusped source is a named
//! incomplete result.
use super::super::certificate_arithmetic::ExactRatio;
use super::algebraic::IsolatedRoot;
use super::budget::Work;
use super::failure::SketchIncidenceCause as Cause;
use super::pieces::PieceRoot;
use super::rational_polynomial::{RationalPolynomial, ratio};
use super::report::SketchContactTransition;
use super::root_sign::sign_at_root;
use super::spans::{HomogeneousPoint, ParametricSpan};
use num_bigint::Sign;

/// A contact at the knot between two target spans.
pub(super) struct Corner<'a> {
    pub source: &'a [ParametricSpan],
    pub root: &'a PieceRoot,
    pub target: [&'a HomogeneousPoint; 2],
    pub knot: f64,
}
/// Tangent direction `(x' w - x w', y' w - y w')` of a homogeneous point.
fn tangent(point: &HomogeneousPoint) -> [RationalPolynomial; 2] {
    let along = |coordinate: &RationalPolynomial| {
        coordinate.derivative().multiply(&point.w).add(
            &coordinate
                .multiply(&point.w.derivative())
                .scale(&ratio(-1.)),
        )
    };
    [along(&point.x), along(&point.y)]
}
fn cross(first: &[RationalPolynomial; 2], second: &[RationalPolynomial; 2]) -> RationalPolynomial {
    first[0]
        .multiply(&second[1])
        .add(&first[1].multiply(&second[0]).scale(&ratio(-1.)))
}
fn constant(vector: &[ExactRatio; 2]) -> [RationalPolynomial; 2] {
    vector.clone().map(RationalPolynomial::constant)
}
pub(super) fn corner_transition(
    corner: Corner<'_>,
    contact: &mut IsolatedRoot,
    work: &mut Work<'_>,
) -> Result<SketchContactTransition, Cause> {
    let [arriving, leaving] = source_rays(&corner);
    let knot = ratio(corner.knot);
    let target = corner
        .target
        .map(|point| tangent(point).map(|coordinate| coordinate.evaluate(&knot)));
    let incoming = constant(&target[0].clone().map(|value| value.negated()));
    let outgoing = constant(&target[1]);
    let sides = SourceSides { arriving, leaving };
    let first = sides.left(&incoming, contact, work)?;
    let second = sides.left(&outgoing, contact, work)?;
    Ok(match first == second {
        true => SketchContactTransition::Touching,
        false => SketchContactTransition::Crossing,
    })
}
// `-P'_L` and `P'_R`; they differ only when the contact is also at a source knot.
fn source_rays(corner: &Corner<'_>) -> [[RationalPolynomial; 2]; 2] {
    let piece = corner.root.piece;
    let left = tangent(&corner.source[piece].point);
    let at_end = corner.source[piece]
        .range
        .is_some_and(|range| corner.root.contact.exact() == Some(&super::pieces::dyadic(range[1])));
    let right = match corner.source.get(piece + 1) {
        Some(next) if at_end => tangent(&next.point),
        _ => left.clone(),
    };
    [left.map(|coordinate| coordinate.scale(&ratio(-1.))), right]
}
struct SourceSides {
    arriving: [RationalPolynomial; 2],
    leaving: [RationalPolynomial; 2],
}
impl SourceSides {
    /// True strictly inside the counterclockwise sector from the leaving to the arriving ray.
    fn left(
        &self,
        ray: &[RationalPolynomial; 2],
        contact: &mut IsolatedRoot,
        work: &mut Work<'_>,
    ) -> Result<bool, Cause> {
        let turn = sign_at_root(contact, &cross(&self.leaving, &self.arriving), work)?;
        let after_leaving = sign_at_root(contact, &cross(&self.leaving, ray), work)?;
        let after_arriving = sign_at_root(contact, &cross(&self.arriving, ray), work)?;
        if after_leaving == Sign::NoSign || after_arriving == Sign::NoSign {
            return Err(Cause::KnotTangency);
        }
        Ok(match turn {
            Sign::Plus => after_leaving == Sign::Plus && after_arriving == Sign::Minus,
            Sign::Minus => after_leaving == Sign::Plus || after_arriving == Sign::Minus,
            Sign::NoSign => self.smooth(contact, work)? && after_leaving == Sign::Plus,
        })
    }
    // Collinear rays are a smooth pass only when they point in opposite directions.
    fn smooth(&self, contact: &mut IsolatedRoot, work: &mut Work<'_>) -> Result<bool, Cause> {
        let [lx, ly] = &self.leaving;
        let [ax, ay] = &self.arriving;
        let dot = lx.multiply(ax).add(&ly.multiply(ay));
        match sign_at_root(contact, &dot, work)? {
            Sign::Minus => Ok(true),
            _ => Err(Cause::KnotTangency),
        }
    }
}
