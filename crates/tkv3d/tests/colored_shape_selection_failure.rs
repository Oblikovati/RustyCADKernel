//! Tiny positive deflection triggers genuine wire construction failures; repair preserves native inherited catch policy.
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Vertex, Wire},
};
use rustycad_tkernel::{
    message::Messenger,
    standard::{Failure, Handle},
};
use rustycad_tkmath::gp::{Axis2, Ellipse3, Point3};
use rustycad_tkservice::aspect::AspectTypeOfDeflection;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
use rustycad_tkv3d::{
    ais::*, prs_mgr::PrsMgrPresentableObjectMembers, select_mgr::*, select3d::*, std_select::*,
};
use std::fmt::Write;
#[test]
fn native_selection_fault_repair_preserves_partial_append_and_parent_fallback() {
    let mut rows = String::new();
    for route in 0..3 {
        for mode in [0, 3] {
            replay_selection_fault(route, mode, &mut rows);
        }
    }
    assert_eq!(
        rows,
        include_str!("fixtures/colored-shape-compute/native-selection-failure.txt")
    );
}
fn replay_selection_fault(route: usize, mode: i32, rows: &mut String) {
    let wire = selection_fault_wire();
    let object = AisColoredShape::new(&wire);
    configure_selection_fault(&object, route);
    let selection = Handle::new(SelectMgrSelection::new(mode));
    let owner = SelectMgrEntityOwner::new(17).into_cast();
    selection
        .add(&Handle::new(Select3DSensitivePoint::new(&owner, Point3::new(9., 8., 7.))).cast())
        .unwrap();
    let failed = invoke_selection_fault(route, &object, &wire, &selection, mode, 1e-9);
    if let Err(failure) = &failed {
        assert_eq!(failure.exception_type(), "Standard_ConstructionError");
    }
    record_selection_fault(
        "fault",
        route,
        mode,
        failed.is_err(),
        &selection,
        &object,
        rows,
    );
    invoke_selection_fault(route, &object, &wire, &selection, mode, 0.1).unwrap();
    record_selection_fault("repaired", route, mode, false, &selection, &object, rows);
}
fn selection_fault_wire() -> Wire {
    let ellipse = Ellipse3::new(Axis2::default(), 3., 1.).unwrap();
    let edge = BRepBuilderAPIMakeEdge::from_elips_params(ellipse, 0.2, 5.4)
        .unwrap()
        .edge()
        .unwrap();
    let builder = BRepBuilder::default();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    builder.add(&mut wire, &edge).unwrap();
    wire
}
fn configure_selection_fault(object: &AisColoredShape, route: usize) {
    object.attributes().set_auto_triangulation(false);
    object
        .attributes()
        .set_type_of_deflection(AspectTypeOfDeflection::Absolute);
    if route != 2 {
        return;
    }
    let mut vertex = Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut vertex, Point3::new(100., 200., 300.), 1e-7)
        .unwrap();
    object.custom_aspects(&vertex).unwrap().set_hidden(true);
}
fn invoke_selection_fault(
    route: usize,
    object: &AisColoredShape,
    wire: &Wire,
    selection: &Handle<SelectMgrSelection>,
    mode: i32,
    deflection: f64,
) -> Result<(), Failure> {
    if route == 0 {
        let owner = StdSelectBRepOwner::with_shape(wire, 7, false).into_cast();
        return StdSelectBrepSelectionTool::compute_sensitive(
            wire, &owner, selection, deflection, 0.5, 9, 500., false,
        );
    }
    object
        .attributes()
        .set_maximal_chordial_deviation(deflection);
    if route == 2 {
        return object.compute_selection(selection, mode);
    }
    let messenger = Messenger::new();
    messenger.printers_mut().clear();
    ais_shape_compute_selection_with_messenger(object, selection, mode, &messenger)
}
#[allow(clippy::too_many_arguments)] // Native row includes both route and retained failure/repair phase.
fn record_selection_fault(
    label: &str,
    route: usize,
    mode: i32,
    failed: bool,
    selection: &SelectMgrSelection,
    object: &AisColoredShape,
    rows: &mut String,
) {
    let entities = selection.entities();
    write!(
        rows,
        "{label} {route} {mode} {} {}",
        i32::from(failed),
        entities.len()
    )
    .unwrap();
    for wrapper in entities {
        let entity = wrapper.base_sensitive();
        let owner = entity.owner_id();
        let assigned =
            owner.selectable().object_address() == (object as *const AisColoredShape).cast();
        write!(
            rows,
            " {} {} {} {} {}",
            entity.dynamic_type().unwrap().name(),
            entity.nb_sub_elements(),
            owner.priority(),
            i32::from(owner.comes_from_decomposition()),
            i32::from(assigned)
        )
        .unwrap();
    }
    rows.push('\n');
}
