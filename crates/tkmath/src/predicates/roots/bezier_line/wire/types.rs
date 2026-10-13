// SPDX-License-Identifier: GPL-2.0-only
use super::super::super::RootIsolationOptions;
/// One directed occurrence; its ID belongs to the caller, not to a naming engine.
#[derive(Clone, Copy, Debug)]
pub struct BezierWireSegment<'a> {
    /// Unique occurrence ID within this query; repeated parent curves need distinct IDs.
    pub id: u64,
    /// Finite planar Bernstein controls.
    pub controls: &'a [[f64; 2]],
    /// Optional finite rational weights with one value per control.
    pub weights: Option<&'a [f64]>,
    /// Closed finite ordered parameter bounds; the basis stays fixed at `[0,1]`.
    pub interval: [f64; 2],
    /// Traverse from upper to lower parameter without changing the supplied curve.
    pub reversed: bool,
}
/// Ordered connected occurrences and one seed/ray frame.
#[derive(Clone, Copy, Debug)]
pub struct BezierWireQuery<'a> {
    /// Nonempty ordered path; its last endpoint must equal its first exactly.
    pub segments: &'a [BezierWireSegment<'a>],
    /// Origin seed followed by a distinct point defining the positive ray.
    pub line: [[f64; 2]; 2],
    /// Degree and width policy per segment; subdivision budget for the entire wire.
    pub options: RootIsolationOptions,
}
/// Seed location under the nonzero winding rule; no face validity is implied.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BezierWireLocation {
    /// No boundary incidence and zero winding number.
    Outside,
    /// Nonzero signed winding number; reversal changes its sign, not membership.
    Inside {
        /// Complete signed count over all directed occurrences.
        winding: i64,
    },
    /// Every incident occurrence ID, retained in input order after full validation.
    Boundary {
        /// Caller-owned IDs of all occurrences containing the origin.
        segment_ids: Vec<u64>,
    },
}
/// Complete exact-closure and location result with total subdivision work.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BezierWireResult {
    pub(super) location: BezierWireLocation,
    pub(super) bisections: u16,
}
impl BezierWireResult {
    /// Read the complete location; example: `result.location() == &BezierWireLocation::Outside`.
    pub fn location(&self) -> &BezierWireLocation {
        &self.location
    }
    /// Read total subdivisions across all occurrences; example: `result.bisections() <= 4096`.
    pub fn bisections(&self) -> u16 {
        self.bisections
    }
}
