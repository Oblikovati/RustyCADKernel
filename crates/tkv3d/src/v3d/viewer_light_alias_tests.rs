//! Native light-list and retained driver-field aliases observe replacement without ownership snapshots.
use super::{viewer_light_probe_replay::make_light_probe_scene, *};
use rustycad_tkernel::standard::{Handle, HandleReference};
use rustycad_tkservice::{
    graphic3d::{Graphic3dCLight, Graphic3dLightSet, Graphic3dTypeOfLightSource},
    null_driver::NullGraphicDriver,
};
#[test]
fn retained_light_node_and_driver_slot_aliases_replay_native_counts() {
    let mut rows = capture_light_node_aliases();
    rows.extend(capture_driver_set_aliases());
    assert_eq!(
        rows.join("\n") + "\n",
        include_str!("../../tests/fixtures/viewer-light-aliases-native.txt")
    );
}
fn capture_light_node_aliases() -> Vec<String> {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let lights = define_alias_lights(&viewer);
    viewer.init_active_lights();
    let mut cursor = viewer.active_light_iterator();
    let alias = cursor.value_ref();
    let fresh = viewer.active_light_ref();
    let mut rows = vec![light_node_alias_row(0, &alias, &fresh, &lights)];
    *viewer.state.write().active_lights.first_mut().unwrap() = lights[1].clone();
    rows.push(light_node_alias_row(1, &alias, &fresh, &lights));
    viewer.state.write().active_lights.append(lights[2].clone());
    cursor.next();
    cursor.next();
    let last = cursor.value_ref();
    rows.push(light_node_alias_row(2, &last, &last, &lights));
    rows
}
fn define_alias_lights(viewer: &V3dViewer) -> [Handle<Graphic3dCLight>; 3] {
    let lights = std::array::from_fn(|_| {
        Handle::new(Graphic3dCLight::new(Graphic3dTypeOfLightSource::Ambient))
    });
    for light in &lights {
        viewer.add_light(light);
    }
    viewer.set_light_on_one(&lights[0]).unwrap();
    viewer.set_light_on_one(&lights[1]).unwrap();
    lights
}
fn light_node_alias_row(
    step: usize,
    alias: &dyn HandleReference<Graphic3dCLight>,
    fresh: &dyn HandleReference<Graphic3dCLight>,
    lights: &[Handle<Graphic3dCLight>; 3],
) -> String {
    let same_slot = (alias.reference_address() == fresh.reference_address()) as u8;
    let first = (alias.object_address() == lights[0].object_address()) as u8;
    let second = (alias.object_address() == lights[1].object_address()) as u8;
    format!(
        "node {step} {same_slot} {first} {second} {} {} {}",
        lights[0].ref_count(),
        lights[1].ref_count(),
        lights[2].ref_count()
    )
}
fn capture_driver_set_aliases() -> Vec<String> {
    let scene = make_light_probe_scene(0);
    let implementation = scene.views[0].view_ref().observe();
    let alias = implementation.lights_ref();
    let first = Handle::new(Graphic3dLightSet::default());
    let second = Handle::new(Graphic3dLightSet::default());
    let mut rows = vec![driver_set_alias_row(
        0,
        &*alias,
        &*implementation.lights_ref(),
        [&first, &second],
    )];
    append_driver_alias_steps(
        implementation.get().unwrap(),
        &*alias,
        [&first, &second],
        &mut rows,
    );
    rows
}
fn append_driver_alias_steps(
    implementation: &dyn rustycad_tkservice::graphic3d::Graphic3dCView,
    alias: &dyn HandleReference<Graphic3dLightSet>,
    targets: [&Handle<Graphic3dLightSet>; 2],
    rows: &mut Vec<String>,
) {
    let null = Handle::default();
    let assignments: [&dyn HandleReference<Graphic3dLightSet>; 4] =
        [targets[0], alias, targets[1], &null];
    for (index, target) in assignments.into_iter().enumerate() {
        implementation.set_lights(target);
        rows.push(driver_set_alias_row(
            index + 1,
            alias,
            &*implementation.lights_ref(),
            targets,
        ));
    }
}
fn driver_set_alias_row(
    step: usize,
    alias: &dyn HandleReference<Graphic3dLightSet>,
    fresh: &dyn HandleReference<Graphic3dLightSet>,
    targets: [&Handle<Graphic3dLightSet>; 2],
) -> String {
    let same_slot = (alias.reference_address() == fresh.reference_address()) as u8;
    let first = (alias.object_address() == targets[0].object_address()) as u8;
    let second = (alias.object_address() == targets[1].object_address()) as u8;
    format!(
        "set-alias {step} {same_slot} {first} {second} {} {}",
        targets[0].ref_count(),
        targets[1].ref_count()
    )
}
#[test]
fn light_field_references_copy_without_owners_and_cursor_identity_survives_append() {
    let scene = make_light_probe_scene(0);
    let original = scene.viewer.active_lights_ref();
    let copied = original;
    assert_eq!(original.reference_address(), copied.reference_address());
    let cursor = original.cursor();
    let node = cursor.node_address();
    let alias = cursor.value_ref();
    let before = alias.observe().ref_count();
    scene.viewer.set_light_on_one(&scene.lights[2]).unwrap();
    assert_eq!(node, cursor.node_address());
    assert_eq!(before, alias.observe().ref_count());
    assert_eq!(copied.len(), 3);
    assert!(!copied.is_empty());
}
