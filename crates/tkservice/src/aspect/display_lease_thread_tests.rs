//! Ordinary host-thread admission checks; this does not certify concurrent native window operations.
use super::baseline::{RecordingDisplayCalls, fake_connection};
use super::*;
use rustycad_tkernel::standard::Handle;
use std::sync::Barrier;

#[test]
fn held_lease_refuses_reset_from_another_thread() {
    let connection = Handle::new(fake_connection(RecordingDisplayCalls::advancing()));
    connection.init_checked(0).unwrap();
    let rendezvous = Arc::new(Barrier::new(2));
    std::thread::scope(|scope| {
        scope.spawn(|| hold_until_reset_attempt(&connection, &rendezvous));
        rendezvous.wait();
        assert_eq!(
            connection.init_checked(0).unwrap_err().failure_class(),
            AspectDisplayFailureClass::InUse
        );
        rendezvous.wait();
    });
    connection.init_checked(0).unwrap();
    assert_eq!(connection.get_display_aspect(), 42);
}
fn hold_until_reset_attempt(connection: &Handle<AspectDisplayConnection>, barrier: &Barrier) {
    let lease =
        AspectDisplayConnection::acquire_lease(connection, AspectDisplayLeasePurpose::Window)
            .unwrap();
    barrier.wait();
    barrier.wait();
    assert_eq!(lease.display_address(), connection.get_display_aspect());
}
#[test]
fn concurrent_retention_never_observes_replacement_while_held() {
    let connection = Handle::new(fake_connection(RecordingDisplayCalls::advancing()));
    connection.init_checked(0).unwrap();
    std::thread::scope(|scope| {
        for _ in 0..4 {
            scope.spawn(|| retain_repeatedly(&connection));
        }
        for _ in 0..256 {
            if let Err(failure) = connection.init_checked(0) {
                assert_eq!(failure.failure_class(), AspectDisplayFailureClass::InUse);
            }
            std::thread::yield_now();
        }
    });
    assert_eq!(
        connection.state.read().leases,
        AspectDisplayLeaseCounts::default()
    );
    connection.init_checked(0).unwrap();
}
fn retain_repeatedly(connection: &Handle<AspectDisplayConnection>) {
    for _ in 0..256 {
        let lease =
            AspectDisplayConnection::acquire_lease(connection, AspectDisplayLeasePurpose::Window)
                .unwrap();
        std::thread::yield_now();
        assert_eq!(lease.display_address(), connection.get_display_aspect());
        drop(lease);
    }
}
