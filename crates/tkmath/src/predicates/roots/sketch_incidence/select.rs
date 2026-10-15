// SPDX-License-Identifier: GPL-2.0-only
//! Certified edit-target selection over a complete report. Overlaps and exact
//! ties are never skipped silently: they refuse or are returned as whole groups.
//!
//! Trim removes the open source part between the incidence groups around a pick.
//! A closed source (a full conic, or a spline whose ends are the same exact point)
//! is opened cyclically, as the application trims circles: it needs two distinct
//! cut points. Extension is defined for lines (their infinite support) and conic
//! arcs (their full conic: the part beyond the end continues to the start). No
//! spline extension rule is defined, so spline sources admit only the segment domain.
mod pick;
mod region;

use super::budget::Work;
use super::failure::{SketchIncidenceCause as Cause, SketchIncidenceFailure, SketchIncidenceStage};
use super::input::SketchLineDomain;
use super::place::{SourcePlace, compare_places};
use super::report::{SketchLineIncidenceReport, SketchLineOverlap, SketchLinePosition};
use super::source_frame::SourceFrame;
use std::cmp::Ordering;

/// Source end to extend.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SketchLineEnd {
    /// Extend beyond the start (`t = 0`, or the start ray of an arc).
    Start,
    /// Extend beyond the end (`t = 1`, or the end ray of an arc).
    End,
}
/// The trim pick: a point projected onto the source, or an exact source parameter.
///
/// Lines (and line offsets) project points orthogonally; conics (and circle
/// offsets) project them centrally (the curve point in the direction of the pick
/// from the center, as the application does). Splines and spline offsets take an
/// exact parameter, since no point projection rule is defined; an ellipse offset
/// has neither rule and refuses picks (`PickKind`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SketchTrimPick {
    /// A world point; admitted for line and conic sources.
    Point([f64; 2]),
    /// A spline source parameter inside its bounds.
    Parameter(f64),
}
/// Explicit refusal; the consumer must not choose a target from this report.
#[derive(Clone, Debug, PartialEq)]
pub enum SketchSelectionRefusal {
    /// The report domain does not serve this edit.
    Domain {
        /// Domain the edit requires.
        required: SketchLineDomain,
        /// Domain of the report.
        actual: SketchLineDomain,
    },
    /// A coincident target part lies in the edited range.
    Overlap {
        /// Target occurrence key of the overlap.
        target_id: u64,
    },
    /// The pick does not project strictly inside an open source, or lies outside
    /// the bounds of a closed spline.
    PickOutsideSegment {
        /// Supplied pick.
        pick: SketchTrimPick,
    },
    /// The pick projects exactly onto an incidence group.
    PickAtIncidence {
        /// Index into `ordered()`.
        group: usize,
    },
    /// The pick kind is not defined for this source representation.
    PickKind {
        /// Supplied pick.
        pick: SketchTrimPick,
    },
    /// A closed source opens only between two distinct incidence points.
    ClosedSourceCuts {
        /// Distinct incidence points found.
        found: usize,
    },
    /// The source has no ends to extend (a full conic).
    ClosedSource,
}
/// Selection failure: a refusal, or incomplete certification of a comparison.
#[derive(Clone, Debug, PartialEq)]
pub enum SketchSelectionError {
    /// Explicit domain, overlap or pick refusal.
    Refused(SketchSelectionRefusal),
    /// A required exact comparison did not complete.
    Incomplete(SketchIncidenceFailure),
}
/// Neighboring incidence groups around a trim pick; None means the source end.
///
/// On a closed source both are present and the removed part runs forward from
/// `before` to `after`, crossing the start when `after` precedes `before`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SketchTrimSelection {
    /// Last group before the pick, as an index into `ordered()`.
    pub before: Option<usize>,
    /// First group after the pick.
    pub after: Option<usize>,
}
impl From<Cause> for SketchSelectionError {
    fn from(cause: Cause) -> Self {
        Self::Incomplete(SketchIncidenceFailure::new(
            SketchIncidenceStage::Selection,
            cause,
        ))
    }
}
fn refused(refusal: SketchSelectionRefusal) -> SketchSelectionError {
    SketchSelectionError::Refused(refusal)
}
impl SketchLineIncidenceReport {
    /// Nearest incidence group strictly beyond an end of a support-domain report.
    ///
    /// `Ok(None)` is a completed empty selection: nothing lies beyond that end.
    /// Example: `let group = report.select_extension(SketchLineEnd::End, || false)?;`.
    pub fn select_extension(
        &self,
        end: SketchLineEnd,
        mut cancelled: impl FnMut() -> bool,
    ) -> Result<Option<usize>, SketchSelectionError> {
        self.require_domain(SketchLineDomain::Support)?;
        let mut work = Work::new(self.options, &mut cancelled);
        let frame = SourceFrame::new(&self.source.curve, &mut work)?;
        if frame.closed() {
            return Err(refused(SketchSelectionRefusal::ClosedSource));
        }
        let candidate = self.nearest_beyond(end, &frame);
        let ends = frame.ends();
        for overlap in self.overlaps() {
            let check = region::ExtensionCheck {
                frame: &frame,
                ends: &ends,
                end,
                candidate: candidate.map(|group| self.group_place(group)),
            };
            if check.blocked(overlap, &mut work)? {
                return Err(overlap_refusal(overlap));
            }
        }
        Ok(candidate)
    }
    /// Incidence groups enclosing a point pick on a segment-domain report.
    /// Example: `let around = report.select_trim([0.5, 0.], || false)?;`.
    pub fn select_trim(
        &self,
        pick: [f64; 2],
        cancelled: impl FnMut() -> bool,
    ) -> Result<SketchTrimSelection, SketchSelectionError> {
        self.select_trim_at(SketchTrimPick::Point(pick), cancelled)
    }
    /// Incidence groups enclosing a point or parameter pick on a segment-domain report.
    /// Example: `report.select_trim_at(SketchTrimPick::Parameter(0.5), || false)?;`.
    pub fn select_trim_at(
        &self,
        pick: SketchTrimPick,
        mut cancelled: impl FnMut() -> bool,
    ) -> Result<SketchTrimSelection, SketchSelectionError> {
        self.require_domain(SketchLineDomain::Segment)?;
        let mut work = Work::new(self.options, &mut cancelled);
        let frame = SourceFrame::new(&self.source.curve, &mut work)?;
        let mut place = pick::pick_place(&frame, pick, &mut work)?;
        let closed = frame.closed();
        if !closed {
            pick::require_inside(&frame, &mut place, pick, &mut work)?;
        }
        let mut selection = self.neighbors(&mut place, &mut work)?;
        if closed {
            selection = self.wrap(selection, &frame)?;
        }
        for overlap in self.overlaps() {
            let removed = region::TrimRegion {
                frame: &frame,
                before: selection.before.map(|group| self.group_place(group)),
                after: selection.after.map(|group| self.group_place(group)),
            };
            if removed.meets(overlap, &mut work)? {
                return Err(overlap_refusal(overlap));
            }
        }
        Ok(selection)
    }
    fn require_domain(&self, required: SketchLineDomain) -> Result<(), SketchSelectionError> {
        if self.source.domain == required {
            return Ok(());
        }
        Err(refused(SketchSelectionRefusal::Domain {
            required,
            actual: self.source.domain,
        }))
    }
    fn overlaps(&self) -> impl Iterator<Item = &SketchLineOverlap> {
        self.targets
            .iter()
            .flat_map(|target| target.overlaps.iter())
    }
    fn group_place(&self, group: usize) -> SourcePlace {
        self.contact(self.ordered[group].members[0]).place.clone()
    }
    // Lines extend away from each end; an arc's support continues from its end to
    // its start, so both extensions search the same gap from opposite sides.
    fn nearest_beyond(&self, end: SketchLineEnd, frame: &SourceFrame) -> Option<usize> {
        let position = |group: &usize| {
            self.contact(self.ordered[*group].members[0])
                .source_position
        };
        let mut groups = 0..self.ordered.len();
        let before_start = match frame.conic() {
            Some(_) => SketchLinePosition::AfterEnd,
            None => SketchLinePosition::BeforeStart,
        };
        match end {
            SketchLineEnd::End => {
                groups.find(|group| position(group) == SketchLinePosition::AfterEnd)
            }
            SketchLineEnd::Start => groups.rev().find(|group| position(group) == before_start),
        }
    }
    fn neighbors(
        &self,
        pick: &mut SourcePlace,
        work: &mut Work<'_>,
    ) -> Result<SketchTrimSelection, SketchSelectionError> {
        let mut selection = SketchTrimSelection {
            before: None,
            after: None,
        };
        for group in 0..self.ordered.len() {
            match compare_places(&mut self.group_place(group), pick, work)? {
                Ordering::Less => selection.before = Some(group),
                Ordering::Equal => {
                    return Err(refused(SketchSelectionRefusal::PickAtIncidence { group }));
                }
                Ordering::Greater => {
                    selection.after = Some(group);
                    break;
                }
            }
        }
        Ok(selection)
    }
    // A closed source opens between the cyclic neighbors of the pick.
    fn wrap(
        &self,
        selection: SketchTrimSelection,
        frame: &SourceFrame,
    ) -> Result<SketchTrimSelection, SketchSelectionError> {
        let found = self.distinct_points(frame);
        if found < 2 {
            return Err(refused(SketchSelectionRefusal::ClosedSourceCuts { found }));
        }
        let last = self.ordered.len() - 1;
        Ok(SketchTrimSelection {
            before: selection.before.or(Some(last)),
            after: selection.after.or(Some(0)),
        })
    }
    // A closed spline reports its seam point at both bounds; count it once.
    fn distinct_points(&self, frame: &SourceFrame) -> usize {
        let groups = self.ordered.len();
        let position = |group: usize| self.contact(self.ordered[group].members[0]).source_position;
        let spline_like = matches!(frame, SourceFrame::Spline(_) | SourceFrame::Offset(_));
        let seam_twice = spline_like
            && frame.conic().is_none()
            && groups >= 2
            && position(0) == SketchLinePosition::AtStart
            && position(groups - 1) == SketchLinePosition::AtEnd;
        groups - usize::from(seam_twice)
    }
}
fn overlap_refusal(overlap: &SketchLineOverlap) -> SketchSelectionError {
    refused(SketchSelectionRefusal::Overlap {
        target_id: overlap.target_id,
    })
}
