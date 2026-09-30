use super::super::{flag, int};
use super::vertex_inspector::report_unit;
use rustycad_tkernel::{
    ncollection::Map,
    standard::{Failure, Handle},
};
use rustycad_tkmesh::brep_mesh::BRepMeshSelectorOfDataStructureOfDelaun as Selector;
#[path = "../../tests/support/delaun_fixture.rs"]
mod fixture;
use fixture::{edge, triangle, vertex};
pub fn selector_case(v: &[i32]) {
    let mesh = fixture::selection_mesh();
    let selector = Handle::new(Selector::new(&mesh));
    flag(selector.is_instance_name("BRepMesh_SelectorOfDataStructureOfDelaun"));
    flag(selector.is_kind_name("Standard_Transient"));
    prepare(&selector, &mesh, v[2]);
    report_unit(select(&selector, v[0], v[1]));
    snapshot(&selector);
    selector.neighbours_of_selector(&Selector::default());
    selector.add_neighbours();
    snapshot(&selector);
    selector.initialize(&Handle::default());
    snapshot(&selector);
    println!();
}
fn prepare(
    selector: &Selector,
    mesh: &Handle<rustycad_tkmesh::brep_mesh::BRepMeshDataStructureOfDelaun>,
    mode: i32,
) {
    if mode & 1 != 0 {
        selector.neighbours_of_node(1).unwrap();
    }
    if mode & 2 != 0 {
        mesh.remove_element(1).unwrap();
    }
    if mode & 4 != 0 {
        selector.initialize(mesh);
    }
}
fn select(selector: &Selector, op: i32, index: i32) -> Result<(), Failure> {
    match op {
        0 => selector.neighbours_of_vertex(&vertex(f64::from(index - 1))),
        1 => selector.neighbours_of_node(index),
        2 => selector.neighbours_of_edge(&edge(2, index)),
        3 => selector.neighbours_of_link(index),
        4 => selector.neighbours_of_triangle(&triangle([1, 2, index])),
        5 => selector.neighbours_of_element(index),
        6 => selector.neighbours_by_edge_of(&triangle([1, 2, index])),
        _ => {
            selector.add_neighbours();
            selector.neighbours_of_selector(selector);
            Ok(())
        }
    }
}
fn report_map(indices: Map<i32>) {
    int(indices.extent() as i32);
    for index in indices.iter() {
        int(*index);
    }
}
fn snapshot(selector: &Selector) {
    report_map(selector.nodes());
    report_map(selector.links());
    report_map(selector.elements());
    report_map(selector.frontier_links());
}
pub fn invalid_case(v: &[i32]) {
    let selector = Selector::default();
    report_unit(select(&selector, v[0], 1));
    println!();
}
