// SPDX-License-Identifier: GPL-2.0-only
//! Pending conic-source contacts: their exact point, target sector test and
//! world construction.
//!
//! A point on a line through the source center at a quadratic irrational scale
//! `sqrt(rho)` is written with `X = sqrt(rho)` as the isolated positive root of
//! `X^2 - rho`, so every sign and enclosure is a linear function of one exact root.
use super::super::certificate_arithmetic::{ExactRatio, RatioInterval, world::WorldEnvelope};
use super::algebraic::IsolatedRoot;
use super::budget::Work;
use super::conic_pair::ConicPair;
use super::construction::WorldLimit;
use super::contact::{ContactDraft, construct_point, finish};
use super::equation::{cauchy_bounds, isolate, transition};
use super::failure::{SketchIncidenceCause as Cause, SketchIncidenceStage};
use super::pieces::PieceRoot;
use super::place::{KeyValue, SourcePlace};
use super::rational_polynomial::{RationalPolynomial, ratio};
use super::report::{SketchContactTransition, SketchLineContact, SketchLinePosition};
use super::root_sign::sign_at_root;
use super::sector::{ContactDirection, sector_position};
use super::source_frame::SourceFrame;
use super::spans::HomogeneousPoint;
use num_bigint::{BigInt, Sign};

/// Exact contact point as functions of the contact variable.
#[derive(Clone)]
pub(super) enum ContactPoint {
    /// `(x, y) / w` with `w > 0` at the contact.
    Homogeneous(HomogeneousPoint),
    /// `C + s sqrt(c / Q(d(K))) d(K)` on concentric conics.
    Radial(RadialPoint),
}
/// Concentric contact: direction `d(K)`, scale `sqrt(constant / form(d))`, half turn sign.
#[derive(Clone)]
pub(super) struct RadialPoint {
    pub center: [ExactRatio; 2],
    pub direction: [RationalPolynomial; 2],
    pub form: RationalPolynomial,
    pub constant: ExactRatio,
    pub positive: bool,
}
/// A certified conic-source contact awaiting its target classification.
pub(super) struct ChartContact {
    pub contact: IsolatedRoot,
    pub sides: [Option<Sign>; 2],
    pub multiplicity: [Option<usize>; 2],
    pub transition: Option<SketchContactTransition>,
    pub point: ContactPoint,
    pub place: SourcePlace,
}
impl ChartContact {
    /// A regular contact whose transition follows its equation sides.
    pub fn regular(root: PieceRoot, point: ContactPoint, place: SourcePlace) -> Self {
        Self {
            contact: root.contact,
            sides: root.sides,
            multiplicity: root.multiplicity,
            transition: None,
            point,
            place,
        }
    }
    /// Classify against the target sector and construct; None outside the sector.
    pub fn finish(
        mut self,
        pair: &ConicPair<'_>,
        frame: &SourceFrame,
        work: &mut Work<'_>,
    ) -> Result<Option<SketchLineContact>, Cause> {
        work.enter(SketchIncidenceStage::Classification);
        let position = match pair.sector {
            None => SketchLinePosition::Interior,
            Some(sector) => {
                let relative = self.point.relative(&pair.target.center);
                let mut direction = ContactDirection {
                    root: &mut self.contact,
                    relative,
                };
                match sector_position(&mut direction, sector, work)? {
                    Some(position) => position,
                    None => return Ok(None),
                }
            }
        };
        let transition = match (position, self.transition) {
            (SketchLinePosition::Interior, Some(fixed)) => fixed,
            (SketchLinePosition::Interior, None) => {
                transition(self.sides, SketchContactTransition::TargetEnd)?
            }
            _ => SketchContactTransition::TargetEnd,
        };
        let envelope = self.point.construct(&mut self.contact, work)?;
        // A key that is the contact root itself keeps the refined bracket.
        if let SourcePlace::Sweep {
            cotangent: Some(KeyValue::Root(key)),
            ..
        } = &mut self.place
            && std::sync::Arc::ptr_eq(&key.equation, &self.contact.equation)
        {
            *key = self.contact.clone();
        }
        let draft = ContactDraft {
            place: self.place,
            target_parameter: None,
            target_position: position,
            transition,
            multiplicity: self.multiplicity,
            envelope,
        };
        finish(draft, pair.owner, frame, work).map(Some)
    }
}
impl ContactPoint {
    /// Direction from `center`, scaled by a positive factor.
    fn relative(&self, center: &[ExactRatio; 2]) -> [RationalPolynomial; 2] {
        match self {
            Self::Homogeneous(point) => point.relative(center),
            // Concentric: the direction from either center is `s d(K)`.
            Self::Radial(radial) => {
                let sign = ratio(if radial.positive { 1. } else { -1. });
                radial
                    .direction
                    .clone()
                    .map(|coordinate| coordinate.scale(&sign))
            }
        }
    }
    fn construct(
        &self,
        contact: &mut IsolatedRoot,
        work: &mut Work<'_>,
    ) -> Result<WorldEnvelope, Cause> {
        match self {
            Self::Homogeneous(point) => construct_point(point, contact, work),
            Self::Radial(radial) => {
                work.enter(SketchIncidenceStage::Construction);
                let limit = WorldLimit::new(work.world_error());
                limit.construct(contact, work, &|root: &IsolatedRoot| radial.enclose(root))
            }
        }
    }
}
impl RadialPoint {
    fn enclose(&self, root: &IsolatedRoot) -> Option<[RatioInterval; 2]> {
        let [lower, upper] = root.ratio_bounds();
        let key = RatioInterval { lower, upper };
        let form = self.form.enclose(&key).reciprocal()?;
        let squared = form.multiply(&RatioInterval::point(self.constant.clone()));
        let precision = precision_bits(&key);
        let mut scale = square_root(&squared, precision)?;
        if !self.positive {
            scale = RatioInterval {
                lower: scale.upper.negated(),
                upper: scale.lower.negated(),
            };
        }
        Some(std::array::from_fn(|axis| {
            let offset = self.direction[axis].enclose(&key).multiply(&scale);
            offset.add(&RatioInterval::point(self.center[axis].clone()))
        }))
    }
}
// Square-root precision follows the key bracket; an exact key needs enough bits to
// resolve any binary64 error limit.
fn precision_bits(key: &RatioInterval) -> u64 {
    let width = key.upper.subtract(&key.lower);
    if width.sign() == std::cmp::Ordering::Equal {
        return 2200;
    }
    let inverse = width
        .denominator()
        .bits()
        .saturating_sub(width.numerator().bits());
    64 + 2 * inverse
}
/// Outward enclosure of `sqrt` over a nonnegative interval with `bits` fraction bits.
pub(super) fn square_root(value: &RatioInterval, bits: u64) -> Option<RatioInterval> {
    if value.lower.sign() == std::cmp::Ordering::Less {
        return None;
    }
    let scale = BigInt::from(1) << (2 * bits);
    let floor =
        |ratio: &ExactRatio| -> BigInt { (ratio.numerator() * &scale) / ratio.denominator() };
    let lower: BigInt = floor(&value.lower).sqrt();
    let high: BigInt = floor(&value.upper) + 1;
    let mut upper = high.sqrt();
    if &upper * &upper < high {
        upper += 1;
    }
    let denominator = BigInt::from(1) << bits;
    Some(RatioInterval {
        lower: ExactRatio::new(lower, denominator.clone()),
        upper: ExactRatio::new(upper, denominator),
    })
}
/// Equation sides and multiplicities shared by both points, when known.
pub(super) type KnownSides = ([Option<Sign>; 2], [Option<usize>; 2]);
/// The two points `C +- sqrt(rho) d` with `rho = c_S / Q_S(d)`, both on the target.
///
/// `sides` gives their common equation sides when known; otherwise each point must
/// be a transversal crossing, certified by the exact sign of `det(grad F_S, grad F_T)`.
pub(super) fn quadratic_contacts(
    pair: &ConicPair<'_>,
    direction: &[ExactRatio; 2],
    places: [SourcePlace; 2],
    sides: Option<KnownSides>,
    work: &mut Work<'_>,
) -> Result<Vec<ChartContact>, Cause> {
    let source = &pair.source.implicit;
    let squared = source.constant.divide(&source.form.value(direction));
    let root = scale_root(&squared, work)?;
    let mut contacts = Vec::with_capacity(2);
    for (positive, place) in [true, false].into_iter().zip(places) {
        let sign = ratio(if positive { 1. } else { -1. });
        let mut contact = root.clone();
        let (sides, multiplicity, transition) = match sides {
            Some((sides, multiplicity)) => (sides, multiplicity, None),
            None => {
                transversal(pair, direction, &squared, &sign, &mut contact, work)?;
                (
                    [None; 2],
                    [Some(1); 2],
                    Some(SketchContactTransition::Crossing),
                )
            }
        };
        let point: [RationalPolynomial; 2] = std::array::from_fn(|axis| {
            RationalPolynomial::linear(source.center[axis].clone(), direction[axis].multiply(&sign))
        });
        contacts.push(ChartContact {
            contact,
            sides,
            multiplicity,
            transition,
            point: ContactPoint::Homogeneous(HomogeneousPoint {
                x: point[0].clone(),
                y: point[1].clone(),
                w: RationalPolynomial::constant(ratio(1.)),
            }),
            place,
        });
    }
    Ok(contacts)
}
// `X = sqrt(rho)` as the positive root of the integer polynomial `den X^2 - num`.
fn scale_root(squared: &ExactRatio, work: &mut Work<'_>) -> Result<IsolatedRoot, Cause> {
    let polynomial = RationalPolynomial(vec![squared.negated(), ratio(0.), ratio(1.)])
        .to_integer(work.max_degree())?
        .ok_or(Cause::Invariant("scale equation vanished"))?;
    let [_, upper] = cauchy_bounds(&polynomial);
    let (equation, isolation) = isolate(&polynomial, [super::pieces::dyadic(0.), upper], work)?;
    let bracket = isolation
        .roots()
        .iter()
        .find(|bracket| bracket.bounds().1 > &super::pieces::dyadic(0.))
        .ok_or(Cause::Invariant("positive scale root missing"))?;
    Ok(IsolatedRoot::from_bracket(&equation, bracket))
}
// `det(M_S d, M_T (Delta + s X d)) s X`, reduced with `X^2 = rho`, must not vanish.
fn transversal(
    pair: &ConicPair<'_>,
    direction: &[ExactRatio; 2],
    squared: &ExactRatio,
    sign: &ExactRatio,
    contact: &mut IsolatedRoot,
    work: &mut Work<'_>,
) -> Result<(), Cause> {
    let offset: [ExactRatio; 2] = std::array::from_fn(|axis| {
        pair.source.implicit.center[axis].subtract(&pair.target.center[axis])
    });
    let source_normal = pair.source.implicit.form.apply(direction);
    let determinant = |vector: [ExactRatio; 2]| {
        source_normal[0]
            .multiply(&vector[1])
            .subtract(&source_normal[1].multiply(&vector[0]))
    };
    let along = determinant(pair.target.form.apply(&offset)).multiply(sign);
    let across = determinant(pair.target.form.apply(direction)).multiply(squared);
    let value = RationalPolynomial::linear(across, along);
    if sign_at_root(contact, &value, work)? == Sign::NoSign {
        return Err(Cause::ExceptionalTangency);
    }
    Ok(())
}
