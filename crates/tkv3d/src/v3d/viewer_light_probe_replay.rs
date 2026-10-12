//! Strong native owners outlive every borrowed alias while real callback bodies mutate live nodes.
use super::{view_camera_alias_fixture::*, viewer_light_probe_fixture::*, *};
use rustycad_tkernel::{
    standard::{Failure, Handle, HandleFieldRef, HandleReference, catch_native_failure},
    tcollection::AsciiString,
};
use rustycad_tkservice::{
    graphic3d::{Graphic3dCLight, Graphic3dTypeOfLightSource},
    null_driver::NullGraphicDriver,
};
use std::sync::Arc;
pub(super) struct LightProbeScene {
    pub viewer: Handle<V3dViewer>,
    pub control: Arc<ViewerLightProbeControl>,
    pub lights: [Handle<Graphic3dCLight>; 3],
    pub views: [Handle<dyn V3dViewSource>; 3],
}
pub(super) fn make_light_probe_scene(policy: usize) -> LightProbeScene {
    let alias = fresh_camera_alias_control();
    let driver = camera_alias_driver(&NullGraphicDriver::new_handle(), &alias);
    let viewer = V3dViewer::new(&driver).unwrap();
    let control = fresh_light_probe_control(&viewer, policy);
    *alias.light_probe.write() = Some(control.clone());
    let lights = make_probe_lights(policy);
    let views = std::array::from_fn(|index| mapped_light_probe_view(&viewer, &control, index + 1));
    *control.lights.write() = lights.each_ref().map(Handle::downgrade);
    *control.views.write() = views.each_ref().map(Handle::downgrade);
    *control.caller.write() = views[0].clone();
    let scene = LightProbeScene {
        viewer,
        control,
        lights,
        views,
    };
    prepare_light_case(&scene);
    scene
}
fn make_probe_lights(policy: usize) -> [Handle<Graphic3dCLight>; 3] {
    std::array::from_fn(|index| {
        if policy == 6 && index == 0 {
            return Handle::default();
        }
        let light = Handle::new(Graphic3dCLight::new(Graphic3dTypeOfLightSource::Ambient));
        light.set_name(&AsciiString::from((index + 1).to_string().as_str()));
        light
    })
}
fn prepare_light_case(scene: &LightProbeScene) {
    for light in &scene.lights {
        scene.viewer.add_light(light);
    }
    if scene.control.policy == 8 || scene.control.policy == 9 {
        scene.viewer.set_view_on_one(&scene.views[0]).unwrap();
        return;
    }
    scene.viewer.set_light_on_one(&scene.lights[0]).unwrap();
    scene.viewer.set_light_on_one(&scene.lights[1]).unwrap();
    if scene.control.policy == 7 || scene.control.policy == 11 {
        scene.viewer.set_view_on_one(&scene.views[0]).unwrap();
        scene.viewer.set_view_on_one(&scene.views[1]).unwrap();
    }
    scene.views[0].set_immediate_update(true);
}
pub(super) fn capture_light_case(policy: usize) -> Vec<String> {
    let scene = make_light_probe_scene(policy);
    let implementation = scene.views[0].view_ref().observe();
    let alias = implementation.lights_ref();
    *scene.control.armed.write() = true;
    scene.views[0].set_immediate_update(true);
    let result = execute_light_case(&scene);
    let mut rows = vec![format!("case {policy}")];
    rows.extend(scene.control.rows.read().iter().cloned());
    rows.push(probe_result_row(result));
    rows.push(probe_case_lists(&scene));
    rows.push(probe_case_counts(&scene, &*alias));
    rows
}
fn execute_light_case(scene: &LightProbeScene) -> Result<(), Failure> {
    if scene.control.policy == 7 || scene.control.policy == 11 {
        return catch_native_failure(|| {
            scene.viewer.update_lights();
            Ok(())
        });
    }
    if scene.control.policy == 8 || scene.control.policy == 9 {
        return scene.viewer.set_light_on();
    }
    if scene.control.policy == 10 {
        return scene.viewer.set_view_on();
    }
    let caller = HandleFieldRef::new(&scene.control.caller, |slot| slot);
    scene.viewer.set_view_on_one(&caller)
}
fn probe_result_row(result: Result<(), Failure>) -> String {
    match result {
        Ok(()) => "result ok".into(),
        Err(failure) => format!(
            "result {} {}",
            failure.exception_type(),
            failure
                .message()
                .strip_suffix("; expected a light")
                .unwrap_or(failure.message())
        ),
    }
}
fn probe_case_lists(scene: &LightProbeScene) -> String {
    let global = probe_list_light_keys(scene.viewer.active_lights_ref());
    let first = probe_list_light_keys(scene.views[0].active_lights_ref());
    let second = probe_list_light_keys(scene.views[1].active_lights_ref());
    format!(
        "lists {global} {first} {second} {}",
        probe_active_view_keys(scene)
    )
}
fn probe_case_counts(
    scene: &LightProbeScene,
    alias: &dyn HandleReference<rustycad_tkservice::graphic3d::Graphic3dLightSet>,
) -> String {
    let view_counts: String = scene
        .views
        .iter()
        .map(|view| format!(" {}", view.ref_count()))
        .collect();
    format!(
        "counts{}{view_counts} {}",
        probe_light_counts(&scene.control),
        alias.observe().ref_count()
    )
}
fn probe_active_view_keys(scene: &LightProbeScene) -> String {
    let mut cursor = scene.viewer.active_view_iterator();
    let mut keys = String::new();
    while cursor.more() {
        let address = cursor.value_ref().object_address();
        for (index, view) in scene.views.iter().enumerate() {
            if view.object_address() == address {
                keys.push_str(&format!("{},", index + 1));
            }
        }
        cursor.next();
    }
    if keys.is_empty() {
        return "-".into();
    }
    keys
}
