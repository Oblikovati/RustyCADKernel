//! XR mode-zero selection owns actual primitive sensitivities and never substitutes axes/laser picks.
use super::*;
use crate::{select_basics::SelectBasicsPickResult, select_mgr::*, select3d::*};
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::aspect::AspectXRTrackedDeviceRole;
const XR_SELECTION_NATIVE: &str = include_str!("../../tests/fixtures/xr-device-selection.txt");
fn xr_selection_entity_columns(
    entity: &Handle<dyn Select3DSensitiveEntity>,
    device: &Handle<AisXrTrackedDevice>,
) -> Vec<f64> {
    let owner = entity.owner_id();
    let center = entity.center_of_geometry();
    vec![
        f64::from(entity.nb_sub_elements()),
        f64::from(entity.sensitivity_factor()),
        f64::from(owner.priority()),
        f64::from(owner.selectable().object_address() == device.object_address()),
        f64::from(entity.to_build_bvh()),
        f64::from(entity.has_init_location()),
        center.x(),
        center.y(),
        center.z(),
    ]
}
fn xr_selection_pick_columns(entity: &Handle<dyn Select3DSensitiveEntity>, step: i32) -> Vec<f64> {
    let volume = crate::select3d::matching_volume(step % 4, step % 2 != 0);
    let mut pick = SelectBasicsPickResult::default();
    let outcome = entity.matches(&volume, &mut pick);
    let mut columns = vec![
        f64::from(outcome.is_err()),
        f64::from(outcome.unwrap_or(false)),
        f64::from(pick.is_valid()),
    ];
    if pick.is_valid() {
        columns.extend([pick.depth(), pick.dist_to_geom_center()]);
    }
    let primitive = entity.clone().cast::<Select3DSensitivePrimitiveArray>();
    append_xr_primitive_detection(&mut columns, &primitive, entity.to_build_bvh());
    columns
}
fn append_xr_primitive_detection(
    columns: &mut Vec<f64>,
    primitive: &Handle<Select3DSensitivePrimitiveArray>,
    dirty: bool,
) {
    columns.extend([
        f64::from(primitive.last_detected_element()),
        f64::from(primitive.last_detected_node()),
        f64::from(primitive.last_detected_edge_node1()),
        f64::from(primitive.last_detected_edge_node2()),
        f64::from(dirty),
    ]);
}
fn xr_selection_row(
    selection: &Handle<SelectMgrSelection>,
    device: &Handle<AisXrTrackedDevice>,
    kind: i32,
    step: i32,
) -> Vec<f64> {
    let entities = selection.entities();
    let mut row = vec![
        f64::from(kind),
        f64::from(step),
        entities.len() as f64,
        f64::from(selection.sensitivity()),
    ];
    for stored in entities {
        let entity = stored.base_sensitive();
        row.extend(xr_selection_entity_columns(&entity, device));
        row.extend(xr_selection_pick_columns(&entity, step));
    }
    row
}
fn replay_xr_selection(kind: i32, expected: &mut std::str::Lines<'_>) {
    let mut mesh = super::xr_state_fixture::xr_state_model(kind);
    let device = xr_selection_device(kind, &mesh);
    let selection = Handle::new(SelectMgrSelection::new(0));
    device.compute_selection(&selection, 1).unwrap();
    for step in 0..10 {
        apply_xr_selection_step(&device, &selection, &mut mesh, kind, step);
        crate::select3d::compare_oracle_snapshot(
            &xr_selection_row(&selection, &device, kind, step),
            expected.next().unwrap(),
        );
    }
}
fn xr_selection_device(
    kind: i32,
    mesh: &Handle<rustycad_tkservice::graphic3d::Graphic3dArrayOfTriangles>,
) -> Handle<AisXrTrackedDevice> {
    if kind == 0 {
        return AisXrTrackedDevice::new().unwrap();
    }
    AisXrTrackedDevice::with_render_model(mesh, &Handle::default()).unwrap()
}
fn apply_xr_selection_step(
    device: &Handle<AisXrTrackedDevice>,
    selection: &Handle<SelectMgrSelection>,
    mesh: &mut Handle<rustycad_tkservice::graphic3d::Graphic3dArrayOfTriangles>,
    kind: i32,
    step: i32,
) {
    if step == 1 || step == 6 || step == 8 {
        device.compute_selection(selection, 0).unwrap();
    }
    if step == 2 {
        set_xr_selection_policies(device);
    }
    if step == 7 && (kind == 2 || kind == 3) {
        xr_move_selection_model(mesh);
    }
    if step == 9 {
        mesh.nullify();
    }
}
fn set_xr_selection_policies(device: &Handle<AisXrTrackedDevice>) {
    device.set_unit_factor(1000.);
    device.set_role(AspectXRTrackedDeviceRole::RightHand);
    device.set_laser_length(9.).unwrap();
}
#[test]
fn native_xr_selection_mode_append_owner_and_actual_volume_detection() {
    let mut expected = XR_SELECTION_NATIVE.lines();
    for kind in 0..6 {
        replay_xr_selection(kind, &mut expected);
    }
    assert!(expected.next().is_none());
}

fn xr_move_selection_model(
    mesh: &Handle<rustycad_tkservice::graphic3d::Graphic3dArrayOfTriangles>,
) {
    for index in 1..=mesh.vertex_number() {
        let point = mesh.vertice(index).unwrap();
        mesh.set_vertice_pnt(
            index,
            &rustycad_tkmath::gp::Point3::new(point.x() + 50., point.y(), point.z()),
        )
        .unwrap();
    }
}
