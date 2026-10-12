//! Null failure state and live borrowed argument replacements use real owner fields.
use super::{viewer_light_probe_replay::make_light_probe_scene, *};
use rustycad_tkernel::standard::{Handle, HandleFieldRef, catch_native_failure};
use rustycad_tkservice::graphic3d::{Graphic3dCLight, Graphic3dLightSet};
#[test]
fn null_membership_and_set_lights_caller_replacement_replay_native() {
    let mut rows = capture_repeated_null_membership();
    rows.extend(capture_set_argument_replacement());
    assert_eq!(
        rows.join("\n") + "\n",
        include_str!("../../tests/fixtures/viewer-light-edge-cases-native.txt")
    );
}
fn capture_repeated_null_membership() -> Vec<String> {
    let scene = make_light_probe_scene(0);
    scene.viewer.state.write().defined_lights.clear();
    scene
        .viewer
        .add_light(&Handle::<Graphic3dCLight>::default());
    (0..2)
        .map(|step| null_failure_row(step, &scene.views[0]))
        .collect()
}
fn null_failure_row(step: usize, view: &Handle<dyn V3dViewSource>) -> String {
    let failure = catch_native_failure(|| {
        view.set_light_on();
        Ok(())
    })
    .unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_ProgramError");
    let member = view.is_active_light(&Handle::<Graphic3dCLight>::default()) as u8;
    let null_set = view.view_ref().observe().lights_ref().is_null() as u8;
    format!(
        "null {step} {} {member} {null_set}",
        view.active_lights_ref().len()
    )
}
fn capture_set_argument_replacement() -> Vec<String> {
    let scene = make_light_probe_scene(12);
    let first = Handle::new(Graphic3dLightSet::default());
    let second = Handle::new(Graphic3dLightSet::default());
    *scene.control.caller_set.write() = first.clone();
    *scene.control.replacement_set.write() = second.downgrade();
    *scene.control.armed.write() = true;
    let caller = HandleFieldRef::new(&scene.control.caller_set, |slot| slot);
    scene.views[0].view_ref().observe().set_lights(&caller);
    let mut rows = scene.control.rows.read().clone();
    rows.push(set_argument_replacement_row(&scene, &first, &second));
    rows
}

fn set_argument_replacement_row(
    scene: &super::viewer_light_probe_replay::LightProbeScene,
    first: &Handle<Graphic3dLightSet>,
    second: &Handle<Graphic3dLightSet>,
) -> String {
    let caller = HandleFieldRef::new(&scene.control.caller_set, |slot| slot);
    let retained = scene.views[0].view_ref().observe();
    let same_caller = (caller.object_address() == second.object_address()) as u8;
    let same_retained = (retained.lights_ref().object_address() == second.object_address()) as u8;
    format!(
        "argument {same_caller} {same_retained} {} {}",
        first.ref_count(),
        second.ref_count()
    )
}
