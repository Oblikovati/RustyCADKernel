// SPDX-License-Identifier: GPL-2.0-only
//! Whole parametric targets: span contacts merged at shared knots, plus overlaps.
use super::budget::Work;
use super::contact::{ContactDraft, TargetKey, finish};
use super::equation::transition;
use super::failure::SketchIncidenceCause as Cause;
use super::overlap::span_overlap;
use super::parametric::{SpanEquations, SpanRoot, span_roots};
use super::report::{SketchLineContact, SketchLineOverlap, SketchLinePosition};
use super::source_line::{SourceLine, position_between};
use super::spans::ParametricSpan;

pub(super) struct ParametricRequest {
    pub spans: Vec<ParametricSpan>,
    /// Bounded target parameter range; None for an unbounded line.
    pub bounds: Option<[f64; 2]>,
    pub owner: TargetKey,
}
#[derive(Default)]
struct MergedTarget {
    roots: Vec<SpanRoot>,
    overlaps: Vec<SketchLineOverlap>,
    previous_overlap: bool,
}
pub(super) fn incidences(
    request: ParametricRequest,
    source: &SourceLine,
    work: &mut Work<'_>,
) -> Result<(Vec<SketchLineContact>, Vec<SketchLineOverlap>), Cause> {
    let mut merged = MergedTarget::default();
    for span in &request.spans {
        let equations = SpanEquations::new(span, source);
        match span_roots(span, &equations, source, work)? {
            Some(roots) => merged.add_roots(roots),
            None => {
                merged.add_overlap(span_overlap(span, &equations, source, request.owner, work)?)
            }
        }
    }
    let contacts = merged
        .roots
        .into_iter()
        .map(|root| contact(root, &request, source, work))
        .collect::<Result<Vec<_>, _>>()?;
    Ok((contacts, merged.overlaps))
}
impl MergedTarget {
    // A root exactly at a span start is the same point as the previous span's end
    // root (C0 continuity). Its two sides come from the two adjacent spans.
    fn add_roots(&mut self, roots: Vec<SpanRoot>) {
        for root in roots {
            if root.sides[0].is_some() {
                self.roots.push(root);
            } else if self.previous_overlap {
                continue;
            } else if let Some(last) = self.roots.last_mut().filter(|last| last.sides[1].is_none())
            {
                last.sides[1] = root.sides[1];
                last.multiplicity[1] = root.multiplicity[1];
            } else {
                self.roots.push(root);
            }
        }
        self.previous_overlap = false;
    }
    // An overlap boundary is not an isolated contact.
    fn add_overlap(&mut self, overlap: SketchLineOverlap) {
        if self
            .roots
            .last()
            .is_some_and(|last| last.sides[1].is_none())
        {
            self.roots.pop();
        }
        self.overlaps.push(overlap);
        self.previous_overlap = true;
    }
}
fn contact(
    mut root: SpanRoot,
    request: &ParametricRequest,
    source: &SourceLine,
    work: &mut Work<'_>,
) -> Result<SketchLineContact, Cause> {
    let target_position = match request.bounds {
        Some(bounds) => position_between(&mut root.target, bounds, work)?,
        None => SketchLinePosition::Interior,
    };
    let one_sided = root.sides.iter().any(Option::is_none);
    if one_sided && target_position == SketchLinePosition::Interior {
        return Err(Cause::Invariant(
            "interior knot contact lacked its adjacent span",
        ));
    }
    let draft = ContactDraft {
        line_root: root.line,
        target_root: Some(root.target),
        target_position,
        transition: transition(root.sides)?,
        multiplicity: root.multiplicity,
    };
    finish(draft, request.owner, source, work)
}
