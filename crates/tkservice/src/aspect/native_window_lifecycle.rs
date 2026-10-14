//! Delivered destruction is a terminal native target state (ADR 0018, behavior v1).
use super::AspectDrawable;
use std::fmt;

/// The checked operation; e.g. `AspectNativeWindowOperation::Capture`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AspectNativeWindowOperation {
    /// Read a completed frame.
    Capture,
    /// Render a new frame.
    Redraw,
    /// Apply a native extent notification.
    Resize,
    /// Bind a view to a target.
    Bind,
    /// Install a GUI packet.
    AttachGui,
    /// Retire a GUI packet.
    ClearGui,
}

/// Evidence that retired a target; e.g. `AspectNativeWindowClosureCause::DestroyNotify`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AspectNativeWindowClosureCause {
    /// The matching native destruction event was delivered.
    DestroyNotify,
}

/// Original target identity survives retirement; inspect `closure.original_handle`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AspectNativeWindowClosure {
    /// The destroyed identity, retained after the live native handle becomes zero.
    pub original_handle: AspectDrawable,
    /// Evidence establishing terminal closure.
    pub cause: AspectNativeWindowClosureCause,
    /// The native-window-lifecycle decision governing this state.
    pub behavior_version: u32,
}

/// A typed closed-target failure; match `AspectNativeWindowFailure::Closed { .. }`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AspectNativeWindowFailure {
    /// The operation cannot succeed on a known closed native target.
    Closed {
        /// The refused operation.
        operation: AspectNativeWindowOperation,
        /// Original target, cause and behavior version.
        closure: AspectNativeWindowClosure,
    },
}
impl AspectNativeWindowClosure {
    /// Associate the refusal with its operation; `closure.failure(AspectNativeWindowOperation::Capture)`.
    pub fn failure(self, operation: AspectNativeWindowOperation) -> AspectNativeWindowFailure {
        AspectNativeWindowFailure::Closed {
            operation,
            closure: self,
        }
    }
}
impl fmt::Display for AspectNativeWindowFailure {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self::Closed { operation, closure } = self;
        write!(
            output,
            "operation={operation:?}, native_window={}, class=Closed, cause={:?}, behavior_version={}; expected a live native target",
            closure.original_handle, closure.cause, closure.behavior_version
        )
    }
}
impl std::error::Error for AspectNativeWindowFailure {}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn closed_failure_preserves_target_operation_and_cause() {
        let closure = AspectNativeWindowClosure {
            original_handle: 91,
            cause: AspectNativeWindowClosureCause::DestroyNotify,
            behavior_version: 1,
        };
        let failure = closure.failure(AspectNativeWindowOperation::Capture);
        let AspectNativeWindowFailure::Closed {
            operation,
            closure: actual,
        } = failure;
        assert_eq!(operation, AspectNativeWindowOperation::Capture);
        assert_eq!(actual, closure);
        let message = failure.to_string();
        assert!(message.contains("native_window=91, class=Closed, cause=DestroyNotify"));
    }
    #[test]
    fn lifecycle_values_are_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<AspectNativeWindowOperation>();
        assert_send_sync::<AspectNativeWindowClosureCause>();
        assert_send_sync::<AspectNativeWindowClosure>();
        assert_send_sync::<AspectNativeWindowFailure>();
    }
}
