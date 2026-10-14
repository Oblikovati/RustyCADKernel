//! Errors name the rejected input or operation and its expected contract.
use rustycad_tkservice::aspect::{AspectDisplayLifecycleFailure, AspectNativeWindowFailure};
use std::fmt;
/// A renderer input, capability or device failure; inspect with `error.to_string()`.
#[derive(Clone, PartialEq, Eq)]
pub struct RenderError(String, Option<NativeLifecycleFailure>);
#[derive(Clone, PartialEq, Eq)]
enum NativeLifecycleFailure {
    Window(AspectNativeWindowFailure),
    Display(AspectDisplayLifecycleFailure),
}
impl RenderError {
    /// Describe a rejected backend input; `RenderError::rejected(0, "positive extent")`.
    pub fn rejected(value: impl fmt::Debug, expected: &str) -> Self {
        rejected(value, expected)
    }
    /// Inspect a closed target without parsing text; `error.native_window_failure()`.
    pub fn native_window_failure(&self) -> Option<&AspectNativeWindowFailure> {
        match &self.1 {
            Some(NativeLifecycleFailure::Window(failure)) => Some(failure),
            _ => None,
        }
    }
    /// Inspect native connection admission failure; `error.native_display_failure()`.
    pub fn native_display_failure(&self) -> Option<&AspectDisplayLifecycleFailure> {
        match &self.1 {
            Some(NativeLifecycleFailure::Display(failure)) => Some(failure),
            _ => None,
        }
    }
}
impl From<AspectNativeWindowFailure> for RenderError {
    fn from(failure: AspectNativeWindowFailure) -> Self {
        Self(
            failure.to_string(),
            Some(NativeLifecycleFailure::Window(failure)),
        )
    }
}
impl From<AspectDisplayLifecycleFailure> for RenderError {
    fn from(failure: AspectDisplayLifecycleFailure) -> Self {
        Self(
            failure.to_string(),
            Some(NativeLifecycleFailure::Display(failure)),
        )
    }
}
impl fmt::Debug for RenderError {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        output.debug_tuple("RenderError").field(&self.0).finish()
    }
}
impl fmt::Display for RenderError {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        output.write_str(&self.0)
    }
}
impl std::error::Error for RenderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.1 {
            Some(NativeLifecycleFailure::Window(failure)) => Some(failure),
            Some(NativeLifecycleFailure::Display(failure)) => Some(failure),
            None => None,
        }
    }
}
pub(crate) fn rejected(value: impl fmt::Debug, expected: &str) -> RenderError {
    RenderError(format!("received {value:?}; expected {expected}"), None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkservice::aspect::*;
    #[test]
    fn ordinary_render_errors_preserve_display_and_debug() {
        let failure = RenderError::rejected(7, "positive extent");
        assert_eq!(failure.to_string(), "received 7; expected positive extent");
        assert_eq!(
            format!("{failure:?}"),
            "RenderError(\"received 7; expected positive extent\")"
        );
        assert!(failure.native_window_failure().is_none());
        assert!(std::error::Error::source(&failure).is_none());
    }
    #[test]
    fn closed_render_errors_retain_typed_cause() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<RenderError>();
        let closure = AspectNativeWindowClosure {
            original_handle: 91,
            cause: AspectNativeWindowClosureCause::DestroyNotify,
            behavior_version: 1,
        };
        let cause = closure.failure(AspectNativeWindowOperation::Capture);
        let failure = RenderError::from(cause);
        assert_eq!(failure.native_window_failure(), Some(&cause));
        assert_eq!(failure.to_string(), cause.to_string());
        assert!(std::error::Error::source(&failure).is_some());
    }
    #[test]
    fn display_render_errors_preserve_structured_admission_cause() {
        let cause = rustycad_tkservice::aspect::AspectDisplayConnection::acquire_lease(
            &rustycad_tkernel::standard::Handle::default(),
            AspectDisplayLeasePurpose::VulkanInstance,
        )
        .err()
        .unwrap();
        let failure = RenderError::from(cause.clone());
        assert_eq!(failure.native_display_failure(), Some(&cause));
        assert!(failure.native_window_failure().is_none());
        assert!(std::error::Error::source(&failure).is_some());
        assert_eq!(failure.to_string(), cause.to_string());
    }
}
