//! Admission and release share the connection lock; a Handle alone never authorizes reset.
use super::*;
use rustycad_tkernel::standard::Handle;

/// Pins one connection until drop; obtain with `AspectDisplayConnection::acquire_lease(&display, purpose)?`.
/// Externally owned raw displays still require their caller to keep the native connection alive.
pub struct AspectDisplayConnectionLease {
    connection: Handle<AspectDisplayConnection>,
    purpose: AspectDisplayLeasePurpose,
    display: usize,
}
impl AspectDisplayConnectionLease {
    /// The retained connection object; `lease.retained_connection()` remains fixed for its lifetime.
    pub fn retained_connection(&self) -> Handle<AspectDisplayConnection> {
        self.connection.clone()
    }
    /// The pinned native address; `lease.display_address()` does not follow later replacement.
    pub fn display_address(&self) -> usize {
        self.display
    }
    /// Compare retained connection objects, not numeric native addresses; `first.same_connection(&second)`.
    pub fn same_connection(&self, other: &Self) -> bool {
        self.connection.object_address() == other.connection.object_address()
    }
}
impl Drop for AspectDisplayConnectionLease {
    fn drop(&mut self) {
        let mut state = self.connection.state.write();
        let count = state.consumer_count_mut(self.purpose);
        *count = count
            .checked_sub(1)
            .expect("display lease count=0; expected an acquired consumer");
    }
}
impl AspectDisplayConnection {
    /// Retain before creating native consumers; `Self::acquire_lease(&display, AspectDisplayLeasePurpose::Window)?`.
    pub fn acquire_lease(
        connection: &Handle<Self>,
        purpose: AspectDisplayLeasePurpose,
    ) -> Result<AspectDisplayConnectionLease, AspectDisplayLifecycleFailure> {
        let Some(value) = connection.get() else {
            return Err(undefined_connection());
        };
        let mut state = value.state.write();
        value.admit_consumer(&mut state, purpose)?;
        Ok(AspectDisplayConnectionLease {
            connection: connection.clone(),
            purpose,
            display: state.display,
        })
    }
    fn admit_consumer(
        &self,
        state: &mut ConnectionState,
        purpose: AspectDisplayLeasePurpose,
    ) -> Result<(), AspectDisplayLifecycleFailure> {
        let failure = self.lifecycle_failure(
            state,
            AspectDisplayOperation::AcquireLease,
            state.display,
            AspectDisplayLifecycleCause::Undefined,
        );
        if state.display == 0 {
            return Err(failure);
        }
        state
            .increment_consumer(purpose)
            .map_err(|cause| AspectDisplayLifecycleFailure { cause, ..failure })
    }
    pub(super) fn lifecycle_failure(
        &self,
        state: &ConnectionState,
        operation: AspectDisplayOperation,
        requested_display: usize,
        cause: AspectDisplayLifecycleCause,
    ) -> AspectDisplayLifecycleFailure {
        AspectDisplayLifecycleFailure {
            operation,
            connection_identity: std::ptr::from_ref(self) as usize,
            current_display: state.display,
            requested_display,
            leases: state.leases,
            cause,
            behavior_version: 1,
        }
    }
}
impl ConnectionState {
    fn consumer_count_mut(&mut self, purpose: AspectDisplayLeasePurpose) -> &mut usize {
        match purpose {
            AspectDisplayLeasePurpose::Window => &mut self.leases.windows,
            AspectDisplayLeasePurpose::VulkanInstance => &mut self.leases.vulkan_instances,
        }
    }
    fn increment_consumer(
        &mut self,
        purpose: AspectDisplayLeasePurpose,
    ) -> Result<(), AspectDisplayLifecycleCause> {
        let count = self.consumer_count_mut(purpose);
        *count = count
            .checked_add(1)
            .ok_or(AspectDisplayLifecycleCause::CountExhausted(purpose))?;
        Ok(())
    }
}
fn undefined_connection() -> AspectDisplayLifecycleFailure {
    AspectDisplayLifecycleFailure {
        operation: AspectDisplayOperation::AcquireLease,
        connection_identity: 0,
        current_display: 0,
        requested_display: 0,
        leases: AspectDisplayLeaseCounts::default(),
        cause: AspectDisplayLifecycleCause::Undefined,
        behavior_version: 1,
    }
}
