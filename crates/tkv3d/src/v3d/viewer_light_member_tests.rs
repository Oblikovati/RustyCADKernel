//! Real membership callbacks preserve global-light rejection, cursors, and default reset counts.
use super::{viewer_light_probe_fixture::probe_list_light_keys, viewer_light_probe_replay::*};
use rustycad_tkernel::standard::HandleReference;
#[test]
fn native_membership_failure_order_cursors_and_defaults_replay() {
    let scene = make_light_probe_scene(0);
    scene.viewer.set_light_off().unwrap();
    scene.viewer.set_view_on_one(&scene.views[0]).unwrap();
    let mut rows = initial_membership_rows(&scene);
    rows.extend(deactivate_and_delete_rows(&scene));
    rows.push(native_light_cursor_row(&scene));
    rows.extend(reset_membership_rows(&scene));
    assert_eq!(
        rows.join("\n") + "\n",
        include_str!("../../tests/fixtures/viewer-light-members-native.txt")
    );
}
fn initial_membership_rows(scene: &LightProbeScene) -> Vec<String> {
    let view = &scene.views[0];
    view.set_light_on();
    let mut rows = vec![membership_row(0, scene)];
    scene.viewer.set_light_on_one(&scene.lights[0]).unwrap();
    rows.push(membership_row(1, scene));
    let failure = view.set_light_off_one(&scene.lights[0]).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_TypeMismatch");
    rows.push("reject view-global".into());
    view.set_light_off();
    rows.push(membership_row(2, scene));
    let failure = scene.viewer.set_light_off().unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_TypeMismatch");
    rows.push("reject viewer-global".into());
    rows.push(membership_row(3, scene));
    rows
}
fn deactivate_and_delete_rows(scene: &LightProbeScene) -> Vec<String> {
    scene.viewer.set_light_off_one(&scene.lights[0]).unwrap();
    let mut rows = vec![membership_row(4, scene)];
    scene.viewer.set_light_on().unwrap();
    scene.viewer.del_light(&scene.lights[1]).unwrap();
    rows.push(membership_row(5, scene));
    rows
}
fn membership_row(step: usize, scene: &LightProbeScene) -> String {
    let defined = probe_list_light_keys(scene.viewer.defined_lights_ref());
    let global = probe_list_light_keys(scene.viewer.active_lights_ref());
    let active = probe_list_light_keys(scene.views[0].active_lights_ref());
    let counts: String = scene
        .lights
        .iter()
        .map(|light| format!(" {}", light.ref_count()))
        .collect();
    format!("members {step} {defined} {global} {active}{counts}")
}
fn native_light_cursor_row(scene: &LightProbeScene) -> String {
    scene.views[0].init_active_lights();
    scene.viewer.init_active_lights();
    scene.viewer.init_defined_lights();
    scene.views[0].next_active_lights();
    scene.viewer.next_active_lights();
    scene.viewer.next_defined_lights();
    let view = scene.views[0].active_light_ref().observe();
    let active = scene.viewer.active_light_ref().observe();
    let defined = scene.viewer.defined_light_ref().observe();
    format!(
        "cursors {} {} {} {} {} {}",
        light_alias_name(&view),
        light_alias_name(&active),
        light_alias_name(&defined),
        scene.views[0].more_active_lights() as u8,
        scene.viewer.more_active_lights() as u8,
        scene.viewer.more_defined_lights() as u8
    )
}
fn light_alias_name(
    light: &rustycad_tkernel::standard::HandleObserver<
        rustycad_tkservice::graphic3d::Graphic3dCLight,
    >,
) -> String {
    String::from_utf8_lossy(light.name().as_bytes()).into_owned()
}
fn reset_membership_rows(scene: &LightProbeScene) -> Vec<String> {
    scene.viewer.set_view_off_one(&scene.views[0]);
    scene.viewer.set_light_off().unwrap();
    scene.views[0].set_light_off();
    let mut rows = vec![membership_row(6, scene)];
    scene.viewer.set_default_lights().unwrap();
    rows.push(membership_row(7, scene));
    rows
}
