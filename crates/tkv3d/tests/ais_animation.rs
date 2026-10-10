//! Exact native timeline behavior, including cached durations and callback order.
#[path = "support/animation_recorder.rs"]
mod recorder;
use recorder::*;
use rustycad_tkernel::{
    standard::{Handle, SharedState},
    tcollection::AsciiString,
};
use rustycad_tkv3d::ais::*;
use std::sync::Arc;
fn timeline(name: &str) -> Handle<AisAnimation> {
    Handle::new(AisAnimation::new(name))
}
fn atom(
    name: &str,
    frames: &Arc<SharedState<Vec<RecordedAnimationFrame>>>,
) -> Handle<RecordingAnimation> {
    Handle::new(RecordingAnimation::new(name, frames.clone()))
}
#[test]
fn defaults_names_and_empty_started_timeline_follow_native_state() {
    let root = timeline("root");
    assert_eq!(root.name(), AsciiString::from("root"));
    assert!(root.is_stopped() && !root.has_own_duration() && root.timer().is_null());
    assert_eq!(
        (
            root.start_pts(),
            root.own_duration(),
            root.duration(),
            root.elapsed_time()
        ),
        (0., 0., 0., 0.)
    );
    assert_eq!(AisAnimationProgress::default().pts, -1.);
    root.start(false).unwrap();
    assert!(!root.is_stopped());
    assert!(!root.update(0.).unwrap());
    assert!(root.is_stopped());
}
#[test]
fn identity_edits_find_cached_limits_and_clear_keep_native_quirks() {
    let root = timeline("root");
    let child = timeline("same");
    child.set_start_pts(2.);
    child.set_own_duration(3.);
    root.add(&child.cast()).unwrap();
    root.add(&child.cast()).unwrap();
    assert_eq!(root.children().length(), 1);
    assert_eq!(
        root.find(&AsciiString::from("same")).unwrap(),
        child.cast::<dyn AisAnimationSource>()
    );
    assert!(root.find(&AsciiString::from("absent")).unwrap().is_null());
    child.set_start_pts(4.);
    assert_eq!(root.duration(), 5.);
    root.add(&child.cast()).unwrap();
    assert_eq!(root.duration(), 7.);
    root.clear();
    assert_eq!(root.duration(), 7.);
    root.update_total_duration().unwrap();
    assert_eq!(root.duration(), 0.);
    assert!(!root.remove(&child.cast()).unwrap());
    assert_eq!(
        root.add(&Handle::default()).unwrap_err().exception_type(),
        "Standard_ProgramError"
    );
}
#[test]
fn replace_allows_duplicate_handles_and_copy_retains_name_and_state() {
    let root = timeline("root");
    let other = timeline("other");
    let first = timeline("first");
    let second = timeline("second");
    first.set_own_duration(2.);
    second.set_own_duration(4.);
    other.add(&first.cast()).unwrap();
    other.add(&second.cast()).unwrap();
    assert!(other.replace(&first.cast(), &second.cast()).unwrap());
    assert_eq!(other.children().length(), 2);
    assert!(other.remove(&second.cast()).unwrap());
    assert_eq!(other.children().length(), 1);
    assert!(!other.replace(&first.cast(), &Handle::default()).unwrap());
    other.set_start_pts(3.);
    other.set_own_duration(9.);
    root.copy_from(&other.cast()).unwrap();
    assert_eq!(
        (root.name(), root.start_pts(), root.own_duration()),
        (AsciiString::from("root"), 3., 9.)
    );
    assert!(root.is_stopped());
    assert_eq!(
        root.children().value(1).unwrap(),
        &second.cast::<dyn AisAnimationSource>()
    );
    root.copy_from(&root.cast()).unwrap();
    assert!(root.children().is_empty());
    assert_eq!(root.duration(), 9.);
}
#[test]
fn nested_progress_is_global_pts_and_local_offsets_with_children_first() {
    let frames = Arc::new(SharedState::new(Vec::new()));
    let root = atom("root", &frames);
    let child = atom("child", &frames);
    let leaf = atom("leaf", &frames);
    root.set_start_pts(1.);
    root.set_own_duration(4.);
    child.set_start_pts(0.5);
    child.set_own_duration(2.);
    leaf.set_start_pts(0.25);
    leaf.set_own_duration(1.);
    child.add(&leaf.cast()).unwrap();
    root.add(&child.cast()).unwrap();
    root.start(true).unwrap();
    assert!(frames.read().is_empty());
    assert!(root.update(2.).unwrap());
    let recorded = frames.read();
    assert_eq!(
        recorded.iter().map(|f| f.name.as_str()).collect::<Vec<_>>(),
        ["leaf", "child", "root"]
    );
    assert_eq!(
        recorded
            .iter()
            .map(|f| f.progress.local_pts)
            .collect::<Vec<_>>(),
        [0.25, 0.5, 1.]
    );
    assert_eq!(
        recorded
            .iter()
            .map(|f| f.progress.local_normalized)
            .collect::<Vec<_>>(),
        [0.25, 0.25, 0.25]
    );
    assert!(recorded.iter().all(|f| f.progress.pts == 2. && !f.stopped));
}
#[test]
fn final_frame_stops_children_before_own_callback_and_pause_stops_children() {
    let frames = Arc::new(SharedState::new(Vec::new()));
    let root = atom("root", &frames);
    let child = atom("child", &frames);
    root.set_own_duration(2.);
    child.set_own_duration(1.);
    root.add(&child.cast()).unwrap();
    root.start(false).unwrap();
    assert!(!root.update(3.).unwrap());
    assert!(
        frames
            .read()
            .iter()
            .all(|f| f.stopped && f.progress.local_normalized == 1.)
    );
    assert_eq!((*root.stop_calls.read(), *child.stop_calls.read()), (1, 2));
    root.start(false).unwrap();
    root.pause().unwrap();
    assert!(root.is_stopped() && child.is_stopped());
    assert_eq!(
        root.animation_base().fields.read().state,
        AisAnimationState::Paused
    );
    let count = frames.read().len();
    assert!(root.update(0.5).unwrap());
    assert_eq!(frames.read().len(), count);
}
#[test]
fn live_child_cursor_visits_an_appended_started_child_during_same_frame() {
    let frames = Arc::new(SharedState::new(Vec::new()));
    let root = atom("root", &frames);
    let first = atom("first", &frames);
    let appended = atom("appended", &frames);
    for child in [&root, &first, &appended] {
        child.set_own_duration(10.);
    }
    root.add(&first.cast()).unwrap();
    appended.start(false).unwrap();
    *first.append_parent.write() = root.cast();
    *first.append_child.write() = appended.cast();
    root.start(false).unwrap();
    root.update(0.5).unwrap();
    assert_eq!(
        frames
            .read()
            .iter()
            .map(|f| f.name.as_str())
            .collect::<Vec<_>>(),
        ["first", "appended", "root"]
    );
    *first.append_parent.write() = Handle::default();
}
#[test]
fn nan_and_signed_zero_follow_cpp_comparison_order() {
    let frames = Arc::new(SharedState::new(Vec::new()));
    let root = atom("root", &frames);
    root.set_own_duration(-0.);
    assert_eq!(root.duration().to_bits(), (-0_f64).to_bits());
    root.set_own_duration(f64::NAN);
    assert!(root.duration().is_nan());
    root.start(false).unwrap();
    assert!(!root.update(f64::NAN).unwrap());
    let frame = frames.read()[0].clone();
    assert!(frame.progress.local_pts.is_nan());
    assert_eq!(frame.progress.local_normalized, 0.);
    assert!(!frame.stopped);
}
#[test]
fn derived_atoms_preserve_native_animation_rtti_and_handle_downcasts() {
    let frames = Arc::new(SharedState::new(Vec::new()));
    let derived = atom("derived", &frames);
    assert!(derived.is_kind_name("AIS_Animation"));
    assert!(derived.is_kind_name("Standard_Transient"));
    let source = derived.cast::<dyn AisAnimationSource>();
    assert!(!source.is_null());
    assert_eq!(source.cast::<RecordingAnimation>(), derived);
    assert!(source.cast::<AisAnimation>().is_null());
    let container = timeline("container");
    assert_eq!(container.dynamic_type().unwrap().name(), "AIS_Animation");
    assert_eq!(
        container
            .cast::<dyn AisAnimationSource>()
            .cast::<AisAnimation>(),
        container
    );
}
