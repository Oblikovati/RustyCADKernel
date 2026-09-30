//! Native tolerance branches, cell lifecycle, shared storage and deletion-queue reuse.
use rustycad_tkernel::{
    ncollection::{CellAction, DynamicArray, Shared},
    standard::{Handle, SharedState, Transient},
};
use rustycad_tkmath::gp::Xy;
use rustycad_tkmesh::brep_mesh::{
    BRepMeshVertex, BRepMeshVertexInspector, BRepMeshVertexTool, DegreeOfFreedom as Freedom,
};
fn vertex(x: f64, y: f64) -> BRepMeshVertex {
    BRepMeshVertex::from_uv(x, y, Freedom::Free)
}
fn inspector() -> BRepMeshVertexInspector {
    BRepMeshVertexInspector::new(&Handle::default())
}
fn tool() -> BRepMeshVertexTool {
    let tool = BRepMeshVertexTool::new(&Handle::default());
    tool.set_cell_size(0.1).unwrap();
    tool
}
#[test]
fn static_geometry_matches_native_zero_based_coordinates() {
    assert_eq!(BRepMeshVertexInspector::DIMENSION, 2);
    let point = Xy::new(2., 3.);
    assert_eq!(BRepMeshVertexInspector::coord(0, &point).unwrap(), 2.);
    assert_eq!(BRepMeshVertexInspector::coord(1, &point).unwrap(), 3.);
    assert!(BRepMeshVertexInspector::coord(2, &point).is_err());
    assert_eq!(BRepMeshVertexInspector::shift(&point, -1.), Xy::new(1., 2.));
    assert!(BRepMeshVertexInspector::is_equal(2, 2));
    assert!(!BRepMeshVertexInspector::is_equal(1, 2));
}
#[test]
fn inspector_strict_distance_keeps_first_tie_and_clear_keeps_query_result() {
    let mut inspector = inspector();
    inspector.set_tolerance(0.1);
    inspector.add(&vertex(-0.05, 0.)).unwrap();
    inspector.add(&vertex(0.05, 0.)).unwrap();
    inspector.set_point(Xy::default());
    inspector.inspect(2).unwrap();
    inspector.inspect(1).unwrap();
    assert_eq!(inspector.get_coincident_point(), 2);
    inspector.set_tolerance(0.05);
    inspector.set_point(Xy::default());
    inspector.inspect(1).unwrap();
    assert_eq!(inspector.get_coincident_point(), 0);
    inspector.set_tolerance(0.1);
    inspector.inspect(1).unwrap();
    inspector.clear();
    assert_eq!(inspector.get_coincident_point(), 1);
    assert_eq!(inspector.nb_vertices(), 0);
}
#[test]
fn squared_second_tolerance_selects_native_radial_or_rectangular_branch() {
    let mut inspector = inspector();
    inspector.add(&vertex(0., 0.05)).unwrap();
    inspector.set_tolerances(0.1, 0.0001);
    inspector.set_point(Xy::default());
    inspector.inspect(1).unwrap();
    assert_eq!(inspector.get_coincident_point(), 1);
    inspector.set_tolerances(0.1, 0.001);
    inspector.set_point(Xy::default());
    inspector.inspect(1).unwrap();
    assert_eq!(inspector.get_coincident_point(), 0);
}
#[test]
fn inspector_repeated_delete_and_purge_append_duplicate_reusable_slots() {
    let mut inspector = inspector();
    inspector.add(&vertex(0., 0.)).unwrap();
    inspector.delete(1).unwrap();
    inspector.delete(1).unwrap();
    assert_eq!(inspector.inspect(1).unwrap(), CellAction::Purge);
    let deleted = inspector.get_list_of_del_points();
    assert_eq!(
        deleted.read().iter().copied().collect::<Vec<_>>(),
        [1, 1, 1]
    );
    for x in [1., 2., 3.] {
        assert_eq!(inspector.add(&vertex(x, 0.)).unwrap(), 1);
    }
    assert_eq!(inspector.nb_vertices(), 1);
    assert_eq!(inspector.get_vertex(1).unwrap().coord().x(), 3.);
    assert!(deleted.read().is_empty());
}
#[test]
fn inspector_vertex_slot_is_live_and_replacement_preserves_old_owner() {
    let mut inspector = inspector();
    inspector.add(&vertex(1., 0.)).unwrap();
    let old = inspector.vertices();
    let slot = inspector.change_vertices();
    *slot.write() = Handle::new(Shared::new(SharedState::new(DynamicArray::default())));
    assert_eq!(inspector.nb_vertices(), 0);
    assert_eq!(old.read().length(), 1);
    inspector.add(&vertex(2., 0.)).unwrap();
    inspector.set_vertex(1, vertex(3., 0.)).unwrap();
    assert_eq!(inspector.get_vertex(1).unwrap().coord().x(), 3.);
    assert_eq!(old.read().value(0).unwrap().coord().x(), 1.);
}
#[test]
fn invalid_index_rust_bounds_failures_are_explicit_native_divergences() {
    let mut inspector = inspector();
    inspector.add(&vertex(0., 0.)).unwrap();
    for index in [-1, 0, 2] {
        assert_eq!(
            inspector.get_vertex(index).unwrap_err().exception_type(),
            "Standard_OutOfRange"
        );
        assert_eq!(
            inspector.delete(index).unwrap_err().exception_type(),
            "Standard_OutOfRange"
        );
        assert_eq!(
            inspector.inspect(index).unwrap_err().exception_type(),
            "Standard_OutOfRange"
        );
    }
}
#[test]
fn native_tool_type_default_tolerances_uniqueness_and_forced_storage() {
    let tool = tool();
    assert!(tool.is_instance_name("BRepMesh_VertexTool"));
    assert!(tool.is_kind_name("Standard_Transient"));
    assert!(tool.is_empty());
    assert_eq!(tool.get_tolerance(), (1e-7, 1e-7));
    assert_eq!(tool.add(&vertex(0., 0.), false).unwrap(), 1);
    assert_eq!(tool.add(&vertex(0., 0.), false).unwrap(), 1);
    assert_eq!(tool.add(&vertex(0., 0.), true).unwrap(), 2);
    assert_eq!(tool.extent(), 2);
    assert_eq!(tool.find_index(&vertex(0., 0.)).unwrap(), 2);
    assert_eq!(tool.find_key(1).unwrap(), vertex(0., 0.));
}
#[test]
fn deletion_substitution_and_remove_last_do_not_shrink_extent() {
    let tool = tool();
    tool.add(&vertex(0., 0.), true).unwrap();
    tool.add(&vertex(1., 0.), true).unwrap();
    tool.delete_vertex(1).unwrap();
    assert_eq!(tool.extent(), 2);
    assert_eq!(tool.add(&vertex(2., 0.), true).unwrap(), 1);
    tool.substitute(2, &vertex(3., 0.)).unwrap();
    assert_eq!(tool.find_index(&vertex(1., 0.)).unwrap(), 0);
    assert_eq!(tool.find_index(&vertex(3., 0.)).unwrap(), 2);
    tool.remove_last().unwrap();
    assert_eq!(tool.extent(), 2);
    assert_eq!(tool.get_list_of_del_nodes().read().first(), Some(&2));
}
#[test]
fn anisotropic_reset_then_scalar_reset_preserves_native_bounds_failure() {
    let tool = tool();
    tool.set_cell_sizes(0.1, 0.2).unwrap();
    tool.set_tolerances(0.01, 0.02);
    tool.add(&vertex(0., 0.), true).unwrap();
    assert_eq!(
        tool.set_cell_size(0.3).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    assert_eq!(tool.extent(), 1);
    assert_eq!(tool.get_tolerance(), (0.01, 0.02));
    let old = tool.vertices();
    tool.set_cell_sizes(0.2, 0.3).unwrap();
    assert_eq!(old.read().length(), 0);
    assert!(tool.is_empty());
}
#[test]
fn shared_replacement_does_not_rebuild_existing_spatial_cells() {
    let tool = tool();
    tool.add(&vertex(0., 0.), true).unwrap();
    let old = tool.vertices();
    let mut replacement = DynamicArray::default();
    replacement.append(vertex(3., 0.));
    *tool.change_vertices().write() = Handle::new(Shared::new(SharedState::new(replacement)));
    assert_eq!(tool.find_key(1).unwrap(), vertex(3., 0.));
    assert_eq!(tool.find_index(&vertex(3., 0.)).unwrap(), 0);
    assert_eq!(old.read().value(0).unwrap().coord().x(), 0.);
}
#[test]
fn negative_cell_sizes_preserve_reversed_empty_insertion_ranges() {
    let tool = tool();
    tool.set_cell_sizes(-0.1, 0.2).unwrap();
    tool.set_tolerance(0.05);
    assert_eq!(tool.add(&vertex(1., 0.), false).unwrap(), 1);
    assert_eq!(tool.find_index(&vertex(1., 0.)).unwrap(), 0);
    assert_eq!(tool.add(&vertex(1., 0.), false).unwrap(), 2);
}
