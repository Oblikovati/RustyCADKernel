//! Typed GUI target observations keep actual renderer faults distinct (ADR 0028).
use crate::RenderError;

/// Select GUI publication semantics; `RenderGuiSurfacePolicy::CheckedV2` opts in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RenderGuiSurfacePolicy {
    /// Replay the original ordering, including legacy out-of-date presentation.
    #[default]
    LegacyV1,
    /// Refuse changed GUI targets while retaining any primary renderer failure.
    CheckedV2,
}
/// Identify where target change was first observed; `RenderGuiSurfacePhase::Capture`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum RenderGuiSurfacePhase {
    /// Packet admission found a different target extent.
    Attachment,
    /// Packet snapshot or invalidation occurred before rendering.
    FrameStart,
    /// Native surface acquisition observed change.
    Acquire,
    /// The acquired target changed before command submission.
    BeforeSubmit,
    /// Completed rendering reached presentation on a changed target.
    Present,
    /// Completed native work or view publication observed change.
    Publication,
    /// Reading a completed result observed target change.
    Capture,
}
/// State of the current packet after refusal; `RenderGuiFreshPacket::Required`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderGuiFreshPacket {
    /// Submit a fresh packet for the current target before rendering.
    Required,
    /// A newer or still current packet is installed; redraw is still required.
    AlreadyCurrent,
    /// No GUI packet is currently installed.
    Absent,
}
/// Positive evidence for refusal; `RenderGuiSurfaceCause::ExtentMismatch`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum RenderGuiSurfaceCause {
    /// An observed extent differs from the accepted packet extent.
    ExtentMismatch,
    /// Delivered resize or rebind changed the target lifetime token.
    TargetRevision,
    /// Packet replacement or clear changed the composition token.
    CompositionRevision,
    /// Vulkan reported OUT_OF_DATE; the replacement extent may be unknown.
    OutOfDate,
}
/// Immutable value describing observed change; inspect `change.accepted_extent`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderGuiSurfaceChange {
    /// First phase that observed this target change.
    pub phase: RenderGuiSurfacePhase,
    /// First positive reason for refusing the accepted target.
    pub cause: RenderGuiSurfaceCause,
    /// Accepted packet extent, or supplied packet extent when attachment is refused.
    pub accepted_extent: [u32; 2],
    /// None when WSI reports out-of-date without an observed replacement extent.
    pub observed_extent: Option<[u32; 2]>,
    /// Observed target lifetime changed independently of extent.
    pub target_revision_changed: bool,
    /// Packet composition was replaced or cleared independently of resize.
    pub composition_revision_changed: bool,
    /// Current packet state after the latest observation.
    pub fresh_packet: RenderGuiFreshPacket,
}
impl RenderGuiSurfaceChange {
    /// Identify this checked contract; `assert_eq!(change.behavior_version(), 2)`.
    pub fn behavior_version(&self) -> u32 {
        // ADR0028 pins interpretation independently of package release numbering.
        2
    }
    pub(crate) fn extent_mismatch(
        phase: RenderGuiSurfacePhase,
        accepted: [u32; 2],
        observed: [u32; 2],
    ) -> Self {
        Self {
            phase,
            cause: RenderGuiSurfaceCause::ExtentMismatch,
            accepted_extent: accepted,
            observed_extent: Some(observed),
            target_revision_changed: false,
            composition_revision_changed: false,
            fresh_packet: RenderGuiFreshPacket::Required,
        }
    }
    pub(crate) fn out_of_date(phase: RenderGuiSurfacePhase, accepted: [u32; 2]) -> Self {
        let mut change = Self::extent_mismatch(phase, accepted, accepted);
        change.observed_extent = None;
        change.cause = RenderGuiSurfaceCause::OutOfDate;
        change
    }
    pub(crate) fn merge_observation(&mut self, newer: Self) {
        self.observed_extent = newer.observed_extent.or(self.observed_extent);
        self.target_revision_changed |= newer.target_revision_changed;
        self.composition_revision_changed |= newer.composition_revision_changed;
        self.fresh_packet = newer.fresh_packet;
    }
}
/// Inspect refusal without treating a real fault as resize; `error.gui_surface_outcome()`.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum RenderGuiSurfaceOutcome<'a> {
    /// A target change alone refused this operation; no primary fault is hidden.
    SurfaceChanged(&'a RenderGuiSurfaceChange),
    /// The primary fault must propagate even though target change was also observed.
    FaultWithSurfaceChange {
        /// Fault-bearing error, possibly compound; follow `primary_failure()` for retained earlier failures.
        fault: &'a RenderError,
        /// Observed change accompanying that primary failure.
        change: &'a RenderGuiSurfaceChange,
    },
}
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct GuiSurfaceFailure {
    pub primary: Option<RenderError>,
    pub change: RenderGuiSurfaceChange,
}
