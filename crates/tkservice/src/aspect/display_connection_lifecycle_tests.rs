//! Native reset admission is exercised without a display or process-global native hooks.
use super::baseline::{RecordingDisplayCalls, fake_connection};
use super::*;
use rustycad_tkernel::standard::Handle;

fn initialized_fixture() -> (Handle<AspectDisplayConnection>, Arc<RecordingDisplayCalls>) {
    let native = RecordingDisplayCalls::new(41);
    let connection = Handle::new(fake_connection(native.clone()));
    connection.init_checked(0).unwrap();
    native.calls.lock().unwrap().clear();
    (connection, native)
}
#[test]
fn retained_display_reset_is_refused_before_any_native_mutation() {
    let (connection, native) = initialized_fixture();
    let lease =
        AspectDisplayConnection::acquire_lease(&connection, AspectDisplayLeasePurpose::Window)
            .unwrap();
    let failure = connection.init_checked(0).unwrap_err();
    assert_eq!(failure.operation, AspectDisplayOperation::Initialize);
    assert_eq!(failure.current_display, 41);
    assert_eq!(failure.requested_display, 0);
    assert_eq!(failure.leases.windows, 1);
    assert_eq!(failure.cause, AspectDisplayLifecycleCause::Retained);
    assert!(native.recorded().is_empty());
    assert_eq!(connection.get_display_aspect(), 41);
    assert_eq!(connection.get_atom(AspectXAtom::DeleteWindow).unwrap(), 73);
    assert_eq!(lease.display_address(), 41);
}
#[test]
fn display_lifecycle_values_and_lease_are_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<AspectDisplayOperation>();
    assert_send_sync::<AspectDisplayLeasePurpose>();
    assert_send_sync::<AspectDisplayLeaseCounts>();
    assert_send_sync::<AspectDisplayFailureClass>();
    assert_send_sync::<AspectDisplayLifecycleCause>();
    assert_send_sync::<AspectDisplayLifecycleFailure>();
    assert_send_sync::<AspectDisplayConnectionLease>();
}

#[test]
fn both_consumer_kinds_block_reset_until_the_last_lease_drops() {
    let (connection, native) = initialized_fixture();
    let window = acquire(&connection, AspectDisplayLeasePurpose::Window);
    let runtime = acquire(&connection, AspectDisplayLeasePurpose::VulkanInstance);
    let observed = connection.init_checked(55).unwrap_err();
    let counts = observed.leases;
    assert_eq!((counts.windows, counts.vulkan_instances), (1, 1));
    drop(window);
    assert_eq!(connection.init_checked(0).unwrap_err().leases.windows, 0);
    assert!(native.recorded().is_empty());
    drop(runtime);
    let ordinary_alias = connection.clone();
    connection.init_checked(0).unwrap();
    assert_eq!(
        native.recorded(),
        ["close(41)", "open(:fixture)", "atom(41)"]
    );
    assert_eq!(ordinary_alias.get_display_aspect(), 41);
    assert_eq!(observed.leases.windows, 1);
}
fn acquire(
    connection: &Handle<AspectDisplayConnection>,
    purpose: AspectDisplayLeasePurpose,
) -> AspectDisplayConnectionLease {
    AspectDisplayConnection::acquire_lease(connection, purpose).unwrap()
}
#[test]
fn lease_identity_does_not_certify_equal_raw_display_addresses() {
    let (first, _) = initialized_fixture();
    let (second, _) = initialized_fixture();
    let first_lease = acquire(&first, AspectDisplayLeasePurpose::Window);
    let same_owner = acquire(&first.clone(), AspectDisplayLeasePurpose::VulkanInstance);
    let different_owner = acquire(&second, AspectDisplayLeasePurpose::Window);
    assert_eq!(
        first_lease.display_address(),
        different_owner.display_address()
    );
    assert!(first_lease.same_connection(&same_owner));
    assert!(!first_lease.same_connection(&different_owner));
}
#[test]
fn undefined_or_exhausted_lease_acquisition_changes_nothing() {
    let empty = Handle::new(fake_connection(RecordingDisplayCalls::new(41)));
    let absent = AspectDisplayConnection::acquire_lease(
        &Handle::default(),
        AspectDisplayLeasePurpose::Window,
    )
    .err()
    .unwrap();
    assert_eq!(absent.connection_identity, 0);
    assert_eq!(absent.cause, AspectDisplayLifecycleCause::Undefined);
    assert!(
        AspectDisplayConnection::acquire_lease(&empty, AspectDisplayLeasePurpose::Window).is_err()
    );
    assert_exhausted_consumer();
}
fn assert_exhausted_consumer() {
    let (connection, native) = initialized_fixture();
    connection.state.write().leases.windows = usize::MAX;
    let failure =
        AspectDisplayConnection::acquire_lease(&connection, AspectDisplayLeasePurpose::Window)
            .err()
            .unwrap();
    assert_eq!(failure.failure_class(), AspectDisplayFailureClass::Capacity);
    assert_eq!(failure.leases.windows, usize::MAX);
    assert_eq!(connection.state.read().leases.windows, usize::MAX);
    assert!(native.recorded().is_empty());
    connection.state.write().leases.windows = 0;
}
#[test]
fn legacy_adapter_names_refusal_but_native_error_payload_stays_in_checked_result() {
    let (connection, _) = initialized_fixture();
    let _lease = acquire(&connection, AspectDisplayLeasePurpose::Window);
    let legacy = connection.init(55).unwrap_err();
    assert_eq!(
        legacy.exception_type(),
        "Aspect_DisplayConnectionLifecycleError"
    );
    assert!(
        legacy
            .message()
            .contains("current_display=41, requested_display=55")
    );
    let checked = connection.init_checked(55).unwrap_err();
    assert_eq!(checked.failure_class(), AspectDisplayFailureClass::InUse);
    assert!(std::error::Error::source(&checked).is_none());
    assert_native_initialization_error();
}
fn assert_native_initialization_error() {
    let native = RecordingDisplayCalls::new(0);
    let connection = fake_connection(native);
    connection.state.write().display = 41;
    connection.state.write().is_own_display = true;
    let failure = connection.init_checked(0).unwrap_err();
    assert_eq!(failure.current_display, 41);
    assert_eq!(connection.get_display_aspect(), 0);
    assert_eq!(
        failure.failure_class(),
        AspectDisplayFailureClass::Unavailable
    );
    assert!(std::error::Error::source(&failure).is_some());
    assert_eq!(
        failure.into_legacy_failure().exception_type(),
        "Aspect_DisplayConnectionDefinitionError"
    );
}
