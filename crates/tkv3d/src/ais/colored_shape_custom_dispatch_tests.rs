//! Actual virtual CustomAspects callbacks precede all custom appearance changes, including failures.
use super::{
    colored_shape_dispatch_fixture::ColoredDispatchFake,
    colored_shape_dispatch_tests::dispatch_scene, *,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle},
};
use std::{fmt::Write, sync::atomic::Ordering};
fn custom_setter_result(
    source: &dyn AisColoredShapeSource,
    key: &Shape,
    setter: i32,
) -> Result<(), Failure> {
    match setter {
        0 => source.set_custom_color(key, Color::named(NamedColor::RED)),
        1 => source.set_custom_width(key, 2.5),
        2 => source.set_custom_transparency(key, 0.4),
        _ => unreachable!("custom setter={setter}; expected 0, 1 or 2"),
    }
}
fn custom_setter_call(object: &Handle<ColoredDispatchFake>, key: &Shape, setter: i32, stage: i32) {
    ais_colored_shape_clear_custom_aspects(&**object);
    object.failure.store(stage, Ordering::Relaxed);
    object.prs.read().set_update_status(false);
    writeln!(object.rows.lock().unwrap(), "setter {setter} {stage}").unwrap();
    let source: Handle<dyn AisColoredShapeSource> = object.cast();
    if let Err(failure) = custom_setter_result(&*source, key, setter) {
        writeln!(
            object.rows.lock().unwrap(),
            "failure {}",
            failure.exception_type()
        )
        .unwrap();
    }
    custom_setter_state(object, key);
}
fn custom_setter_state(object: &ColoredDispatchFake, key: &Shape) {
    let map = object.custom_aspects_map();
    let flags = custom_setter_flags(&map, key);
    writeln!(
        object.rows.lock().unwrap(),
        "setter-state {} {} {} {} {}",
        map.len(),
        i32::from(object.prs.read().must_be_updated()),
        i32::from(flags.has_own_color),
        i32::from(flags.has_own_width),
        i32::from(flags.has_own_transparency)
    )
    .unwrap();
}
#[test]
fn custom_appearance_setters_dispatch_actual_source_callbacks_and_preserve_failure_state() {
    let object = dispatch_scene();
    let key = object.shape_ref().clone_shape();
    for setter in 0..3 {
        for stage in [1, 2, 0] {
            custom_setter_call(&object, &key, setter, stage);
        }
    }
    assert_eq!(
        *object.rows.lock().unwrap(),
        include_str!("../../tests/fixtures/colored-shape-expansion/native-custom-dispatch.txt")
    );
}

fn custom_setter_flags(map: &AisShapeDrawerMapRef<'_>, key: &Shape) -> AisColoredDrawerFlags {
    let drawer = map
        .node_ref(key)
        .ok()
        .map(|node| node.clone_drawer().unwrap());
    drawer
        .as_ref()
        .map(|value| *value.flags.read())
        .unwrap_or_default()
}
