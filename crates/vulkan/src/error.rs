//! Errors name the rejected input or operation and its expected contract.
use rustycad_tkservice::aspect::AspectNativeWindowFailure;
use std::fmt;
/// A renderer input, capability or device failure; inspect with `error.to_string()`.
#[derive(Clone, PartialEq, Eq)]
pub struct RenderError(String, Option<AspectNativeWindowFailure>);
impl RenderError {
    /// Describe a rejected backend input; `RenderError::rejected(0, "positive extent")`.
    pub fn rejected(value: impl fmt::Debug, expected: &str) -> Self {
        rejected(value, expected)
    }
    /// Inspect a closed target without parsing text; `error.native_window_failure()`.
    pub fn native_window_failure(&self) -> Option<&AspectNativeWindowFailure> {
        self.1.as_ref()
    }
}
impl From<AspectNativeWindowFailure> for RenderError {
    fn from(failure: AspectNativeWindowFailure) -> Self {
        Self(failure.to_string(), Some(failure))
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
        self.1
            .as_ref()
            .map(|failure| failure as &dyn std::error::Error)
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
}
