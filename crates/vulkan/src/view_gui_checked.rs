//! Checked publication retains primary faults and refuses obsolete GUI targets.
use super::*;
use crate::RenderGuiSurfacePhase::{Capture, Publication};
use crate::error::gui_native_failure::reconcile_native_target;
use crate::{RenderGuiSurfaceChange, RenderGuiSurfacePhase, RenderGuiSurfacePolicy};
use crate::{gui_target_stamp::GuiTargetStamp, view_gui_layer::GuiLayerSlot};
use AspectNativeWindowOperation::Redraw;
struct GuiPublicationObservation {
    target: Result<(), RenderError>,
    extent: Result<Option<[u32; 2]>, RenderError>,
}
impl Graphic3dRenderView {
    pub(crate) fn checked_gui_policy(&self) -> bool {
        let selected = self.gui_surface;
        selected == RenderGuiSurfacePolicy::CheckedV2
    }
    pub(super) fn checked_gui_redraw(&self, stamp: GuiTargetStamp) -> Result<(), RenderError> {
        let outcome = self.render_current();
        self.publish_checked_gui(stamp, outcome)
    }
    pub(super) fn publish_checked_gui(
        &self,
        stamp: GuiTargetStamp,
        outcome: Result<FrameCapture, RenderError>,
    ) -> Result<(), RenderError> {
        let observation = self.gui_publication_observation(stamp.extent);
        let outcome = reconcile_native_target(outcome, observation.target);
        let observed = observation.extent;
        let mut slot = self.gui_layers.write();
        slot.retain_observed_change(&stamp, observed_target_change(&outcome, &observed, &stamp));
        let outcome = reconcile_observation(outcome, observed, &stamp, &slot, Publication);
        let outcome = self.guard_gui_redraw(outcome);
        slot.last_draw = Some(stamp);
        let result = outcome.as_ref().map(|_| ()).map_err(Clone::clone);
        *self.outcome.write() = Some(outcome);
        self.update_gui_redraw_status(result.is_ok());
        result
    }
    fn gui_publication_observation(&self, extent: Option<[u32; 2]>) -> GuiPublicationObservation {
        let target = self.require_native_target(Redraw);
        let observed = if target.is_ok() {
            self.checked_observed_extent(extent)
        } else {
            Ok(None)
        };
        GuiPublicationObservation {
            target,
            extent: observed,
        }
    }
    fn guard_gui_redraw(
        &self,
        outcome: Result<FrameCapture, RenderError>,
    ) -> Result<FrameCapture, RenderError> {
        let target = self.require_native_target(Redraw);
        reconcile_native_target(outcome, target)
    }
    pub(super) fn checked_gui_capture(&self) -> Result<FrameCapture, RenderError> {
        let outcome = self.recorded_gui_outcome();
        if let Err(target) = self.require_capture_target() {
            let refused = reconcile_native_target(outcome, Err(target));
            *self.outcome.write() = Some(refused.clone());
            return refused;
        }
        self.capture_current_gui(outcome)
    }
    fn recorded_gui_outcome(&self) -> Result<FrameCapture, RenderError> {
        self.outcome
            .read()
            .clone()
            .unwrap_or_else(|| Err(rejected("no redraw", "a successfully rendered view")))
    }
    fn capture_current_gui(
        &self,
        outcome: Result<FrameCapture, RenderError>,
    ) -> Result<FrameCapture, RenderError> {
        let stamp = self.gui_layers.read().capture_stamp();
        let observed = self.checked_observed_extent(stamp.extent);
        let mut slot = self.gui_layers.write();
        slot.retain_observed_change(&stamp, observed_target_change(&outcome, &observed, &stamp));
        let outcome = reconcile_observation(outcome, observed, &stamp, &slot, Capture);
        let outcome = reconcile_native_target(outcome, self.require_capture_target());
        *self.outcome.write() = Some(outcome.clone());
        outcome
    }
    fn checked_observed_extent(
        &self,
        accepted: Option<[u32; 2]>,
    ) -> Result<Option<[u32; 2]>, RenderError> {
        let Some(_) = accepted else {
            return Ok(None);
        };
        let logical = capture_extent(self, self.limit)?;
        let window = self.window();
        if !self.fbo().is_null() || window.is_virtual() || !window.is_mapped() {
            return Ok(Some(logical));
        }
        let presenter = self.presenter.read();
        match presenter.as_ref() {
            Some(presenter) => presenter.observed_extent().map(Some),
            None => Ok(Some(logical)),
        }
    }
    pub(crate) fn checked_gui_snapshot(
        &self,
        extent: [u32; 2],
    ) -> Result<Option<crate::view_gui_layer::GuiLayerPacket>, RenderError> {
        let slot = self.gui_layers.read();
        if let Some(change) =
            slot.target_stamp()
                .observe(&slot, Some(extent), RenderGuiSurfacePhase::FrameStart)
        {
            return Err(RenderError::surface_changed(change));
        }
        slot.snapshot(extent)
    }
    pub(crate) fn invalidate_checked_gui(&self, slot: &GuiLayerSlot) {
        let stamp = slot
            .last_draw
            .clone()
            .unwrap_or_else(|| slot.target_stamp());
        let extent = capture_extent(self, self.limit).ok();
        let change = stamp
            .observe(slot, extent, RenderGuiSurfacePhase::FrameStart)
            .unwrap_or_else(|| composition_change(slot, extent));
        let mut stored = self.outcome.write();
        let failure = match stored.take() {
            Some(Err(fault)) => fault.with_surface_change(change),
            _ => RenderError::surface_changed(change),
        };
        *stored = Some(Err(failure));
        self.retained.invalidate();
    }
}
fn reconcile_observation<T>(
    outcome: Result<T, RenderError>,
    observed: Result<Option<[u32; 2]>, RenderError>,
    stamp: &GuiTargetStamp,
    slot: &GuiLayerSlot,
    phase: RenderGuiSurfacePhase,
) -> Result<T, RenderError> {
    match observed {
        Ok(extent) => match stamp.observe(slot, extent, phase) {
            Some(change) => reject_gui_change(outcome, change),
            None => outcome,
        },
        Err(observation_fault) => crate::error::gui_observed_failure::combine_observed_result(
            outcome,
            Err(observation_fault),
        ),
    }
}
pub(crate) fn reject_gui_change<T>(
    outcome: Result<T, RenderError>,
    change: RenderGuiSurfaceChange,
) -> Result<T, RenderError> {
    let failure = match outcome {
        Ok(_) => RenderError::surface_changed(change),
        Err(fault) => fault.with_surface_change(change),
    };
    Err(failure)
}
fn composition_change(slot: &GuiLayerSlot, extent: Option<[u32; 2]>) -> RenderGuiSurfaceChange {
    RenderGuiSurfaceChange {
        phase: RenderGuiSurfacePhase::FrameStart,
        cause: crate::RenderGuiSurfaceCause::CompositionRevision,
        accepted_extent: slot.packet.as_ref().map_or([0, 0], |packet| packet.extent),
        observed_extent: extent,
        target_revision_changed: false,
        composition_revision_changed: true,
        fresh_packet: slot.fresh_packet_state(extent),
    }
}

fn observed_target_change<T>(
    outcome: &Result<T, RenderError>,
    observed: &Result<Option<[u32; 2]>, RenderError>,
    stamp: &GuiTargetStamp,
) -> bool {
    if let Ok(Some(extent)) = observed
        && stamp.extent.is_some_and(|accepted| accepted != *extent)
    {
        return true;
    }
    outcome.as_ref().err().is_some_and(is_native_surface_change)
}
fn is_native_surface_change(failure: &RenderError) -> bool {
    let Some(typed) = failure.gui_surface_outcome() else {
        return false;
    };
    let change = match typed {
        crate::RenderGuiSurfaceOutcome::SurfaceChanged(change)
        | crate::RenderGuiSurfaceOutcome::FaultWithSurfaceChange { change, .. } => change,
    };
    matches!(
        change.cause,
        crate::RenderGuiSurfaceCause::OutOfDate | crate::RenderGuiSurfaceCause::ExtentMismatch
    )
}
