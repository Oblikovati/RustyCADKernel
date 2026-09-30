//! Accumulated native neighborhoods, no-op overloads and initialization semantics.
use rustycad_tkernel::standard::{Handle, Transient};
use rustycad_tkmesh::brep_mesh::BRepMeshSelectorOfDataStructureOfDelaun as Selector;
#[path = "support/delaun_fixture.rs"]
mod fixture;
use fixture::*;
fn selected(selector: &Selector) -> Vec<i32> {
    let mut indices: Vec<i32> = selector.elements().iter().copied().collect();
    indices.sort_unstable();
    indices
}
fn empty_auxiliary_maps(selector: &Selector) {
    assert!(selector.nodes().is_empty());
    assert!(selector.links().is_empty());
    assert!(selector.frontier_links().is_empty());
}
#[test]
fn vertex_and_edge_queries_select_all_incident_elements() {
    let selector = Selector::new(&selection_mesh());
    assert!(selector.is_instance_name("BRepMesh_SelectorOfDataStructureOfDelaun"));
    assert!(selector.is_kind_name("Standard_Transient"));
    selector.neighbours_of_vertex(&vertex(1.)).unwrap();
    assert_eq!(selected(&selector), [1]);
    selector.neighbours_of_edge(&edge(2, 3)).unwrap();
    assert_eq!(selected(&selector), [1, 2, 3]);
    empty_auxiliary_maps(&selector);
}
#[test]
fn edge_neighbors_are_narrower_than_triangle_node_neighbors() {
    let mesh = selection_mesh();
    let selector = Selector::new(&mesh);
    selector
        .neighbours_by_edge_of(&triangle([1, 2, 3]))
        .unwrap();
    assert_eq!(selected(&selector), [1, 2]);
    selector
        .neighbours_of_triangle(&triangle([1, 2, 3]))
        .unwrap();
    assert_eq!(selected(&selector), [1, 2, 3]);
    selector.initialize(&mesh);
    assert!(selected(&selector).is_empty());
    selector.neighbours_of_element(1).unwrap();
    assert_eq!(selected(&selector), [1, 2, 3]);
}
#[test]
fn checked_failure_retains_preceding_selection_and_initialization_clears() {
    let mesh = selection_mesh();
    let selector = Selector::new(&mesh);
    assert!(selector.neighbours_of_edge(&edge(2, 99)).is_err());
    assert_eq!(selected(&selector), [1]);
    selector.neighbours_of_link(7).unwrap();
    assert_eq!(selected(&selector), [1, 3]);
    selector.initialize(&mesh);
    mesh.remove_element(1).unwrap();
    selector.neighbours_of_node(2).unwrap();
    assert!(selected(&selector).is_empty());
    selector.neighbours_of_link(3).unwrap();
    assert_eq!(selected(&selector), [2, 3]);
}
#[test]
fn default_null_mesh_noops_remain_noops_but_queries_report_divergence() {
    let selector = Selector::default();
    selector.add_neighbours();
    selector.neighbours_of_selector(&selector);
    empty_auxiliary_maps(&selector);
    assert!(selected(&selector).is_empty());
    assert_eq!(
        selector.neighbours_of_node(1).unwrap_err().exception_type(),
        "Standard_NullObject"
    );
    selector.initialize(&selection_mesh());
    selector.neighbours_of_element(1).unwrap();
    selector.initialize(&Handle::default());
    assert!(selected(&selector).is_empty());
}
