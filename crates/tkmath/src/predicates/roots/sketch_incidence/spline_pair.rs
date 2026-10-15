// SPDX-License-Identifier: GPL-2.0-only
//! Spline sources against spline targets, one pair of spans at a time.
//!
//! The coordinate equations `X_t(u) W_s(s) - X_s(s) W_t(u)` and the same in `y`
//! are eliminated in `u`. Their resultant `R(s)` is, up to a positive factor and a
//! constant sign, the target span's implicit equation along the source span, so it
//! carries the source sides of every contact. A root `s*` is a contact of the two
//! spans only through its fibre `u* = -beta/alpha` of the first subresultant, which
//! must be a single certified point inside the target span. A fibre with
//! `alpha(s*) = 0` (several target points, or a straight span of degree two or
//! more) is a named incomplete result. Contacts at a target interior knot are
//! classified by the exact cyclic order of the four half tangents there.
use super::super::certificate_arithmetic::RatioInterval;
use super::algebraic::IsolatedRoot;
use super::bivariate::{Bivariate, FibreRow, fibre_row, resultant};
use super::budget::Work;
use super::contact::{ContactDraft, TargetKey, construct_point, finish};
use super::equation::transition;
use super::failure::{SketchIncidenceCause as Cause, SketchIncidenceStage};
use super::pieces::{MergedRoots, PieceRoot, piece_roots};
use super::place::{KeyValue, SourcePlace};
use super::rational_parameter::RationalParameter;
use super::rational_polynomial::{RationalPolynomial, ratio};
use super::report::{
    SketchContactTransition, SketchLineContact, SketchLinePosition, TargetParameter,
};
use super::root_sign::sign_at_root;
use super::source_frame::{SourceFrame, SplineSource};
use super::spans::{HomogeneousPoint, ParametricSpan};
use super::spline_corner::{Corner, corner_transition};
use num_bigint::Sign;

/// One spline target against a spline source.
pub(super) struct SplinePair<'a> {
    pub source: &'a SplineSource,
    pub spans: &'a [ParametricSpan],
    pub bounds: [f64; 2],
    pub owner: TargetKey,
}
/// Exact elimination of one source span against one target span.
struct SpanSystem {
    resultant: RationalPolynomial,
    row: Option<FibreRow>,
    leading: [RationalPolynomial; 2],
    range: Option<[f64; 2]>,
}
pub(super) fn incidences(
    pair: &SplinePair<'_>,
    frame: &SourceFrame,
    work: &mut Work<'_>,
) -> Result<Vec<SketchLineContact>, Cause> {
    let mut contacts = Vec::new();
    for (index, span) in pair.spans.iter().enumerate() {
        let target = TargetSpan { pair, span, index };
        contacts.extend(target.contacts(frame, work)?);
    }
    Ok(contacts)
}
struct TargetSpan<'a> {
    pair: &'a SplinePair<'a>,
    span: &'a ParametricSpan,
    index: usize,
}
impl TargetSpan<'_> {
    fn contacts(
        &self,
        frame: &SourceFrame,
        work: &mut Work<'_>,
    ) -> Result<Vec<SketchLineContact>, Cause> {
        let mut merged = MergedRoots::default();
        let mut systems = Vec::with_capacity(self.pair.source.pieces.len());
        for (index, piece) in self.pair.source.pieces.iter().enumerate() {
            work.enter(SketchIncidenceStage::Equation);
            work.check()?;
            if separated(piece, self.span) {
                merged.add_roots(Vec::new());
                systems.push(None);
                continue;
            }
            let system = SpanSystem::new(piece, &self.span.point)?;
            let roots = piece_roots(&system.resultant, piece.range, index, work)?
                .ok_or(Cause::UncertifiedOverlap { range: piece.range })?;
            merged.add_roots(roots);
            systems.push(Some(system));
        }
        let mut contacts = Vec::new();
        for root in merged.roots {
            let system = systems[root.piece]
                .as_ref()
                .expect("roots come from computed spans");
            if let Some(contact) = self.contact(root, system, frame, work)? {
                contacts.push(contact);
            }
        }
        Ok(contacts)
    }
    fn contact(
        &self,
        mut root: PieceRoot,
        system: &SpanSystem,
        frame: &SourceFrame,
        work: &mut Work<'_>,
    ) -> Result<Option<SketchLineContact>, Cause> {
        work.enter(SketchIncidenceStage::Classification);
        let parameter = system.fibre(&mut root.contact, work)?;
        let range = self.span.range.expect("spline spans are bounded");
        let local = parameter.position(&mut root.contact, range, work)?;
        if !local.within_closed() || self.duplicate_knot(local, range) {
            return Ok(None);
        }
        let position = parameter.position(&mut root.contact, self.pair.bounds, work)?;
        let at_knot =
            position == SketchLinePosition::Interior && local == SketchLinePosition::AtEnd;
        let (transition, multiplicity) = match position {
            _ if at_knot => (self.corner(&root, range[1], work)?, [Some(1); 2]),
            SketchLinePosition::Interior => (
                transition(root.sides, SketchContactTransition::SourceEnd)?,
                root.multiplicity,
            ),
            _ => (SketchContactTransition::TargetEnd, root.multiplicity),
        };
        let piece = &self.pair.source.pieces[root.piece];
        let envelope = construct_point(&piece.point, &mut root.contact, work)?;
        let bounds = parameter.bounds(&root.contact);
        let draft = ContactDraft {
            place: SourcePlace::Parameter(KeyValue::Root(root.contact)),
            target_parameter: bounds.map(TargetParameter::Bounds),
            target_position: position,
            transition,
            multiplicity,
            envelope,
        };
        finish(draft, self.pair.owner, frame, work).map(Some)
    }
    // An interior target knot meets the next span; both pieces must be transversal.
    fn corner(
        &self,
        root: &PieceRoot,
        knot: f64,
        work: &mut Work<'_>,
    ) -> Result<SketchContactTransition, Cause> {
        let next = &self.pair.spans[self.index + 1];
        let corner = Corner {
            source: &self.pair.source.pieces,
            root,
            target: [&self.span.point, &next.point],
            knot,
        };
        corner_transition(corner, &mut root.contact.clone(), work)
    }
    // A contact at the start of a span that begins at an interior target knot is the
    // previous span's end contact; it is reported once, from there.
    fn duplicate_knot(&self, local: SketchLinePosition, range: [f64; 2]) -> bool {
        local == SketchLinePosition::AtStart && range[0] > self.pair.bounds[0]
    }
}
impl SpanSystem {
    fn new(source: &ParametricSpan, target: &HomogeneousPoint) -> Result<Self, Cause> {
        // Integer coefficients keep every sampled determinant integral.
        let (source_point, target) = (source.point.integral(), target.integral());
        let first = coordinate_equation(&source_point, &target, Axis::X);
        let second = coordinate_equation(&source_point, &target, Axis::Y);
        let degrees = [first.degree(), second.degree()];
        // A target span with constant coordinates is one point, not a curve.
        if degrees.iter().all(|degree| *degree == Some(0)) {
            return Err(Cause::ExceptionalFiber {
                range: source.range,
            });
        }
        let degrees = degrees.map(|degree| degree.unwrap_or(0));
        Ok(Self {
            resultant: resultant(&first, degrees, &second),
            row: fibre_row(&first, degrees, &second),
            leading: [first.leading(degrees[0]), second.leading(degrees[1])],
            range: source.range,
        })
    }
    /// The single certified target parameter of a root, or a named exceptional fibre.
    fn fibre(
        &self,
        contact: &mut IsolatedRoot,
        work: &mut Work<'_>,
    ) -> Result<RationalParameter, Cause> {
        let exceptional = Cause::ExceptionalFiber { range: self.range };
        let Some(row) = &self.row else {
            return Err(exceptional);
        };
        let first = sign_at_root(contact, &self.leading[0], work)?;
        let second = sign_at_root(contact, &self.leading[1], work)?;
        let alpha = sign_at_root(contact, &row.alpha, work)?;
        if alpha == Sign::NoSign || (first == Sign::NoSign && second == Sign::NoSign) {
            return Err(exceptional);
        }
        Ok(RationalParameter {
            numerator: row.beta.scale(&ratio(-1.)),
            denominator: row.alpha.clone(),
        })
    }
}
// A certified exclusion: spans in disjoint closed boxes share no point.
fn separated(first: &ParametricSpan, second: &ParametricSpan) -> bool {
    let (Some(one), Some(other)) = (span_box(first), span_box(second)) else {
        return false;
    };
    (0..2).any(|axis| one[axis].upper < other[axis].lower || other[axis].upper < one[axis].lower)
}
fn span_box(span: &ParametricSpan) -> Option<[RatioInterval; 2]> {
    span.point.hull_box(span.range?)
}
#[derive(Clone, Copy)]
enum Axis {
    X,
    Y,
}
// `T_axis(u) W_s(s) - S_axis(s) W_t(u)` as coefficients in `u`.
fn coordinate_equation(
    source: &HomogeneousPoint,
    target: &HomogeneousPoint,
    axis: Axis,
) -> Bivariate {
    let (own, other) = match axis {
        Axis::X => (&source.x, &target.x),
        Axis::Y => (&source.y, &target.y),
    };
    let length = other.0.len().max(target.w.0.len());
    Bivariate(
        (0..length)
            .map(|power| {
                let along = source.w.scale(&other.coefficient(power));
                let weight = own.scale(&target.w.coefficient(power).negated());
                along.add(&weight)
            })
            .collect(),
    )
}
