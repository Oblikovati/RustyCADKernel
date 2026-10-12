//! Native virtual Camera slot selection is independent of previously retained constHandle aliases.
use super::view_camera_alias_fixture::*;
use super::*;
use rustycad_tkernel::standard::{Handle, HandleFieldRef, HandleReference};
use rustycad_tkservice::{
    graphic3d::{Graphic3dCViewMembers as _, Graphic3dCamera},
    null_driver::NullGraphicDriver,
};
#[test]
fn protected_driver_view_replacement_updates_view_alias_without_rebinding_old_camera_alias() {
    let (_control, viewer, owner) = camera_borrow_scenario();
    let original = owner.view_ref().clone_handle();
    let next = viewer
        .driver()
        .create_view(&viewer.structure_manager())
        .unwrap();
    let view_alias = owner.view_ref();
    let camera_alias = owner.camera_ref();
    owner.driver_view_slot.assign(&next).unwrap();
    assert_replaced_driver_alias(&*owner, view_alias, &next);
    assert_eq!(original.ref_count(), 1);
    assert_eq!(next.ref_count(), 2);
    assert_camera_alias_stays_at_original_driver(&*owner, &original, &*camera_alias);
}
fn assert_replaced_driver_alias(
    owner: &dyn V3dViewSource,
    alias: &dyn HandleReference<dyn rustycad_tkservice::graphic3d::Graphic3dCView>,
    next: &Handle<dyn rustycad_tkservice::graphic3d::Graphic3dCView>,
) {
    assert_eq!(alias.object_address(), next.object_address());
    assert_eq!(
        alias.reference_address(),
        owner.view_ref().reference_address()
    );
}
fn assert_camera_alias_stays_at_original_driver(
    owner: &dyn V3dViewSource,
    original: &Handle<dyn rustycad_tkservice::graphic3d::Graphic3dCView>,
    camera_alias: &dyn HandleReference<Graphic3dCamera>,
) {
    let original_address = original.camera_ref().reference_address();
    let fresh_address = owner.camera_ref().reference_address();
    assert_eq!(camera_alias.reference_address(), original_address);
    assert_ne!(camera_alias.reference_address(), fresh_address);
    assert_eq!(camera_alias.observe().ref_count(), 1);
    assert_eq!(owner.camera_ref().observe().ref_count(), 1);
}

#[test]
fn native_selected_camera_slots_and_intrusive_counts_replay_without_copied_aliases() {
    let (control, _viewer, view) = camera_borrow_scenario();
    let rows = capture_selected_camera_slots(&view, &control);
    assert_eq!(
        rows.join("\n") + "\n",
        include_str!("../../tests/fixtures/v3d-camera-reference-native.txt")
    );
}
fn camera_borrow_scenario() -> (
    std::sync::Arc<CameraAliasControl>,
    Handle<V3dViewer>,
    Handle<dyn V3dViewSource>,
) {
    let (control, viewer) = camera_alias_viewer();
    let view = viewer.create_view().unwrap();
    // The native alias domain retains its containing viewer and original targets.
    (control, viewer, view)
}
fn capture_selected_camera_slots(
    view: &Handle<dyn V3dViewSource>,
    control: &CameraAliasControl,
) -> Vec<String> {
    let original = view.camera();
    let next = Handle::new(Graphic3dCamera::default());
    let retained = view.camera_ref();
    let owners = [&original, &next];
    let mut rows = vec![initial_camera_alias_row(view, &*retained, owners)];
    *control.alternate_selected.write() = true;
    rows.extend(camera_alias_mutation_rows(
        view, control, &*retained, owners,
    ));
    rows
}
fn camera_alias_mutation_rows(
    view: &Handle<dyn V3dViewSource>,
    control: &CameraAliasControl,
    retained: &dyn HandleReference<Graphic3dCamera>,
    owners: [&Handle<Graphic3dCamera>; 2],
) -> Vec<String> {
    let alternate = view.camera_ref();
    let aliases = [retained, &*alternate];
    let mut rows = vec![selected_camera_alias_row(1, view, aliases, owners)];
    *control.alternate.write() = owners[1].clone();
    rows.push(selected_camera_alias_row(2, view, aliases, owners));
    view.set_camera(owners[1]).unwrap();
    rows.push(selected_camera_alias_row(3, view, aliases, owners));
    *control.alternate_selected.write() = false;
    rows.push(selected_camera_alias_row(4, view, aliases, owners));
    rows
}
fn selected_camera_alias_row(
    step: i32,
    view: &Handle<dyn V3dViewSource>,
    aliases: [&dyn HandleReference<Graphic3dCamera>; 2],
    owners: [&Handle<Graphic3dCamera>; 2],
) -> String {
    camera_alias_row(
        step,
        aliases[0],
        aliases[1],
        &*view.camera_ref(),
        owners[0],
        owners[1],
    )
}

fn camera_alias_row(
    step: i32,
    retained: &dyn HandleReference<Graphic3dCamera>,
    alias: &dyn HandleReference<Graphic3dCamera>,
    fresh: &dyn HandleReference<Graphic3dCamera>,
    original: &Handle<Graphic3dCamera>,
    next: &Handle<Graphic3dCamera>,
) -> String {
    let same_retained = (retained.reference_address() == alias.reference_address()) as u8;
    let same_fresh = (alias.reference_address() == fresh.reference_address()) as u8;
    let is_original = (retained.object_address() == original.object_address()) as u8;
    let is_next = (alias.object_address() == next.object_address()) as u8;
    format!(
        "{step} {same_retained} {same_fresh} {is_original} {is_next} {} {}",
        retained.observe().ref_count(),
        alias.observe().ref_count()
    )
}
#[test]
fn dsm_view_default_and_driver_references_retain_actual_native_slot_identity() {
    let control = fresh_camera_alias_control();
    let driver = camera_alias_driver(&NullGraphicDriver::new_handle(), &control);
    let viewer = V3dViewer::new(&driver).unwrap();
    let view = viewer.create_view().unwrap();
    let cview = view.view_ref().observe();
    assert_eq!(
        view.camera_ref().reference_address(),
        cview.camera_ref().reference_address()
    );
    assert_eq!(view.default_camera_ref().observe().ref_count(), 1);
    assert_eq!(view.view_ref().observe().ref_count(), 1);
}
#[test]
fn camera_setter_override_mutates_and_rereads_the_genuine_caller_slot() {
    let (control, _viewer, view) = camera_borrow_scenario();
    let first = Handle::new(Graphic3dCamera::default());
    let next = Handle::new(Graphic3dCamera::default());
    *control.caller.write() = first.clone();
    *control.replacement.write() = next.downgrade();
    *control.replace_camera_argument.write() = true;
    let argument = HandleFieldRef::new(&control.caller, |slot| slot);
    view.set_camera(&argument).unwrap();
    let row = camera_argument_row(&view, &argument, [&first, &next]);
    assert_eq!(
        row,
        include_str!("../../tests/fixtures/v3d-camera-argument-native.txt")
    );
}
fn camera_argument_row(
    view: &Handle<dyn V3dViewSource>,
    argument: &dyn HandleReference<Graphic3dCamera>,
    owners: [&Handle<Graphic3dCamera>; 2],
) -> String {
    let same_slot = (argument.object_address() == owners[1].object_address()) as u8;
    let same_target = (view.camera_ref().object_address() == owners[1].object_address()) as u8;
    format!(
        "camera-argument {same_slot} {same_target} {} {}\n",
        owners[0].ref_count(),
        owners[1].ref_count()
    )
}

#[test]
fn activation_override_retargets_following_nonvirtual_view_operations() {
    use super::view_source_fixture::*;
    let (control, viewer) = camera_alias_viewer();
    let trace = fresh_callback_trace();
    let first = mapped_probe_view(&viewer, 1, &trace);
    let second = mapped_probe_view(&viewer, 2, &trace);
    viewer.set_view_off_one(&first);
    viewer.set_view_off_one(&second);
    arm_activation_reference(&control, &first, &second);
    second.set_immediate_update(true);
    let argument = HandleFieldRef::new(&control.view_caller, |slot| slot);
    viewer.set_view_on_one(&argument).unwrap();
    assert_eq!(
        activation_argument_row(&first, &second, &argument),
        include_str!("../../tests/fixtures/v3d-activation-argument-native.txt")
    );
}
fn activation_argument_row(
    first: &Handle<dyn V3dViewSource>,
    second: &Handle<dyn V3dViewSource>,
    argument: &dyn HandleReference<dyn V3dViewSource>,
) -> String {
    let first_immediate = first.set_immediate_update(false);
    let second_immediate = second.set_immediate_update(false);
    first.set_immediate_update(first_immediate);
    second.set_immediate_update(second_immediate);
    let same_target = (argument.object_address() == second.object_address()) as u8;
    let first_active = first.view_ref().observe().is_active() as u8;
    let second_active = second.view_ref().observe().is_active() as u8;
    format!(
        "activation-argument {same_target} {first_active} {second_active} {} {} {} {}\n",
        first_immediate as u8,
        second_immediate as u8,
        first.ref_count(),
        second.ref_count()
    )
}

fn camera_alias_viewer() -> (std::sync::Arc<CameraAliasControl>, Handle<V3dViewer>) {
    let control = fresh_camera_alias_control();
    let driver = camera_alias_driver(&NullGraphicDriver::new_handle(), &control);
    let viewer = V3dViewer::new(&driver).unwrap();
    (control, viewer)
}
fn arm_activation_reference(
    control: &CameraAliasControl,
    first: &Handle<dyn V3dViewSource>,
    second: &Handle<dyn V3dViewSource>,
) {
    *control.view_caller.write() = first.clone();
    *control.view_replacement.write() = second.downgrade();
    *control.replace_activation_argument.write() = true;
}

fn initial_camera_alias_row(
    view: &Handle<dyn V3dViewSource>,
    retained: &dyn HandleReference<Graphic3dCamera>,
    owners: [&Handle<Graphic3dCamera>; 2],
) -> String {
    let initial = view.camera_ref();
    selected_camera_alias_row(0, view, [retained, &*initial], owners)
}

#[test]
fn direct_render_camera_field_stays_independent_of_virtual_camera_selection() {
    use rustycad_tkservice::graphic3d::graphic3d_cview_camera_ref;
    let (control, _viewer, view) = camera_borrow_scenario();
    let driver_view = view.view_ref().observe();
    let direct = graphic3d_cview_camera_ref(driver_view.cview_base());
    let original = direct.clone_handle();
    *control.alternate_selected.write() = true;
    assert_ne!(
        view.camera_ref().reference_address(),
        direct.reference_address()
    );
    assert_eq!(direct.object_address(), original.object_address());
    assert_eq!(
        view.camera_ref().object_address(),
        control.alternate.read().object_address()
    );
}

#[test]
fn xr_setup_and_unset_mutate_owned_camera_without_selecting_virtual_alternate_slot() {
    use rustycad_tkservice::graphic3d::{Graphic3dCViewXr, graphic3d_cview_camera_ref};
    let (control, _viewer, view) = camera_borrow_scenario();
    let cview = view.view_ref().observe();
    let base = cview.camera_ref().clone_handle();
    let posed = Handle::new(Graphic3dCamera::default());
    configure_xr_camera_slots(cview.cview_base(), &base, &posed);
    *control.alternate_selected.write() = true;
    let selected = view.camera_ref();
    let direct = graphic3d_cview_camera_ref(cview.cview_base());
    cview.setup_xr_posed_camera();
    assert_eq!(direct.object_address(), posed.object_address());
    assert_selected_alternate(&*selected, &control);
    cview.unset_xr_posed_camera().unwrap();
    assert_eq!(direct.object_address(), base.object_address());
    assert_selected_alternate(&*selected, &control);
}
fn configure_xr_camera_slots(
    cview: &rustycad_tkservice::graphic3d::Graphic3dCViewBase,
    base: &Handle<Graphic3dCamera>,
    posed: &Handle<Graphic3dCamera>,
) {
    let mut fields = cview.fields.write();
    fields.base_xr_camera = base.clone();
    fields.posed_xr_camera = posed.clone();
}

fn assert_selected_alternate(
    selected: &dyn HandleReference<Graphic3dCamera>,
    control: &CameraAliasControl,
) {
    assert_eq!(
        selected.object_address(),
        control.alternate.read().object_address()
    );
}
