// SPDX-License-Identifier: GPL-2.0-only
//! Parametric sources (a line, spline spans, offset pieces) against implicit
//! targets (lines, conics, line offsets).
//!
//! The target equation composed with each source piece gives an exact equation in
//! the source parameter: a polynomial, or a surd for offset pieces (ADR 0080).
//! Roots are merged at source knots (and around a closed offset conic). The
//! parameter is the source place, except on an ellipse offset, which is placed by
//! its basis point. A line target positions each contact by exact signs of its own
//! parameter; a conic sector is tested by exact ray signs. Sides are taken along
//! the source, so the transition is the target side change along it.
use super::algebraic::IsolatedRoot;
use super::budget::Work;
use super::contact::{ContactDraft, TargetKey, construct_point, finish};
use super::coordinate::Coordinate;
use super::equation::transition;
use super::failure::{SketchIncidenceCause as Cause, SketchIncidenceStage};
use super::implicit::{ConicImplicit, LineImplicit};
use super::implicit_curve::{ImplicitCurve, OffsetLineImplicit};
use super::pieces::{MergedRoots, PieceRoot};
use super::place::{KeyValue, SourcePlace};
use super::rational_parameter::RationalParameter;
use super::report::{
    SketchContactTransition, SketchLineContact, SketchLinePosition, TargetParameter,
};
use super::sector::{ArcBounds, ContactDirection, sector_position};
use super::source_frame::SourceFrame;
use super::spans::ParametricSpan;
use super::surd_roots::surd_piece_roots;
use super::sweep::PointPlacer;

/// A target with an exact implicit equation.
pub(super) enum ImplicitTarget<'a> {
    Line {
        implicit: LineImplicit,
        bounded: bool,
    },
    Conic {
        implicit: ConicImplicit,
        sector: Option<ArcBounds<'a>>,
    },
    OffsetLine {
        implicit: OffsetLineImplicit,
        bounded: bool,
    },
}
/// Target facts of one contact; None when the contact lies outside a bounded target.
struct TargetFacts<T> {
    position: SketchLinePosition,
    parameter: Option<RationalParameter<T>>,
}
pub(super) fn incidences<T: Coordinate>(
    pieces: &[ParametricSpan<T>],
    target: &ImplicitTarget<'_>,
    owner: TargetKey,
    source: &SourceFrame,
    work: &mut Work<'_>,
) -> Result<Vec<SketchLineContact>, Cause> {
    let (merged, placers) = piece_roots(pieces, target, source, work)?;
    let mut contacts = Vec::new();
    for root in merged.roots {
        let request = ContactRequest {
            piece: &pieces[root.piece],
            placer: placers[root.piece].as_ref(),
            target,
            owner,
        };
        if let Some(contact) = request.contact(root, source, work)? {
            contacts.push(contact);
        }
    }
    Ok(contacts)
}
// Roots of every source piece, merged at shared ends, with each piece's placer.
fn piece_roots<T: Coordinate>(
    pieces: &[ParametricSpan<T>],
    target: &ImplicitTarget<'_>,
    source: &SourceFrame,
    work: &mut Work<'_>,
) -> Result<(MergedRoots, Vec<Option<PointPlacer>>), Cause> {
    let curve = target.curve();
    let mut merged = MergedRoots::default();
    let mut placers = Vec::with_capacity(pieces.len());
    for (index, piece) in pieces.iter().enumerate() {
        work.enter(SketchIncidenceStage::Equation);
        work.check()?;
        let equation = curve.equation(&piece.point);
        let roots = surd_piece_roots(&equation, piece.range, index, work)?
            .ok_or(Cause::UncertifiedOverlap { range: piece.range })?;
        placers.push(source.piece_placer(index, roots.first(), work)?);
        merged.add_roots(roots);
    }
    if source.cyclic_pieces() {
        merged.close_cycle();
    }
    Ok((merged, placers))
}
impl ImplicitTarget<'_> {
    fn curve(&self) -> ImplicitCurve {
        match self {
            Self::Line { implicit, .. } => ImplicitCurve::Line(implicit.clone()),
            Self::Conic { implicit, .. } => ImplicitCurve::Conic(implicit.clone()),
            Self::OffsetLine { implicit, .. } => ImplicitCurve::OffsetLine(implicit.clone()),
        }
    }
}
struct ContactRequest<'a, T> {
    piece: &'a ParametricSpan<T>,
    placer: Option<&'a PointPlacer>,
    target: &'a ImplicitTarget<'a>,
    owner: TargetKey,
}
impl<T: Coordinate> ContactRequest<'_, T> {
    fn contact(
        &self,
        mut root: PieceRoot,
        source: &SourceFrame,
        work: &mut Work<'_>,
    ) -> Result<Option<SketchLineContact>, Cause> {
        work.enter(SketchIncidenceStage::Classification);
        let Some(facts) = self.target_facts(&mut root.contact, work)? else {
            return Ok(None);
        };
        let transition = if facts.position == SketchLinePosition::Interior {
            transition(root.sides, SketchContactTransition::SourceEnd)?
        } else {
            SketchContactTransition::TargetEnd
        };
        let envelope = construct_point(&self.piece.point, &mut root.contact, work)?;
        let parameter = facts
            .parameter
            .as_ref()
            .and_then(|parameter| parameter.bounds(&root.contact));
        let place = match self.placer {
            Some(placer) => placer.place(&mut root.contact, work)?,
            None => SourcePlace::Parameter(KeyValue::Root(root.contact)),
        };
        let draft = ContactDraft {
            place,
            target_parameter: parameter.map(TargetParameter::Bounds),
            target_position: facts.position,
            transition,
            multiplicity: root.multiplicity,
            envelope,
        };
        finish(draft, self.owner, source, work).map(Some)
    }
    fn target_facts(
        &self,
        contact: &mut IsolatedRoot,
        work: &mut Work<'_>,
    ) -> Result<Option<TargetFacts<T>>, Cause> {
        match self.target {
            ImplicitTarget::Line { implicit, bounded } => {
                self.line_facts(implicit, *bounded, contact, work)
            }
            ImplicitTarget::OffsetLine { implicit, bounded } => {
                self.line_facts(&implicit.line, *bounded, contact, work)
            }
            ImplicitTarget::Conic { implicit, sector } => {
                let position = match sector {
                    None => Some(SketchLinePosition::Interior),
                    Some(sector) => {
                        let relative = self.piece.point.relative(&implicit.center);
                        let mut direction = ContactDirection {
                            root: contact,
                            relative,
                        };
                        sector_position(&mut direction, *sector, work)?
                    }
                };
                Ok(position.map(|position| TargetFacts {
                    position,
                    parameter: None,
                }))
            }
        }
    }
    // The target parameter `u = d.(p - A) / |d|^2` is positioned by exact signs; a
    // line offset shares it, since its offset is orthogonal to `d`.
    fn line_facts(
        &self,
        implicit: &LineImplicit,
        bounded: bool,
        contact: &mut IsolatedRoot,
        work: &mut Work<'_>,
    ) -> Result<Option<TargetFacts<T>>, Cause> {
        let parameter = RationalParameter {
            numerator: implicit.along(&self.piece.point),
            denominator: self.piece.point.w.scale(&implicit.squared_length()),
        };
        let position = match bounded {
            true => parameter.position(contact, [0., 1.], work)?,
            false => SketchLinePosition::Interior,
        };
        let facts = TargetFacts {
            position,
            parameter: Some(parameter),
        };
        Ok(facts.position.within_closed().then_some(facts))
    }
}
