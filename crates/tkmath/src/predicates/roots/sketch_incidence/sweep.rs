// SPDX-License-Identifier: GPL-2.0-only
//! Sweep frames of conic sources and the placement of contact points on a source.
//!
//! The frame measures directions `q` from the center against a reference ray `r`
//! in the sweep sense `o` (+1 counterclockwise): `across = o det(r, q)`,
//! `along = dot(r, q)`. A linear map with positive determinant keeps the cyclic
//! order of directions, so the polar order of `q` is also the parametric order of
//! an ellipse. The cotangent key `-along / across` increases with the sweep angle
//! inside each open half turn.
use super::super::certificate_arithmetic::ExactRatio;
use super::algebraic::IsolatedRoot;
use super::budget::Work;
use super::failure::SketchIncidenceCause as Cause;
use super::place::{KeyValue, SketchSweepChart, SourcePlace};
use super::rational_polynomial::{RationalPolynomial, ratio};
use super::root_map::{MappedEquation, RationalMap};
use super::root_sign::sign_at_root;
use num_bigint::Sign;
use std::cmp::Ordering;

/// Center, exact reference ray and sweep sense of a conic source.
#[derive(Clone, Debug)]
pub(super) struct SweepFrame {
    pub center: [ExactRatio; 2],
    pub reference: [ExactRatio; 2],
    pub counterclockwise: bool,
}
/// How a point given by polynomials of a contact variable is placed on the source.
pub(super) enum PointPlacer {
    /// Line source: the image of the line parameter map.
    Parameter(MappedEquation),
    /// Conic source: chart signs and the cotangent key map.
    Sweep {
        across: RationalPolynomial,
        along: RationalPolynomial,
        key: MappedEquation,
    },
}
impl SweepFrame {
    pub fn new(center: [f64; 2], reference: [f64; 2], counterclockwise: bool) -> Self {
        Self {
            center: center.map(ratio),
            reference: reference.map(ratio),
            counterclockwise,
        }
    }
    fn oriented(&self, value: ExactRatio) -> ExactRatio {
        if self.counterclockwise {
            value
        } else {
            value.negated()
        }
    }
    /// `[o det(r, q), dot(r, q)]` for polynomial relative coordinates.
    pub fn across_along(&self, relative: &[RationalPolynomial; 2]) -> [RationalPolynomial; 2] {
        let [rx, ry] = &self.reference;
        let [qx, qy] = relative;
        let across = qy.scale(rx).add(&qx.scale(&ry.negated()));
        let across = across.scale(&self.oriented(ratio(1.)));
        [across, qx.scale(rx).add(&qy.scale(ry))]
    }
    /// Exact place of a nonzero rational direction from the center.
    pub fn exact_place(&self, direction: &[ExactRatio; 2]) -> SourcePlace {
        let [rx, ry] = &self.reference;
        let [qx, qy] = direction;
        let across = self.oriented(rx.multiply(qy).subtract(&ry.multiply(qx)));
        let along = rx.multiply(qx).add(&ry.multiply(qy));
        let chart = chart(across.sign(), along.sign());
        let cotangent = (across.sign() != Ordering::Equal)
            .then(|| KeyValue::Ratio(along.negated().divide(&across)));
        SourcePlace::Sweep { chart, cotangent }
    }
    /// Placer for a contact point `(x, y) / w` with `w > 0` at every contact.
    pub fn placer(
        &self,
        relative: &[RationalPolynomial; 2],
        contact: &RationalPolynomial,
        work: &mut Work<'_>,
    ) -> Result<PointPlacer, Cause> {
        let [across, along] = self.across_along(relative);
        let map = RationalMap {
            numerator: along.scale(&ratio(-1.)),
            denominator: across.clone(),
        };
        let key = MappedEquation::new(contact, map, work)?;
        Ok(PointPlacer::Sweep { across, along, key })
    }
}
/// Chart from the exact signs of `across` and `along`.
pub(super) fn chart(across: Ordering, along: Ordering) -> SketchSweepChart {
    match (across, along) {
        (Ordering::Greater, _) => SketchSweepChart::Upper,
        (Ordering::Less, _) => SketchSweepChart::Lower,
        (Ordering::Equal, Ordering::Less) => SketchSweepChart::Opposite,
        (Ordering::Equal, _) => SketchSweepChart::Reference,
    }
}
fn ordering(sign: Sign) -> Ordering {
    match sign {
        Sign::Minus => Ordering::Less,
        Sign::NoSign => Ordering::Equal,
        Sign::Plus => Ordering::Greater,
    }
}
impl PointPlacer {
    /// Exact source place of the contact; the contact bracket may be narrowed.
    pub fn place(
        &self,
        contact: &mut IsolatedRoot,
        work: &mut Work<'_>,
    ) -> Result<SourcePlace, Cause> {
        match self {
            Self::Parameter(map) => Ok(SourcePlace::Parameter(KeyValue::Root(
                map.map_root(contact, work)?,
            ))),
            Self::Sweep { across, along, key } => {
                let across_sign = ordering(sign_at_root(contact, across, work)?);
                if across_sign != Ordering::Equal {
                    let cotangent = Some(KeyValue::Root(key.map_root(contact, work)?));
                    let chart = chart(across_sign, Ordering::Equal);
                    return Ok(SourcePlace::Sweep { chart, cotangent });
                }
                let along_sign = ordering(sign_at_root(contact, along, work)?);
                Ok(SourcePlace::Sweep {
                    chart: chart(Ordering::Equal, along_sign),
                    cotangent: None,
                })
            }
        }
    }
}
