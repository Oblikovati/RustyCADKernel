//! Committed native input snapshots preserve reset and independent sequence ownership.
#[path = "support/view_input_buffer_fixture.rs"]
mod fixture;
use fixture::{native_values, seeded, snapshot};
use rustycad_tkv3d::ais::*;
#[test]
fn constructors_and_public_event_defaults_match_native() {
    assert_eq!(
        snapshot(&AisViewInputBuffer::new()),
        native_values("default")
    );
    assert_eq!(
        snapshot(&AisViewInputBuffer::default()),
        native_values("default")
    );
    assert_eq!(snapshot(&seeded()), native_values("pending"));
}
#[test]
fn reset_acknowledges_flags_and_zoom_but_retains_gesture_payload() {
    let mut events = seeded();
    events.reset();
    assert_eq!(snapshot(&events), native_values("reset"));
    events.reset();
    assert_eq!(snapshot(&events), native_values("reset_twice"));
}
#[test]
fn copied_event_sequences_are_independent() {
    let mut events = seeded();
    let mut copy = events.clone();
    events.reset();
    assert_eq!(snapshot(&copy), native_values("copy"));
    copy.selection.points.clear(None);
    copy.zoom_actions.clear(None);
    assert_eq!(snapshot(&copy), native_values("copy_modified"));
    assert_eq!(snapshot(&events), native_values("reset"));
}
#[test]
fn selection_and_render_buffer_enum_values_match_native() {
    let actual = [
        AisViewSelectionTool::Picking as i32,
        AisViewSelectionTool::RubberBand as i32,
        AisViewSelectionTool::Polygon as i32,
        AisViewSelectionTool::ZoomWindow as i32,
        AisViewInputBufferType::Ui as i32,
        AisViewInputBufferType::Gl as i32,
    ];
    assert_eq!(actual.map(f64::from).as_slice(), native_values("enums"));
    assert_eq!(
        AisViewSelectionTool::default(),
        AisViewSelectionTool::Picking
    );
    assert_eq!(
        AisViewInputBufferType::default(),
        AisViewInputBufferType::Ui
    );
}
