//! No display is needed after retirement: any accidental Xlib call would violate this fixture.
use super::*;

fn retained_native_fixture() -> XwWindow {
    XwWindow::with_fields(
        &Handle::default(),
        XwWindowFields {
            x_window: 91,
            x_right: 64,
            y_bottom: 48,
            ..Default::default()
        },
    )
}

#[test]
fn delivered_destruction_is_terminal_and_preserves_original_identity() {
    let window = retained_native_fixture();
    assert_eq!(window.retire_destroyed_window(91), Some(true));
    let closure = window.native_window_closure().unwrap();
    assert_eq!(closure.original_handle, 91);
    assert_eq!(closure.behavior_version, 1);
    assert_eq!(window.retire_destroyed_window(91), Some(false));
    assert_eq!(window.retire_destroyed_window(92), None);
    assert_eq!(window.native_window_closure(), Some(closure));
    assert_eq!(window.native_handle(), 0);
    assert!(!window.fields.read().is_own_win);
}

#[test]
fn closed_legacy_callbacks_never_query_a_native_target() {
    let window = retained_native_fixture();
    window.retire_destroyed_window(91);
    assert_eq!(window.size(), (0, 0));
    assert_eq!(window.position(), (0, 0, 0, 0));
    assert_eq!(window.ratio(), 0.);
    assert_eq!(window.do_resize(), Resize::Unknown);
    assert!(!window.is_mapped());
    assert!(!window.do_mapping());
    window.map();
    window.unmap();
    window.set_title(&AsciiString::from("closed"));
    window.invalidate_content(&Handle::default());
}

#[test]
fn foreign_and_zero_destruction_leave_the_target_live() {
    let window = retained_native_fixture();
    assert_eq!(window.retire_destroyed_window(92), None);
    assert_eq!(window.retire_destroyed_window(0), None);
    assert!(window.native_window_closure().is_none());
    assert_eq!(window.native_handle(), 91);
}
