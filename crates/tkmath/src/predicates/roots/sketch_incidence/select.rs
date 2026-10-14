// SPDX-License-Identifier: GPL-2.0-only
//! Certified edit-target selection over a complete report. Overlaps and exact
//! ties are never skipped silently: they refuse or are returned as whole groups.
use super::super::certificate_arithmetic::ExactRatio;
use super::algebraic::IsolatedRoot;
use super::budget::Work;
use super::failure::{SketchIncidenceCause as Cause, SketchIncidenceFailure, SketchIncidenceStage};
use super::input::SketchLineDomain;
use super::rational_polynomial::ratio;
use super::report::{SketchLineIncidenceReport, SketchLineOverlap, SketchLinePosition};
use super::source_line::SourceLine;
use std::cmp::Ordering;

/// Source line end to extend.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SketchLineEnd {
    /// Extend beyond `t = 0`.
    Start,
    /// Extend beyond `t = 1`.
    End,
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
    /// The pick does not project strictly inside the segment.
    PickOutsideSegment {
        /// Supplied pick point.
        pick: [f64; 2],
    },
    /// The pick projects exactly onto an incidence group.
    PickAtIncidence {
        /// Index into `ordered()`.
        group: usize,
    },
}
/// Selection failure: a refusal, or incomplete certification of a comparison.
#[derive(Clone, Debug, PartialEq)]
pub enum SketchSelectionError {
    /// Explicit domain, overlap or pick refusal.
    Refused(SketchSelectionRefusal),
    /// A required exact comparison did not complete.
    Incomplete(SketchIncidenceFailure),
}
/// Neighboring incidence groups around a trim pick; None means the segment end.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SketchTrimSelection {
    /// Last group strictly before the pick, as an index into `ordered()`.
    pub before: Option<usize>,
    /// First group strictly after the pick.
    pub after: Option<usize>,
}
impl SketchLineEnd {
    // Strictly past this end of the source segment.
    fn beyond(self, value: &ExactRatio) -> bool {
        match self {
            Self::End => *value > ratio(1.),
            Self::Start => *value < ratio(0.),
        }
    }
}
impl From<Cause> for SketchSelectionError {
    fn from(cause: Cause) -> Self {
        Self::Incomplete(SketchIncidenceFailure::new(
            SketchIncidenceStage::Selection,
            cause,
        ))
    }
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
        let candidate = self.nearest_beyond(end);
        for overlap in self.overlaps() {
            self.check_extension_overlap(overlap, end, candidate, &mut work)?;
        }
        Ok(candidate)
    }
    /// Incidence groups enclosing a pick projected exactly onto a segment-domain report.
    /// Example: `let around = report.select_trim([0.5, 0.], || false)?;`.
    pub fn select_trim(
        &self,
        pick: [f64; 2],
        mut cancelled: impl FnMut() -> bool,
    ) -> Result<SketchTrimSelection, SketchSelectionError> {
        self.require_domain(SketchLineDomain::Segment)?;
        let refused =
            SketchSelectionError::Refused(SketchSelectionRefusal::PickOutsideSegment { pick });
        if !pick.iter().all(|value| value.is_finite()) {
            return Err(refused);
        }
        let parameter = SourceLine::new(self.source.points, self.options.max_world_error)
            .projected_parameter(pick);
        if parameter <= ratio(0.) || parameter >= ratio(1.) {
            return Err(refused);
        }
        let mut work = Work::new(self.options, &mut cancelled);
        let selection = self.neighbors(&parameter, &mut work)?;
        for overlap in self.overlaps() {
            self.check_trim_overlap(overlap, selection, &mut work)?;
        }
        Ok(selection)
    }
    fn require_domain(&self, required: SketchLineDomain) -> Result<(), SketchSelectionError> {
        if self.source.domain == required {
            return Ok(());
        }
        Err(SketchSelectionError::Refused(
            SketchSelectionRefusal::Domain {
                required,
                actual: self.source.domain,
            },
        ))
    }
    fn overlaps(&self) -> impl Iterator<Item = &SketchLineOverlap> {
        self.targets
            .iter()
            .flat_map(|target| target.overlaps.iter())
    }
    fn group_root(&self, group: usize) -> IsolatedRoot {
        self.contact(self.ordered[group].members[0])
            .line_root
            .clone()
    }
    fn nearest_beyond(&self, end: SketchLineEnd) -> Option<usize> {
        let position = |group: &usize| self.contact(self.ordered[*group].members[0]).line_position;
        let mut groups = 0..self.ordered.len();
        match end {
            SketchLineEnd::End => {
                groups.find(|group| position(group) == SketchLinePosition::AfterEnd)
            }
            SketchLineEnd::Start => groups
                .rev()
                .find(|group| position(group) == SketchLinePosition::BeforeStart),
        }
    }
    fn check_extension_overlap(
        &self,
        overlap: &SketchLineOverlap,
        end: SketchLineEnd,
        candidate: Option<usize>,
        work: &mut Work<'_>,
    ) -> Result<(), SketchSelectionError> {
        let refused = SketchSelectionError::Refused(SketchSelectionRefusal::Overlap {
            target_id: overlap.target_id,
        });
        let Some([lower, upper]) = sorted_range(overlap) else {
            return Err(refused);
        };
        let (near, far, wanted) = match end {
            SketchLineEnd::End => (lower, upper, Ordering::Less),
            SketchLineEnd::Start => (upper, lower, Ordering::Greater),
        };
        if !end.beyond(&far) {
            return Ok(());
        }
        if !end.beyond(&near) {
            return Err(refused);
        }
        // The candidate must be strictly nearer to the extended end than the overlap.
        match candidate {
            Some(group) if self.group_root(group).compare_ratio(&near, work)? == wanted => Ok(()),
            _ => Err(refused),
        }
    }
    fn neighbors(
        &self,
        parameter: &ExactRatio,
        work: &mut Work<'_>,
    ) -> Result<SketchTrimSelection, SketchSelectionError> {
        let mut selection = SketchTrimSelection {
            before: None,
            after: None,
        };
        for group in 0..self.ordered.len() {
            match self.group_root(group).compare_ratio(parameter, work)? {
                Ordering::Less => selection.before = Some(group),
                Ordering::Equal => {
                    return Err(SketchSelectionError::Refused(
                        SketchSelectionRefusal::PickAtIncidence { group },
                    ));
                }
                Ordering::Greater => {
                    selection.after = Some(group);
                    break;
                }
            }
        }
        Ok(selection)
    }
    // Refuse when a coincident part meets the open line interval removed by the trim.
    fn check_trim_overlap(
        &self,
        overlap: &SketchLineOverlap,
        selection: SketchTrimSelection,
        work: &mut Work<'_>,
    ) -> Result<(), SketchSelectionError> {
        let refused = SketchSelectionError::Refused(SketchSelectionRefusal::Overlap {
            target_id: overlap.target_id,
        });
        let Some([lower, upper]) = sorted_range(overlap) else {
            return Err(refused);
        };
        let below_upper = match selection.after {
            Some(group) => self.group_root(group).compare_ratio(&lower, work)? == Ordering::Greater,
            None => lower < ratio(1.),
        };
        let above_lower = match selection.before {
            Some(group) => self.group_root(group).compare_ratio(&upper, work)? == Ordering::Less,
            None => upper > ratio(0.),
        };
        if below_upper && above_lower {
            return Err(refused);
        }
        Ok(())
    }
}
fn sorted_range(overlap: &SketchLineOverlap) -> Option<[ExactRatio; 2]> {
    let [first, second] = overlap.line_range.clone()?;
    Some(if first <= second {
        [first, second]
    } else {
        [second, first]
    })
}
