//! Compound errors retain the first primary fault and first observed phase.
use super::{RenderError, RenderFailureCause};
use crate::{
    RenderGuiSurfaceChange, RenderGuiSurfaceOutcome, gui_surface_outcome::GuiSurfaceFailure,
};
impl RenderError {
    /// Distinguish recoverable target change from a real fault; `error.gui_surface_outcome()`.
    pub fn gui_surface_outcome(&self) -> Option<RenderGuiSurfaceOutcome<'_>> {
        let failure = match self.1.as_ref()? {
            RenderFailureCause::GuiSurface(failure) => failure,
            RenderFailureCause::WindowTarget(failure) => {
                return self.closed_surface_outcome(&failure.primary);
            }
            RenderFailureCause::ObservedFault(failure) => {
                return self
                    .closed_surface_outcome(&failure.primary)
                    .or_else(|| self.closed_surface_outcome(&failure.observed));
            }
            _ => return None,
        };
        Self::direct_surface_outcome(failure)
    }
    fn direct_surface_outcome(failure: &GuiSurfaceFailure) -> Option<RenderGuiSurfaceOutcome<'_>> {
        Some(match &failure.primary {
            Some(fault) => RenderGuiSurfaceOutcome::FaultWithSurfaceChange {
                fault,
                change: &failure.change,
            },
            None => RenderGuiSurfaceOutcome::SurfaceChanged(&failure.change),
        })
    }
    fn closed_surface_outcome<'a>(
        &'a self,
        primary: &'a RenderError,
    ) -> Option<RenderGuiSurfaceOutcome<'a>> {
        let change = match primary.gui_surface_outcome()? {
            RenderGuiSurfaceOutcome::SurfaceChanged(change)
            | RenderGuiSurfaceOutcome::FaultWithSurfaceChange { change, .. } => change,
        };
        Some(RenderGuiSurfaceOutcome::FaultWithSurfaceChange {
            fault: self,
            change,
        })
    }
    pub(crate) fn surface_changed(change: RenderGuiSurfaceChange) -> Self {
        let message = super::rejected(&change, "a fresh GUI packet and unchanged target").0;
        let detail = GuiSurfaceFailure {
            primary: None,
            change,
        };
        Self(
            message,
            Some(RenderFailureCause::GuiSurface(Box::new(detail))),
        )
    }
    pub(crate) fn with_surface_change(mut self, change: RenderGuiSurfaceChange) -> Self {
        if let Some(RenderFailureCause::GuiSurface(failure)) = &mut self.1 {
            failure.change.merge_observation(change);
            return self;
        }
        let message = self.0.clone();
        let detail = GuiSurfaceFailure {
            primary: Some(self),
            change,
        };
        Self(
            message,
            Some(RenderFailureCause::GuiSurface(Box::new(detail))),
        )
    }
}
