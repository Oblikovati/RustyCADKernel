// SPDX-License-Identifier: GPL-2.0-only
//! Parametric sources (a line, spline spans) against implicit targets (lines, conics).
//!
//! The target equation composed with each source piece gives an exact polynomial in
//! the source parameter, which is itself the source place. Roots are merged at
//! source knots. A line target positions each contact by exact signs of its own
//! rational parameter; a conic sector is tested by exact ray signs. Sides are taken
//! along the source, so the transition is the target side change along it.
use super::algebraic::IsolatedRoot;
use super::budget::Work;
use super::contact::{ContactDraft, TargetKey, construct_point, finish};
use super::equation::transition;
use super::failure::{SketchIncidenceCause as Cause, SketchIncidenceStage};
use super::implicit::{ConicImplicit, LineImplicit};

use super::pieces::{MergedRoots, PieceRoot, piece_roots};
use super::place::{KeyValue, SourcePlace};
use super::rational_parameter::RationalParameter;
use super::rational_polynomial::RationalPolynomial;
use super::report::{
    SketchContactTransition, SketchLineContact, SketchLinePosition, TargetParameter,
};
use super::sector::{ArcBounds, ContactDirection, sector_position};
use super::source_frame::SourceFrame;
use super::spans::ParametricSpan;

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
}
/// Target facts of one contact; None when the contact lies outside a bounded target.
struct TargetFacts {
    position: SketchLinePosition,
    parameter: Option<RationalParameter>,
}
pub(super) fn incidences(
    pieces: &[ParametricSpan],
    target: &ImplicitTarget<'_>,
    owner: TargetKey,
    source: &SourceFrame,
    work: &mut Work<'_>,
) -> Result<Vec<SketchLineContact>, Cause> {
    let mut merged = MergedRoots::default();
    for (index, piece) in pieces.iter().enumerate() {
        work.enter(SketchIncidenceStage::Equation);
        work.check()?;
        let equation = target.compose(piece);
        let roots = piece_roots(&equation, piece.range, index, work)?
            .ok_or(Cause::UncertifiedOverlap { range: piece.range })?;
        merged.add_roots(roots);
    }
    let mut contacts = Vec::new();
    for root in merged.roots {
        let piece = &pieces[root.piece];
        let request = ContactRequest {
            piece,
            target,
            owner,
        };
        if let Some(contact) = request.contact(root, source, work)? {
            contacts.push(contact);
        }
    }
    Ok(contacts)
}
impl ImplicitTarget<'_> {
    fn compose(&self, piece: &ParametricSpan) -> RationalPolynomial {
        match self {
            Self::Line { implicit, .. } => implicit.compose(&piece.point),
            Self::Conic { implicit, .. } => implicit.compose(&piece.point),
        }
    }
}
struct ContactRequest<'a> {
    piece: &'a ParametricSpan,
    target: &'a ImplicitTarget<'a>,
    owner: TargetKey,
}
impl ContactRequest<'_> {
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
        let draft = ContactDraft {
            place: SourcePlace::Parameter(KeyValue::Root(root.contact)),
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
    ) -> Result<Option<TargetFacts>, Cause> {
        match self.target {
            ImplicitTarget::Line { implicit, bounded } => {
                self.line_facts(implicit, *bounded, contact, work)
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
    // The target parameter `u = d.(p - A) / |d|^2` is positioned by exact signs.
    fn line_facts(
        &self,
        implicit: &LineImplicit,
        bounded: bool,
        contact: &mut IsolatedRoot,
        work: &mut Work<'_>,
    ) -> Result<Option<TargetFacts>, Cause> {
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
