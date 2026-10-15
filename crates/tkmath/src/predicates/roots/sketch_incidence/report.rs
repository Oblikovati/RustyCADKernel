// SPDX-License-Identifier: GPL-2.0-only
//! Immutable owned results. Every listed contact is certified; nothing is partial.
use super::super::certificate_arithmetic::{ExactRatio, rounding};
use super::algebraic::IsolatedRoot;
use super::input::{SketchIncidenceOptions, SketchIncidenceSource, SketchIncidenceTarget};
use super::place::{KeyValue, SketchSweepChart, SourcePlace};

/// Exact position of a value relative to the start and end of a parameter range.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum SketchLinePosition {
    /// Strictly before the start.
    BeforeStart,
    /// Exactly at the start.
    AtStart,
    /// Strictly between start and end.
    Interior,
    /// Exactly at the end.
    AtEnd,
    /// Strictly after the end.
    AfterEnd,
}
impl SketchLinePosition {
    /// True inside the closed range; example: `position.within_closed()`.
    pub fn within_closed(self) -> bool {
        matches!(self, Self::AtStart | Self::Interior | Self::AtEnd)
    }
}
/// How the target meets the source line, from exact signs of the defining equation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum SketchContactTransition {
    /// The target passes from one side of the line to the other.
    Crossing,
    /// The target touches the line and returns to the same side (tangency).
    Touching,
    /// The contact is a bounded target end; only one side exists.
    TargetEnd,
    /// The contact is a bounded spline source end; only one source side exists.
    SourceEnd,
}
/// Place of a conic-source contact: chart around the reference ray and cotangent key.
///
/// The reference ray is the sector start (or the frame axis of a full conic) and
/// the key `-dot(r, q) / det(r, q)` increases with the sweep angle in each open
/// half turn. Example: `contact.source_sweep().map(|sweep| sweep.chart)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SketchSweepPlace {
    /// True on an angle-bounded arc when the place precedes the start angle in its
    /// frame, so it follows every other place in sweep order from the start.
    pub wrapped: bool,
    /// Chart of the contact direction.
    pub chart: SketchSweepChart,
    /// Outward bounds of the exact cotangent key inside the open half turns.
    pub cotangent_bounds: Option<[f64; 2]>,
}
/// Exact target parameter of a contact: an isolated root, or certified outward
/// bounds of a rational function of the contact root.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum TargetParameter {
    Root(IsolatedRoot),
    Bounds([f64; 2]),
}
/// One certified isolated incidence between the source support line and one target.
#[derive(Clone, Debug, PartialEq)]
pub struct SketchLineContact {
    pub(super) target_index: usize,
    pub(super) target_id: u64,
    pub(super) place: SourcePlace,
    pub(super) target_parameter: Option<TargetParameter>,
    pub(super) source_position: SketchLinePosition,
    pub(super) target_position: SketchLinePosition,
    pub(super) transition: SketchContactTransition,
    pub(super) multiplicity: [Option<usize>; 2],
    pub(super) point: [f64; 2],
    pub(super) world_bounds: [[f64; 2]; 2],
    pub(super) achieved_world_error: f64,
}
impl SketchLineContact {
    /// Owning target key; example: `contact.target_id()`.
    pub fn target_id(&self) -> u64 {
        self.target_id
    }
    /// Index of the owning target in the request; example: `report.targets()[contact.target_index()]`.
    pub fn target_index(&self) -> usize {
        self.target_index
    }
    /// Outward bounds of the exact source parameter: `t` of a line source or the
    /// parameter of a spline source; None for conic sources (see `source_sweep`).
    /// Example: `let [lower, upper] = contact.source_parameter_bounds().unwrap();`.
    pub fn source_parameter_bounds(&self) -> Option<[f64; 2]> {
        match &self.place {
            SourcePlace::Parameter(value) => Some(value.outward()),
            SourcePlace::Sweep { .. } => None,
        }
    }
    /// Sweep place on a conic source; None for line and spline sources.
    pub fn source_sweep(&self) -> Option<SketchSweepPlace> {
        match &self.place {
            SourcePlace::Sweep {
                turn,
                chart,
                cotangent,
            } => Some(SketchSweepPlace {
                wrapped: *turn,
                chart: *chart,
                cotangent_bounds: cotangent.as_ref().map(KeyValue::outward),
            }),
            SourcePlace::Parameter(_) => None,
        }
    }
    /// Outward bounds of the exact target parameter, for line and spline targets.
    /// Conic targets are classified by exact sector signs and carry no angle.
    pub fn target_parameter_bounds(&self) -> Option<[f64; 2]> {
        self.target_parameter
            .as_ref()
            .map(|parameter| match parameter {
                TargetParameter::Root(root) => KeyValue::Root(root.clone()).outward(),
                TargetParameter::Bounds(bounds) => *bounds,
            })
    }
    /// Exact position on the source relative to its ends: `t = 0` and `t = 1` of a
    /// line, the bounds of a spline, the sector rays of an arc (outside is `AfterEnd`,
    /// since the support continues to the start); a full conic is `Interior`.
    pub fn source_position(&self) -> SketchLinePosition {
        self.source_position
    }
    /// Exact position on a bounded target; full curves and unbounded lines are `Interior`.
    pub fn target_position(&self) -> SketchLinePosition {
        self.target_position
    }
    /// Crossing, touching (tangency), or a one-sided target or spline source end.
    pub fn transition(&self) -> SketchContactTransition {
        self.transition
    }
    /// Algebraic multiplicity of the defining equation on the left and right sides,
    /// along the curve whose parameter carries the equation (the target for line and
    /// conic sources against line and spline targets, the source otherwise).
    /// They differ only at spline knots; None marks a side beyond that curve's end.
    pub fn multiplicity(&self) -> [Option<usize>; 2] {
        self.multiplicity
    }
    /// Constructed binary64 point; example: `contact.point()`.
    pub fn point(&self) -> [f64; 2] {
        self.point
    }
    /// Per-axis lower/upper bounds containing the exact incidence point.
    pub fn world_bounds(&self) -> [[f64; 2]; 2] {
        self.world_bounds
    }
    /// Certified Euclidean bound from `point()` to the exact incidence point.
    pub fn achieved_world_error(&self) -> f64 {
        self.achieved_world_error
    }
}
/// A target part lying on the source support line, with exact end positions.
#[derive(Clone, Debug, PartialEq)]
pub struct SketchLineOverlap {
    pub(super) target_index: usize,
    pub(super) target_id: u64,
    pub(super) target_range: Option<[f64; 2]>,
    pub(super) source_range: Option<[SourcePlace; 2]>,
    pub(super) source_positions: Option<[SketchLinePosition; 2]>,
    pub(super) opposite: bool,
}
impl SketchLineOverlap {
    /// Owning target key.
    pub fn target_id(&self) -> u64 {
        self.target_id
    }
    /// Index of the owning target in the request.
    pub fn target_index(&self) -> usize {
        self.target_index
    }
    /// Exact target parameter range; None for an unbounded collinear line.
    pub fn target_range(&self) -> Option<[f64; 2]> {
        self.target_range
    }
    /// Outward bounds of the exact source parameters at the target range start and
    /// end; None for unbounded overlaps and for conic sources.
    pub fn source_parameter_range_bounds(&self) -> Option<[[f64; 2]; 2]> {
        match self.source_range.as_ref()? {
            [SourcePlace::Parameter(start), SourcePlace::Parameter(end)] => {
                Some([start.outward(), end.outward()])
            }
            _ => None,
        }
    }
    /// Exact source positions of the range start and end.
    pub fn source_positions(&self) -> Option<[SketchLinePosition; 2]> {
        self.source_positions
    }
    /// True when the target runs against the source direction.
    pub fn opposite(&self) -> bool {
        self.opposite
    }
}
/// All certified incidences of one target, in exact target order.
#[derive(Clone, Debug, PartialEq)]
pub struct SketchTargetIncidences {
    pub(super) target: SketchIncidenceTarget,
    pub(super) contacts: Vec<SketchLineContact>,
    pub(super) overlaps: Vec<SketchLineOverlap>,
}
impl SketchTargetIncidences {
    /// Owned copy of the request target.
    pub fn target(&self) -> &SketchIncidenceTarget {
        &self.target
    }
    /// Every isolated support-line contact; empty is a completed empty result.
    pub fn contacts(&self) -> &[SketchLineContact] {
        &self.contacts
    }
    /// Every coincident part.
    pub fn overlaps(&self) -> &[SketchLineOverlap] {
        &self.overlaps
    }
}
/// Contacts with exactly equal line parameters; member order is request order, not identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SketchContactGroup {
    pub(super) members: Vec<[usize; 2]>,
}
impl SketchContactGroup {
    /// `[target_index, contact_index]` pairs; example: `group.members()[0]`.
    pub fn members(&self) -> &[[usize; 2]] {
        &self.members
    }
}
/// Complete certified result for one source curve and every requested target.
#[derive(Clone, Debug, PartialEq)]
pub struct SketchLineIncidenceReport {
    pub(super) source: SketchIncidenceSource,
    pub(super) options: SketchIncidenceOptions,
    pub(super) targets: Vec<SketchTargetIncidences>,
    pub(super) ordered: Vec<SketchContactGroup>,
    pub(super) max_achieved_world_error: f64,
    pub(super) refinements: u32,
}
impl SketchLineIncidenceReport {
    /// Owned request source.
    pub fn source(&self) -> &SketchIncidenceSource {
        &self.source
    }
    /// Requested options, unchanged.
    pub fn options(&self) -> SketchIncidenceOptions {
        self.options
    }
    /// Per-target results in request order.
    pub fn targets(&self) -> &[SketchTargetIncidences] {
        &self.targets
    }
    /// Contacts inside the requested domain, in exact source order (line or spline
    /// parameter, or conic sweep from the reference ray).
    pub fn ordered(&self) -> &[SketchContactGroup] {
        &self.ordered
    }
    /// Contact addressed by a group member.
    pub fn contact(&self, member: [usize; 2]) -> &SketchLineContact {
        &self.targets[member[0]].contacts[member[1]]
    }
    /// Largest certified construction error over all contacts (0 when none).
    pub fn max_achieved_world_error(&self) -> f64 {
        self.max_achieved_world_error
    }
    /// Refinement work used after isolation, within `options().max_refinements`.
    pub fn refinements(&self) -> u32 {
        self.refinements
    }
}
// Values beyond the binary64 range get infinite outward bounds, which still contain them.
pub(super) fn outward_ratio(value: &ExactRatio) -> [f64; 2] {
    rounding::outward_binary64(value).unwrap_or([f64::NEG_INFINITY, f64::INFINITY])
}
