//! Typed native-display-lifecycle v1 refusals; the legacy init adapter is intentionally lossy.
use rustycad_tkernel::standard::Failure;
use std::fmt;

/// The protected operation; e.g. `AspectDisplayOperation::Initialize`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AspectDisplayOperation {
    /// Replace the native connection.
    Initialize,
    /// Retain a connection for a native consumer.
    AcquireLease,
}
/// The native consumer whose lifetime pins a connection; e.g. `AspectDisplayLeasePurpose::Window`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AspectDisplayLeasePurpose {
    /// An owned/wrapped XwWindow through destruction, or a temporary expose send.
    Window,
    /// Vulkan's connection retention, through instance teardown.
    VulkanInstance,
}
/// Owned consumer counts, not ordinary handle counts; inspect `counts.windows`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AspectDisplayLeaseCounts {
    /// Retained XwWindow objects and active expose sends.
    pub windows: usize,
    /// Retained Vulkan instance connections.
    pub vulkan_instances: usize,
}
/// Classification of a refused display operation; e.g. `AspectDisplayFailureClass::InUse`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AspectDisplayFailureClass {
    /// A native consumer still retains the connection.
    InUse,
    /// The requested connection is unavailable.
    Unavailable,
    /// The consumer count cannot be represented.
    Capacity,
}
/// Structured cause, including native initialization errors; match `AspectDisplayLifecycleCause::Retained`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AspectDisplayLifecycleCause {
    /// Native consumers prohibit replacement.
    Retained,
    /// A null or uninitialized connection cannot be retained.
    Undefined,
    /// The native initialization operation failed.
    Native(Box<Failure>),
    /// Incrementing the selected count would overflow.
    CountExhausted(AspectDisplayLeasePurpose),
}
/// Immutable diagnostics; inspect `failure.operation`, `failure.cause`, and `failure.leases`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AspectDisplayLifecycleFailure {
    /// Operation that could not complete.
    pub operation: AspectDisplayOperation,
    /// Retained connection-object identity, not a native pointer identity guarantee.
    pub connection_identity: usize,
    /// Native connection at admission time.
    pub current_display: usize,
    /// Requested replacement; zero requests opening the named display.
    pub requested_display: usize,
    /// Consumers at admission time.
    pub leases: AspectDisplayLeaseCounts,
    /// Typed refusal cause.
    pub cause: AspectDisplayLifecycleCause,
    /// Native-display-lifecycle decision version.
    pub behavior_version: u32,
}
impl AspectDisplayLifecycleFailure {
    /// Classify the failure without parsing text; `failure.failure_class()`.
    pub fn failure_class(&self) -> AspectDisplayFailureClass {
        match self.cause {
            AspectDisplayLifecycleCause::Retained => AspectDisplayFailureClass::InUse,
            AspectDisplayLifecycleCause::CountExhausted(_) => AspectDisplayFailureClass::Capacity,
            _ => AspectDisplayFailureClass::Unavailable,
        }
    }
    /// Legacy adapter discards structured fields; prefer checked init (ADR0019).
    /// Example: `failure.into_legacy_failure().exception_type()`.
    pub(crate) fn into_legacy_failure(self) -> Failure {
        if let AspectDisplayLifecycleCause::Native(failure) = self.cause {
            return *failure;
        }
        Failure::with_message("Aspect_DisplayConnectionLifecycleError", &self.to_string())
            .with_exception_parent("Aspect_DisplayConnectionDefinitionError")
    }
}
impl fmt::Display for AspectDisplayLifecycleFailure {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            output,
            "operation={:?}, connection={}, current_display={}, requested_display={}, class={:?}, cause={:?}, leases={:?}, behavior_version={}; expected an initialized connection for retention or zero native consumer leases for replacement",
            self.operation,
            self.connection_identity,
            self.current_display,
            self.requested_display,
            self.failure_class(),
            self.cause,
            self.leases,
            self.behavior_version
        )
    }
}
impl std::error::Error for AspectDisplayLifecycleFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.cause {
            AspectDisplayLifecycleCause::Native(failure) => Some(failure.as_ref()),
            _ => None,
        }
    }
}
