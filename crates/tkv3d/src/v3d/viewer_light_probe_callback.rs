//! Real SetLights callback reads native owner counts before changing actual retained list nodes.
use super::viewer_light_probe_fixture::*;
use rustycad_tkernel::standard::{Failure, HandleReference, raise_native_failure};
use rustycad_tkservice::graphic3d::Graphic3dLightSet;
pub(super) fn record_light_set_callback(
    control: &ViewerLightProbeControl,
    identity: usize,
    implementation_owners: i32,
    lights: &dyn HandleReference<Graphic3dLightSet>,
) {
    if !*control.armed.read() {
        return;
    }
    let ordinal = next_light_callback_ordinal(control);
    append_light_callback_row(control, identity, implementation_owners, lights, ordinal);
    reject_configured_light_callback(control, ordinal);
    if ordinal == 1 {
        edit_light_callback(control);
    }
}
fn probe_set_light_keys(lights: &dyn HandleReference<Graphic3dLightSet>) -> String {
    let observed = lights.observe();
    let Some(lights) = observed.get() else {
        return "null".into();
    };
    // These temporary key copies end before callback counts are observed.
    let keys: String = lights
        .lights()
        .iter()
        .map(|light| format!("{},", String::from_utf8_lossy(light.name().as_bytes())))
        .collect();
    if keys.is_empty() {
        return "-".into();
    }
    keys
}
fn next_light_callback_ordinal(control: &ViewerLightProbeControl) -> usize {
    let mut ordinal = control.ordinal.write();
    *ordinal += 1;
    *ordinal
}
fn append_light_callback_row(
    control: &ViewerLightProbeControl,
    identity: usize,
    implementation_owners: i32,
    lights: &dyn HandleReference<Graphic3dLightSet>,
    ordinal: usize,
) {
    let keys = probe_set_light_keys(lights);
    let target_owners = lights.observe().ref_count();
    control.rows.write().push(format!(
        "set {identity} {ordinal} {implementation_owners} {target_owners} {keys}{}",
        probe_light_counts(control)
    ));
}
fn reject_configured_light_callback(control: &ViewerLightProbeControl, ordinal: usize) {
    if (control.policy == 3 && ordinal == 1) || (control.policy == 4 && ordinal == 2) {
        raise_native_failure::<()>(Err(Failure::new(
            "SetLights callback=failed; expected success",
        )));
    }
}
fn edit_light_callback(control: &ViewerLightProbeControl) {
    match control.policy {
        1 => edit_global_light_append(control),
        2 => replace_global_light_node(control),
        5 => *control.caller.write() = control.views.read()[1].observe().clone(),
        7 => edit_active_views_callback(control),
        8 => remove_defined_light_node(control),
        9 => replace_defined_light_node(control),
        11 => replace_active_view_node(control),
        12 => replace_caller_set_slot(control),
        _ => {}
    }
}
fn edit_global_light_append(control: &ViewerLightProbeControl) {
    let viewer = control.viewer.observe();
    let lights = control.lights.read();
    let mut fields = viewer.state.write();
    super::viewer_light_mutation::remove_light_node(
        &mut fields.active_lights,
        lights[1].object_address(),
    );
    fields.active_lights.append(lights[2].observe().clone());
}
fn replace_global_light_node(control: &ViewerLightProbeControl) {
    let viewer = control.viewer.observe();
    let replacement = control.lights.read()[1].observe().clone();
    *viewer.state.write().active_lights.first_mut().unwrap() = replacement;
}
fn remove_defined_light_node(control: &ViewerLightProbeControl) {
    let viewer = control.viewer.observe();
    let lights = control.lights.read();
    super::viewer_light_mutation::remove_light_node(
        &mut viewer.state.write().defined_lights,
        lights[1].object_address(),
    );
}
fn replace_defined_light_node(control: &ViewerLightProbeControl) {
    let viewer = control.viewer.observe();
    let replacement = control.lights.read()[1].observe().clone();
    *viewer.state.write().defined_lights.first_mut().unwrap() = replacement;
}
fn replace_active_view_node(control: &ViewerLightProbeControl) {
    let viewer = control.viewer.observe();
    let replacement = control.views.read()[1].observe().clone();
    *viewer.state.write().active_views.first_mut().unwrap() = replacement;
}
fn edit_active_views_callback(control: &ViewerLightProbeControl) {
    let viewer = control.viewer.observe();
    let views = control.views.read();
    let mut fields = viewer.state.write();
    super::viewer::remove_view_from(&mut fields.active_views, views[1].object_address());
    fields.active_views.append(views[2].observe().clone());
}

fn replace_caller_set_slot(control: &ViewerLightProbeControl) {
    let replacement = control.replacement_set.read().observe().clone();
    *control.caller_set.write() = replacement;
}

pub(super) fn record_driver_light_limit(
    control: &ViewerLightProbeControl,
    kind: rustycad_tkservice::graphic3d::Graphic3dTypeOfLimit,
    owners: i32,
) {
    if !*control.armed.read()
        || control.policy != 15
        || kind != rustycad_tkservice::graphic3d::Graphic3dTypeOfLimit::MaxNbLights
    {
        return;
    }
    control.rows.write().push(format!("limit {owners}"));
}
