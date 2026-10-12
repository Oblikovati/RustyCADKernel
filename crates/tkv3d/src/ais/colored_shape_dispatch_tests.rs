//! Native virtual before/after failure order against the real owned container.
use super::{colored_shape_dispatch_fixture::ColoredDispatchFake, *};
use crate::{prs_mgr::*, select_mgr::SelectMgrSelectableObject};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::{graphic3d::Graphic3dStructureManager, null_driver::NullGraphicDriver};
use std::{fmt::Write, sync::atomic::Ordering};
pub(super) fn dispatch_scene() -> Handle<ColoredDispatchFake> {
    let key = shape_drawer_oracle_inputs::shape_drawer_vertex(Point3::new(1., 2., 3.));
    let object = ColoredDispatchFake::new(&key);
    let structures =
        Handle::new(Graphic3dStructureManager::new(&NullGraphicDriver::new_handle()).unwrap());
    let manager = PrsMgrPresentationManager::new(&structures);
    let prs = PrsMgrPresentation::new(&manager, &object.clone().cast(), 0).unwrap();
    object
        .presentable_base()
        .fields
        .write()
        .presentations
        .push(prs.clone());
    *object.prs.write() = prs;
    object
}
fn dispatch_failure(object: &ColoredDispatchFake, result: Result<(), Failure>, label: &str) {
    if let Err(error) = result {
        writeln!(
            object.rows.lock().unwrap(),
            "failure {}",
            error.exception_type()
        )
        .unwrap();
    }
    object.event(label);
}
fn call_custom(object: &Handle<ColoredDispatchFake>, stage: i32) {
    object.failure.store(stage, Ordering::Relaxed);
    object.prs.read().set_update_status(false);
    let source: Handle<dyn AisColoredShapeSource> = object.cast();
    let key = source.shape_ref().clone_shape();
    let result = source.custom_aspects(&key).map(|_| ());
    dispatch_failure(object, result, "custom-final");
}
fn call_clear(object: &Handle<ColoredDispatchFake>, stage: i32) {
    object.failure.store(stage, Ordering::Relaxed);
    object.prs.read().set_update_status(false);
    let source: Handle<dyn AisColoredShapeSource> = object.cast();
    let result = source.clear_custom_aspects();
    dispatch_failure(object, result, "clear-final");
}
#[test]
fn actual_virtual_custom_clear_failure_order_replays_native() {
    let object = dispatch_scene();
    record_runtime_views(&object);
    for stage in [1, 2, 0] {
        call_custom(&object, stage);
    }
    for stage in [1, 2, 0] {
        call_clear(&object, stage);
    }
    object.clear_custom_aspects().unwrap();
    assert_eq!(
        *object.rows.lock().unwrap(),
        include_str!("../../tests/fixtures/colored-shape/native-dispatch.txt")
    );
}

fn record_runtime_views(object: &Handle<ColoredDispatchFake>) {
    let parent: Handle<dyn AisShapeSource> = object.cast();
    let colored: Handle<dyn AisColoredShapeSource> = parent.cast();
    let names = dispatch_runtime_names(object);
    let same = i32::from(colored.object_address() == object.object_address());
    writeln!(
        object.rows.lock().unwrap(),
        "runtime {names} {same} {}",
        object.ref_count()
    )
    .unwrap();
    drop(parent);
    drop(colored);
    writeln!(
        object.rows.lock().unwrap(),
        "runtime-release {}",
        object.ref_count()
    )
    .unwrap();
}

fn dispatch_runtime_names(object: &Handle<ColoredDispatchFake>) -> String {
    let descriptor = object.dynamic_type().unwrap();
    let parent = descriptor.parent().unwrap();
    format!(
        "{} {} {}",
        descriptor.name(),
        parent.name(),
        parent.parent().unwrap().name()
    )
}

#[test]
fn named_lifecycle_probe_rejects_geometry_io_without_substituting_it() {
    let object = dispatch_scene();
    let presentation = object.prs.read().clone().cast();
    let error = object
        .compute(&Handle::default(), &presentation, 19)
        .unwrap_err();
    assert!(error.message().contains("mode=19"));
    let error = object
        .compute_selection(&Handle::default(), 20)
        .unwrap_err();
    assert!(error.message().contains("mode=20"));
}
