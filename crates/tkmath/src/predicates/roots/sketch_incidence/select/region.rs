// SPDX-License-Identifier: GPL-2.0-only
//! Whether a coincident target part meets the source part an edit would change.
//!
//! Line and spline overlaps are parameter intervals. A conic overlap is the cyclic
//! arc of the target's sweep from its first to its last place in the source sense;
//! the parts an edit changes are cyclic arcs too, so both are compared cyclically.
use super::super::budget::Work;
use super::super::failure::SketchIncidenceCause as Cause;
use super::super::place::{SourcePlace, compare_places};
use super::super::report::SketchLineOverlap;
use super::super::source_frame::SourceFrame;
use super::SketchLineEnd;
use std::cmp::Ordering;

/// The open source part removed by a trim, between two groups or source ends.
pub(super) struct TrimRegion<'a> {
    pub frame: &'a SourceFrame,
    pub before: Option<SourcePlace>,
    pub after: Option<SourcePlace>,
}
/// The source part an extension would add, up to its candidate group.
pub(super) struct ExtensionCheck<'a> {
    pub frame: &'a SourceFrame,
    pub ends: &'a [SourcePlace; 2],
    pub end: SketchLineEnd,
    pub candidate: Option<SourcePlace>,
}
fn less(first: &SourcePlace, second: &SourcePlace, work: &mut Work<'_>) -> Result<bool, Cause> {
    let order = compare_places(&mut first.clone(), &mut second.clone(), work)?;
    Ok(order == Ordering::Less)
}
fn same(first: &SourcePlace, second: &SourcePlace, work: &mut Work<'_>) -> Result<bool, Cause> {
    let order = compare_places(&mut first.clone(), &mut second.clone(), work)?;
    Ok(order == Ordering::Equal)
}
fn at_most(first: &SourcePlace, second: &SourcePlace, work: &mut Work<'_>) -> Result<bool, Cause> {
    Ok(!less(second, first, work)?)
}
/// Overlap range in increasing source order; None for an unbounded overlap.
fn sorted_range(
    overlap: &SketchLineOverlap,
    work: &mut Work<'_>,
) -> Result<Option<[SourcePlace; 2]>, Cause> {
    let Some([first, second]) = overlap.source_range.clone() else {
        return Ok(None);
    };
    Ok(Some(match less(&second, &first, work)? {
        true => [second, first],
        false => [first, second],
    }))
}
/// `x` on the closed cyclic arc from `from` forward to `to`.
fn on_closed_arc(
    x: &SourcePlace,
    [from, to]: [&SourcePlace; 2],
    work: &mut Work<'_>,
) -> Result<bool, Cause> {
    let after_from = at_most(from, x, work)?;
    let before_to = at_most(x, to, work)?;
    if at_most(from, to, work)? {
        return Ok(after_from && before_to);
    }
    Ok(after_from || before_to)
}
/// `x` strictly inside the open cyclic arc from `from` forward to `to`.
fn on_open_arc(
    x: &SourcePlace,
    [from, to]: [&SourcePlace; 2],
    work: &mut Work<'_>,
) -> Result<bool, Cause> {
    let after_from = less(from, x, work)?;
    let before_to = less(x, to, work)?;
    if less(from, to, work)? {
        return Ok(after_from && before_to);
    }
    Ok(after_from || before_to)
}
impl TrimRegion<'_> {
    /// True when the overlap meets the open removed part.
    pub fn meets(&self, overlap: &SketchLineOverlap, work: &mut Work<'_>) -> Result<bool, Cause> {
        let [start, end] = self.frame.ends();
        let from = self.before.clone().unwrap_or(start);
        let to = self.after.clone().unwrap_or(end);
        if self.frame.conic().is_some() {
            let Some([first, last]) = &overlap.source_range else {
                return Ok(true);
            };
            // A closed arc meets an open one at a point inside it, or by starting in it.
            let inside = on_open_arc(first, [&from, &to], work)?;
            let entered = on_closed_arc(&from, [first, last], work)? && !same(&from, last, work)?;
            return Ok(inside || entered);
        }
        let Some([lower, upper]) = sorted_range(overlap, work)? else {
            return Ok(true);
        };
        let below_to = less(&lower, &to, work)?;
        let above_from = less(&from, &upper, work)?;
        // A cyclic region runs from `from` past the source end and on to `to`.
        if less(&to, &from, work)? {
            return Ok(below_to || above_from);
        }
        Ok(below_to && above_from)
    }
}
impl ExtensionCheck<'_> {
    /// True when the overlap lies in the added part, at or before the candidate.
    pub fn blocked(&self, overlap: &SketchLineOverlap, work: &mut Work<'_>) -> Result<bool, Cause> {
        if self.frame.conic().is_some() {
            return self.blocked_on_conic(overlap, work);
        }
        let Some([lower, upper]) = sorted_range(overlap, work)? else {
            return Ok(true);
        };
        let (near, far) = match self.end {
            SketchLineEnd::End => (lower, upper),
            SketchLineEnd::Start => (upper, lower),
        };
        if !self.beyond_line(&far, work)? {
            return Ok(false);
        }
        if !self.beyond_line(&near, work)? {
            return Ok(true);
        }
        // The candidate must be strictly nearer to the extended end than the overlap.
        let Some(candidate) = &self.candidate else {
            return Ok(true);
        };
        let nearer = match self.end {
            SketchLineEnd::End => less(candidate, &near, work)?,
            SketchLineEnd::Start => less(&near, candidate, work)?,
        };
        Ok(!nearer)
    }
    fn beyond_line(&self, place: &SourcePlace, work: &mut Work<'_>) -> Result<bool, Cause> {
        match self.end {
            SketchLineEnd::End => less(&self.ends[1], place, work),
            SketchLineEnd::Start => less(place, &self.ends[0], work),
        }
    }
    // An arc's support runs from its end ray through the gap back to its start ray.
    // The added part is the closed arc from the extended end to the candidate.
    fn blocked_on_conic(
        &self,
        overlap: &SketchLineOverlap,
        work: &mut Work<'_>,
    ) -> Result<bool, Cause> {
        let Some([first, last]) = &overlap.source_range else {
            return Ok(true);
        };
        let [start, end] = self.ends;
        let added = match (self.end, &self.candidate) {
            (SketchLineEnd::End, Some(candidate)) => [end, candidate],
            (SketchLineEnd::Start, Some(candidate)) => [candidate, start],
            (_, None) => [end, start],
        };
        let inside = on_closed_arc(first, added, work)?;
        Ok(inside || on_closed_arc(added[0], [first, last], work)?)
    }
}
