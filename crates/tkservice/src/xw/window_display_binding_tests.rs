//! Named fake display calls isolate binding identity from mutable window metadata.
use super::*;
use crate::aspect::recorded_display_fixture;

#[test]
fn metadata_substitution_preserves_original_native_binding() {
    let original = recorded_display_fixture(41);
    let replacement = recorded_display_fixture(55);
    let mut window = XwWindow::with_fields(&original, XwWindowFields::default());
    window.retain_connection(&original).unwrap();
    window.window_base().fields_mut().display = replacement.clone();
    assert_eq!(window.display_address(), 41);
    assert_eq!(
        window.native_display_connection().object_address(),
        original.object_address()
    );
    assert_eq!(window.display_connection().get_display_aspect(), 55);
    assert_eq!(original.init_checked(0).unwrap_err().leases.windows, 1);
    replacement.init_checked(0).unwrap();
    drop(window);
    original.init_checked(0).unwrap();
}

#[test]
fn expose_default_pins_original_and_explicit_override_retains_its_argument() {
    let original = recorded_display_fixture(41);
    let replacement = recorded_display_fixture(55);
    let mut window = XwWindow::with_fields(&original, XwWindowFields::default());
    window.retain_connection(&original).unwrap();
    window.window_base().fields_mut().display = replacement.clone();
    let default = window.expose_connection_lease(&Handle::default()).unwrap();
    assert_eq!(default.display_address(), 41);
    let explicit = window.expose_connection_lease(&replacement).unwrap();
    assert_eq!(explicit.display_address(), 55);
    assert_eq!(replacement.init_checked(0).unwrap_err().leases.windows, 1);
    drop(explicit);
    replacement.init_checked(0).unwrap();
    assert_eq!(original.init_checked(0).unwrap_err().leases.windows, 2);
    drop(default);
    assert_eq!(original.init_checked(0).unwrap_err().leases.windows, 1);
}
#[test]
fn unavailable_expose_binding_is_a_typed_admission_failure() {
    let window = XwWindow::with_fields(&Handle::default(), XwWindowFields::default());
    let Err(failure) = window.expose_connection_lease(&Handle::default()) else {
        panic!("null display; expected typed unavailable binding refusal");
    };
    assert_eq!(
        failure.failure_class(),
        crate::aspect::AspectDisplayFailureClass::Unavailable
    );
}

struct RecordingExposeSend {
    display: Handle<AspectDisplayConnection>,
    addresses: Vec<usize>,
}
impl RecordingExposeSend {
    fn send(&mut self, address: usize) {
        assert_eq!(self.display.init_checked(0).unwrap_err().leases.windows, 1);
        self.addresses.push(address);
    }
}
#[test]
fn explicit_expose_send_retains_during_callback_and_releases_afterward() {
    let window = XwWindow::with_fields(&Handle::default(), XwWindowFields::default());
    let display = recorded_display_fixture(55);
    let mut sender = RecordingExposeSend {
        display: display.clone(),
        addresses: Vec::new(),
    };
    window
        .with_expose_connection(&display, |address| sender.send(address))
        .unwrap();
    assert_eq!(sender.addresses, [55]);
    display.init_checked(0).unwrap();
}
#[test]
fn refused_expose_admission_never_calls_sender() {
    let window = XwWindow::with_fields(&Handle::default(), XwWindowFields::default());
    let mut sender = RecordingExposeSend {
        display: recorded_display_fixture(55),
        addresses: Vec::new(),
    };
    assert!(
        window
            .with_expose_connection(&Handle::default(), |address| sender.send(address))
            .is_err()
    );
    assert!(sender.addresses.is_empty());
}
#[test]
fn default_native_binding_tracks_custom_window_metadata() {
    let window = crate::aspect::AspectNeutralWindow::default();
    let display = recorded_display_fixture(41);
    assert!(window.native_display_connection().is_null());
    window.window_base().fields_mut().display = display.clone();
    assert_eq!(
        window.native_display_connection().object_address(),
        display.object_address()
    );
}
