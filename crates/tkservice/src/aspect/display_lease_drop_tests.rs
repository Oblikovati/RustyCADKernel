//! Fake native consumers prove cleanup order independently of the Xlib/Vulkan implementations.
use super::baseline::{RecordingDisplayCalls, fake_connection};
use super::*;
use rustycad_tkernel::standard::Handle;

struct RecordingNativeConsumer {
    _lease: AspectDisplayConnectionLease,
    native: Arc<RecordingDisplayCalls>,
    teardown: &'static str,
}
impl Drop for RecordingNativeConsumer {
    fn drop(&mut self) {
        self.native
            .calls
            .lock()
            .unwrap()
            .push(self.teardown.to_owned());
    }
}
fn owned_consumer(
    purpose: AspectDisplayLeasePurpose,
    teardown: &'static str,
    native: Arc<RecordingDisplayCalls>,
) -> RecordingNativeConsumer {
    let connection = Handle::new(fake_connection(native.clone()));
    connection.init_checked(0).unwrap();
    let lease = AspectDisplayConnection::acquire_lease(&connection, purpose).unwrap();
    native.calls.lock().unwrap().clear();
    RecordingNativeConsumer {
        _lease: lease,
        native,
        teardown,
    }
}
#[test]
fn native_consumer_teardown_precedes_last_owned_connection_close() {
    for (purpose, event) in [
        (AspectDisplayLeasePurpose::Window, "destroy-window"),
        (
            AspectDisplayLeasePurpose::VulkanInstance,
            "destroy-instance",
        ),
    ] {
        let native = RecordingDisplayCalls::new(41);
        let consumer = owned_consumer(purpose, event, native.clone());
        assert!(native.recorded().is_empty());
        drop(consumer);
        assert_eq!(native.recorded(), [event, "close(41)"]);
    }
}
struct FailingNativeConsumer;
impl FailingNativeConsumer {
    fn create(connection: &Handle<AspectDisplayConnection>) -> Result<(), &'static str> {
        let _lease =
            AspectDisplayConnection::acquire_lease(connection, AspectDisplayLeasePurpose::Window)
                .unwrap();
        Err("native consumer setup failed; expected successful fixture creation")
    }
}
#[test]
fn failed_consumer_setup_releases_its_temporary_lease() {
    let native = RecordingDisplayCalls::new(41);
    let connection = Handle::new(fake_connection(native.clone()));
    connection.init_checked(0).unwrap();
    assert!(FailingNativeConsumer::create(&connection).is_err());
    assert_eq!(
        connection.state.read().leases,
        AspectDisplayLeaseCounts::default()
    );
    connection.init_checked(0).unwrap();
}
