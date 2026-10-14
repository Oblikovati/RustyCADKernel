// SPDX-License-Identifier: GPL-2.0-only
//! Whether a coincident target part meets the source part an edit would change.
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
/// Overlap range in increasing source order; None for an unbounded overlap.
fn sorted_range(
    overlap: &SketchLineOverlap,
    work: &mut Work<'_>,
) -> Result<Option<[SourcePlace; 2]>, Cause> {
    let Some([mut first, mut second]) = overlap.source_range.clone() else {
        return Ok(None);
    };
    Ok(Some(match compare_places(&mut first, &mut second, work)? {
        Ordering::Greater => [second, first],
        _ => [first, second],
    }))
}
fn less(first: &SourcePlace, second: &SourcePlace, work: &mut Work<'_>) -> Result<bool, Cause> {
    let order = compare_places(&mut first.clone(), &mut second.clone(), work)?;
    Ok(order == Ordering::Less)
}
impl TrimRegion<'_> {
    /// True when the overlap meets the open removed part.
    pub fn meets(&self, overlap: &SketchLineOverlap, work: &mut Work<'_>) -> Result<bool, Cause> {
        let Some([lower, upper]) = sorted_range(overlap, work)? else {
            return Ok(true);
        };
        let [start, end] = self.frame.ends();
        let from = self.before.clone().unwrap_or(start);
        let to = self.after.clone().unwrap_or(end);
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
        let Some([lower, upper]) = sorted_range(overlap, work)? else {
            return Ok(true);
        };
        match self.frame {
            SourceFrame::Conic(_) => self.blocked_on_conic(lower, upper, work),
            _ => self.blocked_on_line(lower, upper, work),
        }
    }
    // The support line continues away from the extended end.
    fn blocked_on_line(
        &self,
        lower: SourcePlace,
        upper: SourcePlace,
        work: &mut Work<'_>,
    ) -> Result<bool, Cause> {
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
    // An overlap that crosses the start ray in sweep order is reported as two
    // places with `lower` at the reference; it touches the start.
    fn blocked_on_conic(
        &self,
        lower: SourcePlace,
        upper: SourcePlace,
        work: &mut Work<'_>,
    ) -> Result<bool, Cause> {
        let [start, end] = self.ends;
        let touches_start = lower == *start;
        let reaches_gap = !less(&upper, end, work)?;
        match self.end {
            SketchLineEnd::End => {
                if !reaches_gap {
                    return Ok(false);
                }
                if !less(end, &lower, work)? {
                    return Ok(true);
                }
                self.candidate_not_before(&lower, true, work)
            }
            SketchLineEnd::Start => {
                if touches_start {
                    return Ok(true);
                }
                if !reaches_gap {
                    return Ok(false);
                }
                self.candidate_not_before(&upper, false, work)
            }
        }
    }
    // The candidate must lie strictly between the extended end and the overlap.
    fn candidate_not_before(
        &self,
        bound: &SourcePlace,
        forward: bool,
        work: &mut Work<'_>,
    ) -> Result<bool, Cause> {
        let Some(candidate) = &self.candidate else {
            return Ok(true);
        };
        let nearer = match forward {
            true => less(candidate, bound, work)?,
            false => less(bound, candidate, work)?,
        };
        Ok(!nearer)
    }
}
