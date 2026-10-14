//! Later real observation faults cannot leave a surface-only recoverable classification.
use super::{ObservedRenderFault, RenderError, RenderFailureCause};
use crate::RenderGuiSurfaceOutcome;
impl RenderError {
    /// Inspect this layer's latest distinct observed real fault; `error.observed_failure()` complements `primary_failure()`.
    /// A new observation replaces the previous companion; it never replaces the original primary.
    pub fn observed_failure(&self) -> Option<&RenderError> {
        match &self.1 {
            Some(RenderFailureCause::ObservedFault(failure)) => Some(&failure.observed),
            _ => None,
        }
    }
    pub(crate) fn with_observed_failure(self, observed: RenderError) -> Self {
        if let Some(RenderGuiSurfaceOutcome::SurfaceChanged(change)) =
            observed.gui_surface_outcome()
        {
            return self.with_surface_change(change.clone());
        }
        if let Some(RenderGuiSurfaceOutcome::SurfaceChanged(change)) = self.gui_surface_outcome() {
            return observed.with_surface_change(change.clone());
        }
        if let Some(RenderFailureCause::Window(closure)) = &observed.1 {
            return self.with_window_target(*closure);
        }
        self.retain_observed_companion(observed)
    }
    fn retain_observed_companion(mut self, observed: RenderError) -> Self {
        if self.contains_observed_failure(&observed) || self.update_observed_companion(&observed) {
            return self;
        }
        let message = self.0.clone();
        let detail = ObservedRenderFault {
            primary: self,
            observed,
        };
        Self(
            message,
            Some(RenderFailureCause::ObservedFault(Box::new(detail))),
        )
    }
    fn update_observed_companion(&mut self, observed: &RenderError) -> bool {
        match &mut self.1 {
            Some(RenderFailureCause::ObservedFault(failure)) => failure.observed = observed.clone(),
            Some(RenderFailureCause::GuiSurface(failure)) => {
                let Some(primary) = &mut failure.primary else {
                    return false;
                };
                *primary = primary.clone().with_observed_failure(observed.clone());
            }
            Some(RenderFailureCause::WindowTarget(failure)) => {
                let primary = failure.primary.clone();
                failure.primary = primary.with_observed_failure(observed.clone());
            }
            _ => return false,
        }
        true
    }
    fn contains_observed_failure(&self, observed: &RenderError) -> bool {
        self == observed
            || self
                .primary_failure()
                .is_some_and(|primary| primary.contains_observed_failure(observed))
            || self
                .observed_failure()
                .is_some_and(|later| later.contains_observed_failure(observed))
    }
}
pub(crate) fn combine_observed_result<T>(
    outcome: Result<T, RenderError>,
    observation: Result<(), RenderError>,
) -> Result<T, RenderError> {
    let Err(observed) = observation else {
        return outcome;
    };
    match outcome {
        Err(primary) => Err(primary.with_observed_failure(observed)),
        Ok(_) => Err(observed),
    }
}
