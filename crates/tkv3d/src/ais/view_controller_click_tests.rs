//! Committed local oracle snapshots keep CI independent of native OCCT.
use crate::v3d::V3dViewSource;
use super::*;
use crate::v3d::V3dViewer;
use rustycad_tkernel::{
    ncollection::Vec2,
    standard::{Failure, Handle, SharedState, StandardTransient},
};
use rustycad_tkservice::null_driver::NullGraphicDriver;
pub(super) struct ClickPauseRecorder {
    transient: StandardTransient,
    base: AisAnimationBase,
    pauses: SharedState<i32>,
    fail_pause: bool,
}
rustycad_tkernel::impl_transient!(ClickPauseRecorder,"ClickPauseRecorder",
    dyn AisAnimationSource,transient,interfaces:[dyn AisAnimationSource]);
impl ClickPauseRecorder {
    pub(super) fn new(fail_pause: bool) -> Self {
        Self {
            transient: StandardTransient::default(),
            base: AisAnimationBase::new("click"),
            pauses: SharedState::new(0),
            fail_pause,
        }
    }
}
impl AisAnimationSource for ClickPauseRecorder {
    fn animation_base(&self) -> &AisAnimationBase {
        &self.base
    }
    fn pause(&self) -> Result<(), Failure> {
        *self.pauses.write() += 1;
        if self.fail_pause {
            return Err(Failure::new(
                "click pause failed; expected successful animation pause",
            ));
        }
        super::animation_playback::ais_animation_pause(self)
    }
}
fn native_click_rows(kind: &str) -> Vec<Vec<i32>> {
    include_str!("../../tests/fixtures/view-controller-clicks-native.txt")
        .lines()
        .filter_map(|line| {
            let (name, rest) = line.split_once(' ')?;
            (name == kind).then(|| {
                rest.split_whitespace()
                    .map(|v| v.parse().unwrap())
                    .collect()
            })
        })
        .collect()
}
fn click_timeline(state: i32) -> Handle<ClickPauseRecorder> {
    let animation = Handle::new(ClickPauseRecorder::new(false));
    if state >= 2 {
        animation.start(false).unwrap();
    }
    if state == 2 {
        animation.pause().unwrap();
    }
    *animation.pauses.write() = 0;
    animation
}
fn replay_native_click(row: &[i32]) {
    let mut controller = AisViewController::new().unwrap();
    let animation = click_timeline(row[0]);
    let timeline = if row[0] == 0 {
        Handle::default()
    } else {
        animation.cast::<dyn AisAnimationSource>()
    };
    controller.set_objects_animation(&timeline);
    controller.set_pause_objects_animation(row[1] != 0);
    set_native_click_binding(&mut controller, row);
    let result = controller
        .update_mouse_click(Vec2::new(-7, 13), row[2] as u32, row[3] as u32, row[4] != 0)
        .unwrap();
    assert_click_snapshot(&controller, &animation, result, row);
}
fn set_native_click_binding(c: &mut AisViewController, row: &[i32]) {
    if row[5] != -2 {
        c.change_mouse_selection_schemes().bind(
            (row[2] | row[3]) as u32,
            selection_scheme_from_native(row[5]),
        );
    }
}
fn assert_click_snapshot(c: &AisViewController, a: &ClickPauseRecorder, result: bool, row: &[i32]) {
    let selection = &c.input_buffer(AisViewInputBufferType::Ui).selection;
    let actual = [
        result as i32,
        selection.tool as i32,
        selection.scheme as i32,
        selection.points.len() as i32,
        selection.to_apply_tool as i32,
        a.is_stopped() as i32,
        *a.pauses.read(),
    ];
    assert_eq!(actual, row[6..], "{row:?}");
    if result {
        assert_eq!(selection.points.first(), Some(&Vec2::new(-7, 13)));
    }
}
fn selection_scheme_from_native(value: i32) -> AisSelectionScheme {
    match value {
        -1 => AisSelectionScheme::Unknown,
        0 => AisSelectionScheme::Replace,
        1 => AisSelectionScheme::Add,
        2 => AisSelectionScheme::Remove,
        3 => AisSelectionScheme::Xor,
        4 => AisSelectionScheme::Clear,
        5 => AisSelectionScheme::ReplaceExtra,
        _ => panic!("selection scheme {value}; expected -1..=5"),
    }
}
#[test]
fn clicks_replay_all_native_modifier_double_click_and_animation_states() {
    let rows = native_click_rows("click");
    assert_eq!(rows.len(), 158);
    for row in rows {
        replay_native_click(&row);
    }
}
#[test]
fn native_default_timelines_are_nonnull_stopped_and_match_snapshot() {
    let c = AisViewController::new().unwrap();
    let actual = format!(
        "defaults {} {} {} {} {} {} {} {}",
        c.view_animation().is_null() as i32,
        c.objects_animation().is_null() as i32,
        c.view_animation().is_stopped() as i32,
        c.objects_animation().is_stopped() as i32,
        c.view_animation().own_duration(),
        c.objects_animation().own_duration(),
        c.view_animation().target_view().is_null() as i32,
        c.mouse_selection_schemes().extent()
    );
    let row = include_str!("../../tests/fixtures/view-controller-clicks-native.txt")
        .lines()
        .next()
        .unwrap();
    assert_eq!(actual, row);
}
#[test]
fn default_timelines_have_native_names_and_independent_handle_identity() {
    let first = AisViewController::new().unwrap();
    let second = AisViewController::new().unwrap();
    assert_eq!(
        first.view_animation().name().as_bytes(),
        b"AIS_ViewController_ViewAnimation"
    );
    assert_eq!(
        first.objects_animation().name().as_bytes(),
        b"AIS_ViewController_ObjectsAnimation"
    );
    assert!(first.view_animation() != second.view_animation());
    assert!(first.objects_animation() != second.objects_animation());
}
fn click_camera_target() -> Handle<dyn V3dViewSource> {
    V3dViewer::new(&NullGraphicDriver::new_handle())
        .unwrap()
        .create_view()
        .unwrap()
}
#[test]
fn abort_camera_replays_native_running_paused_stopped_and_null_handles() {
    for row in native_click_rows("camera") {
        replay_camera_abort(&row);
    }
}
fn replay_camera_abort(row: &[i32]) {
    let mut controller = AisViewController::new().unwrap();
    let target = click_camera_target();
    let animation = Handle::new(AisAnimationCamera::new("camera", &target));
    seed_camera_abort(&mut controller, &animation, row[0]);
    controller.abort_view_animation().unwrap();
    assert_eq!(
        [
            controller.view_animation().is_null() as i32,
            animation.is_stopped() as i32,
            animation.target_view().is_null() as i32
        ],
        row[1..]
    );
    if row[3] == 0 {
        assert_eq!(animation.target_view(), target);
    }
    controller.abort_view_animation().unwrap();
}
fn seed_camera_abort(c: &mut AisViewController, a: &Handle<AisAnimationCamera>, state: i32) {
    c.set_view_animation(&if state == 0 {
        Handle::default()
    } else {
        a.clone()
    });
    if state >= 2 {
        a.start(false).unwrap();
    }
    if state == 2 {
        a.pause().unwrap();
    }
}
#[test]
fn timeline_setters_retain_identity_and_accept_null_without_stopping_previous_owner() {
    let mut controller = AisViewController::new().unwrap();
    let camera = controller.view_animation().clone();
    let objects = controller.objects_animation().clone();
    camera.start(false).unwrap();
    objects.start(false).unwrap();
    controller.set_view_animation(&camera);
    controller.set_objects_animation(&objects);
    assert_eq!(controller.view_animation(), &camera);
    assert_eq!(controller.objects_animation(), &objects);
    controller.set_view_animation(&Handle::default());
    controller.set_objects_animation(&Handle::default());
    assert!(controller.view_animation().is_null() && controller.objects_animation().is_null());
    assert!(!camera.is_stopped() && !objects.is_stopped());
}
#[test]
fn selection_map_edits_share_storage_and_clear_removes_default_bindings() {
    let mut c = AisViewController::new().unwrap();
    let address = c.mouse_selection_schemes() as *const _;
    assert_eq!(address, c.change_mouse_selection_schemes() as *const _);
    c.change_mouse_selection_schemes().clear(false);
    assert!(
        !c.update_mouse_click(Vec2::new(2, 3), 8192, 0, false)
            .unwrap()
    );
}
#[test]
fn custom_clicks_keep_duplicate_picks_and_do_not_mutate_render_queue() {
    let mut c = AisViewController::new().unwrap();
    c.change_mouse_selection_schemes()
        .bind(3, AisSelectionScheme::Add);
    for double in [false, true] {
        assert!(c.update_mouse_click(Vec2::new(2, 3), 1, 2, double).unwrap());
    }
    let ui = &c.input_buffer(AisViewInputBufferType::Ui).selection;
    let gl = &c.input_buffer(AisViewInputBufferType::Gl).selection;
    assert_eq!(ui.points.len(), 2);
    assert!(gl.points.is_empty());
}
#[test]
fn failing_polymorphic_pause_precedes_selection_and_preserves_queued_points() {
    let mut controller = AisViewController::new().unwrap();
    let animation = Handle::new(ClickPauseRecorder::new(true));
    animation.start(false).unwrap();
    controller.set_objects_animation(&animation.cast::<dyn AisAnimationSource>());
    controller.set_pause_objects_animation(true);
    let result = controller.update_mouse_click(Vec2::new(1, 2), 8192, 0, false);
    assert!(result.is_err());
    assert_eq!(*animation.pauses.read(), 1);
    let selection = &controller
        .input_buffer(AisViewInputBufferType::Ui)
        .selection;
    assert!(selection.points.is_empty());
    assert!(!animation.is_stopped());
}
