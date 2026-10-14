//! A closed target accompanies rather than replaces a failed checked GUI operation.
use super::{RenderError, RenderFailureCause, WindowTargetFailure};
use rustycad_tkservice::aspect::AspectNativeWindowFailure;
impl RenderError {
    /// Inspect the immediately retained failure; `error.primary_failure()` may itself have a primary.
    /// Follow `primary_failure()` or `Error::source()` to inspect earlier causes; no leaf identity is implied.
    pub fn primary_failure(&self) -> Option<&RenderError> {
        match &self.1 {
            Some(RenderFailureCause::GuiSurface(failure)) => failure.primary.as_ref(),
            Some(RenderFailureCause::WindowTarget(failure)) => Some(&failure.primary),
            Some(RenderFailureCause::ObservedFault(failure)) => Some(&failure.primary),
            _ => None,
        }
    }
    pub(crate) fn with_window_target(mut self, closure: AspectNativeWindowFailure) -> Self {
        if let Some(RenderFailureCause::GuiSurface(failure)) = &mut self.1 {
            let primary = failure.primary.take().map_or_else(
                || Self::from(closure),
                |primary| primary.with_window_target(closure),
            );
            self.0 = primary.to_string();
            failure.primary = Some(primary);
            return self;
        }
        if let Some(RenderFailureCause::WindowTarget(failure)) = &mut self.1 {
            failure.closure = closure;
            return self;
        }
        if self.native_window_failure() == Some(&closure) {
            return self;
        }
        self.wrap_window_target(closure)
    }
    fn wrap_window_target(self, closure: AspectNativeWindowFailure) -> Self {
        let message = self.0.clone();
        let detail = WindowTargetFailure {
            primary: self,
            closure,
        };
        Self(
            message,
            Some(RenderFailureCause::WindowTarget(Box::new(detail))),
        )
    }
}
pub(crate) fn reconcile_native_target<T>(
    outcome: Result<T, RenderError>,
    observed: Result<(), RenderError>,
) -> Result<T, RenderError> {
    let Err(target) = observed else {
        return outcome;
    };
    match (outcome, target.native_window_failure()) {
        (Err(primary), Some(closure)) => Err(primary.with_window_target(*closure)),
        (Err(primary), None) => Err(primary.with_observed_failure(target)),
        (Ok(_), _) => Err(target),
    }
}
