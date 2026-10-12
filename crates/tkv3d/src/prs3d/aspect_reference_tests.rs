//! Pinned native const-field traces distinguish live slots from owning getter snapshots.
use super::*;
use rustycad_tkernel::{
    quantity::Color,
    standard::{Handle, HandleFieldRef, RuntimeTyped},
};
use rustycad_tkservice::aspect::{AspectTypeOfLine, AspectTypeOfMarker};

fn replay_aspect_slot<T: RuntimeTyped>(
    label: &str,
    slot: HandleFieldRef<'_, Handle<T>, T>,
    alias: HandleFieldRef<'_, Handle<T>, T>,
    incoming: Handle<T>,
    replace: impl Fn(Handle<T>),
) {
    let previous = slot.clone_handle();
    let mut rows = aspect_before_row(label, slot.is_same_reference(&alias), previous.ref_count());
    replace(incoming.clone());
    rows.push_str(&aspect_replaced_row(label, slot, &previous, &incoming));
    replace(Handle::default());
    rows.push_str(&aspect_null_row(label, slot, &incoming));
    assert_eq!(rows, aspect_native_rows(label));
}
fn aspect_before_row(label: &str, same: bool, count: i32) -> String {
    // Slot identity, unlike target identity, remains unchanged through setter replacement.
    format!("{label} before {} {count}\n", i32::from(same))
}
fn aspect_replaced_row<T: RuntimeTyped>(
    label: &str,
    slot: HandleFieldRef<'_, Handle<T>, T>,
    previous: &Handle<T>,
    incoming: &Handle<T>,
) -> String {
    // Old and incoming objects retain only the explicit owners counted by the native fixture.
    let same = slot.object_address() == incoming.object_address();
    let counts = (previous.ref_count(), incoming.ref_count());
    format!(
        "{label} replaced {} {} {}\n",
        i32::from(same),
        counts.0,
        counts.1
    )
}
fn aspect_null_row<T: RuntimeTyped>(
    label: &str,
    slot: HandleFieldRef<'_, Handle<T>, T>,
    incoming: &Handle<T>,
) -> String {
    // An existing const-slot alias observes clearing without being recreated.
    format!(
        "{label} null {} {}\n",
        i32::from(slot.is_null()),
        incoming.ref_count()
    )
}
fn aspect_native_rows(label: &str) -> String {
    let native = include_str!("../../tests/fixtures/aspect-slot-reference/native.txt");
    native
        .lines()
        .filter(|line| line.starts_with(label))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

#[test]
fn line_aspect_slot_replays_native_replacement_and_counts() {
    let line = Prs3dLineAspect::new(Color::default(), AspectTypeOfLine::Solid, 1.).unwrap();
    let next = Prs3dLineAspect::new(Color::default(), AspectTypeOfLine::Solid, 1.).unwrap();
    replay_aspect_slot(
        "line",
        line.aspect_ref(),
        line.aspect_ref(),
        next.aspect(),
        |value| line.set_aspect(value),
    );
}
#[test]
fn point_aspect_slot_replays_native_replacement_and_counts() {
    let point = Prs3dPointAspect::new(AspectTypeOfMarker::Point, Color::default(), 1.).unwrap();
    let next = Prs3dPointAspect::new(AspectTypeOfMarker::Point, Color::default(), 1.).unwrap();
    replay_aspect_slot(
        "point",
        point.aspect_ref(),
        point.aspect_ref(),
        next.aspect(),
        |value| point.set_aspect(value),
    );
}
#[test]
fn shading_aspect_slot_replays_native_replacement_and_counts() {
    let shading = Prs3dShadingAspect::new().unwrap();
    let next = Prs3dShadingAspect::new().unwrap();
    replay_aspect_slot(
        "shading",
        shading.aspect_ref(),
        shading.aspect_ref(),
        next.aspect(),
        |value| shading.set_aspect(value),
    );
}
#[test]
fn aspect_slot_callback_releases_lock_and_does_not_retain_target() {
    let line = Prs3dLineAspect::new(Color::default(), AspectTypeOfLine::Solid, 1.).unwrap();
    let owned = line.aspect();
    let reference = line.aspect_ref();
    reference.with_object(|value| {
        assert!(value.is_some());
        assert_eq!(owned.ref_count(), 2);
        line.set_aspect(Handle::default());
        assert_eq!(owned.ref_count(), 1);
    });
    assert!(reference.is_null());
}
