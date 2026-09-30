//! Native connectivity, exception persistence and compaction regressions.
use rustycad_tkernel::standard::{Handle, Transient};
use rustycad_tkmesh::brep_mesh::{
    BRepMeshDataStructureOfDelaun as Mesh, BRepMeshEdge, BRepMeshTriangle, BRepMeshVertex,
    DegreeOfFreedom as Freedom,
};
fn mesh() -> Mesh {
    let mesh = Mesh::new(&Handle::new(Default::default()), 1);
    mesh.vertex_tool().set_cell_size(0.5).unwrap();
    for index in 0..5 {
        mesh.add_node(&vertex(index as f64), true).unwrap();
    }
    mesh
}
fn vertex(x: f64) -> BRepMeshVertex {
    BRepMeshVertex::from_uv(x, 0., Freedom::Free)
}
fn edge(first: i32, last: i32) -> BRepMeshEdge {
    BRepMeshEdge::new(first, last, Freedom::Free)
}
fn triangle(edges: [i32; 3]) -> BRepMeshTriangle {
    BRepMeshTriangle::new(edges, [true; 3], Freedom::Free)
}
fn triangulated() -> Mesh {
    let mesh = mesh();
    for (a, b) in [(1, 2), (2, 3), (3, 1), (3, 4), (4, 1)] {
        mesh.add_link(&edge(a, b)).unwrap();
    }
    mesh.add_element(&triangle([1, 2, 3])).unwrap();
    mesh.add_element(&triangle([3, 4, 5])).unwrap();
    mesh
}
#[test]
fn native_node_identity_and_deletion_preserve_slots() {
    let mesh = mesh();
    assert!(mesh.is_instance_name("BRepMesh_DataStructureOfDelaun"));
    assert!(mesh.is_kind_name("Standard_Transient"));
    assert!(!mesh.allocator().is_null());
    assert_eq!(mesh.add_node(&vertex(0.), false).unwrap(), 1);
    assert!(!mesh.substitute_node(1, &vertex(1.)).unwrap());
    assert!(mesh.substitute_node(1, &vertex(8.)).unwrap());
    assert_eq!(mesh.index_of_node(&vertex(8.)).unwrap(), 1);
    mesh.remove_node(1, false).unwrap();
    assert_eq!(mesh.nb_nodes(), 5);
    assert_eq!(mesh.get_node(1).unwrap().movability(), Freedom::Deleted);
    assert_eq!(mesh.add_node(&vertex(9.), true).unwrap(), 1);
}
#[test]
fn reversed_links_self_links_and_slot_reuse_preserve_adjacency() {
    let mesh = mesh();
    assert_eq!(mesh.add_link(&edge(1, 2)).unwrap(), 1);
    assert_eq!(mesh.add_link(&edge(2, 1)).unwrap(), -1);
    assert_eq!(mesh.add_link(&edge(3, 3)).unwrap(), 2);
    assert_eq!(mesh.links_connected_to(3).unwrap().extent(), 2);
    mesh.remove_link(2, true).unwrap();
    assert!(mesh.links_connected_to(3).unwrap().is_empty());
    assert_eq!(mesh.index_of_link(&edge(3, 3)), 0);
    assert_eq!(mesh.add_link(&edge(4, 5)).unwrap(), 2);
    assert_eq!(mesh.links_of_domain().extent(), 2);
    mesh.remove_node(1, true).unwrap();
    assert_eq!(mesh.get_node(1).unwrap().movability(), Freedom::Free);
}
#[test]
fn substitute_link_resets_pairs_and_deleted_branch_does_not_attach() {
    let mesh = triangulated();
    assert!(!mesh.substitute_link(1, &edge(2, 3)).unwrap());
    assert!(mesh.substitute_link(1, &edge(2, 4)).unwrap());
    assert!(mesh.elements_connected_to(1).unwrap().is_empty());
    let index = mesh.add_link(&edge(5, 5)).unwrap();
    mesh.remove_link(index, true).unwrap();
    assert!(mesh.substitute_link(index, &edge(1, 5)).unwrap());
    assert!(mesh.links_connected_to(5).unwrap().is_empty());
    assert!(!mesh.links_of_domain().contains(&index));
}
#[test]
fn add_element_failure_keeps_appended_element_and_prior_pairs() {
    let mesh = triangulated();
    assert!(mesh.add_element(&triangle([1, 3, 5])).is_err());
    assert_eq!(mesh.nb_elements(), 3);
    assert!(mesh.elements_of_domain().contains(&3));
    assert_eq!(mesh.elements_connected_to(1).unwrap().index(2).unwrap(), 3);
    assert_eq!(mesh.elements_connected_to(5).unwrap().extent(), 1);
    assert!(mesh.add_link(&edge(1, 99)).is_err());
    assert_eq!(mesh.nb_links(), 6);
    assert!(mesh.links_connected_to(1).unwrap().contains(&6));
    assert!(!mesh.links_of_domain().contains(&6));
}
#[test]
fn native_substitute_element_writes_next_slot() {
    let mesh = triangulated();
    let replacement = triangle([4, 5, 1]);
    assert!(mesh.substitute_element(1, &replacement).unwrap());
    assert_eq!(mesh.get_element(1).unwrap().edges, [1, 2, 3]);
    assert_eq!(mesh.get_element(2).unwrap().edges, [4, 5, 1]);
    assert_eq!(mesh.elements_connected_to(4).unwrap().index(2).unwrap(), 1);
    assert_eq!(mesh.element_nodes(&replacement).unwrap(), [3, 4, 1]);
}
#[test]
fn deleted_element_replacement_retains_missing_pair_registration() {
    let mesh = triangulated();
    mesh.remove_element(1).unwrap();
    assert!(mesh.substitute_element(1, &triangle([1, 2, 3])).unwrap());
    assert_eq!(mesh.get_element(1).unwrap().movability, Freedom::Deleted);
    assert_eq!(mesh.get_element(2).unwrap().edges, [1, 2, 3]);
    assert!(!mesh.elements_of_domain().contains(&1));
    assert!(mesh.elements_connected_to(1).unwrap().is_empty());
}
#[test]
fn clear_domain_removes_free_elements_and_links_without_shrinking() {
    let mesh = triangulated();
    mesh.clear_domain().unwrap();
    assert_eq!(mesh.nb_elements(), 2);
    assert_eq!(mesh.nb_links(), 5);
    assert!(mesh.elements_of_domain().is_empty());
    assert!(mesh.links_of_domain().is_empty());
    mesh.clear_deleted().unwrap();
    assert_eq!(mesh.nb_links(), 0);
    assert_eq!(mesh.nb_nodes(), 5);
}
#[test]
fn compaction_moves_last_link_and_retains_vertex_extent() {
    let mesh = mesh();
    mesh.add_link(&edge(1, 2)).unwrap();
    mesh.add_link(&edge(3, 4)).unwrap();
    mesh.remove_link(1, true).unwrap();
    mesh.remove_node(1, true).unwrap();
    mesh.clear_deleted().unwrap();
    assert_eq!(mesh.nb_links(), 1);
    assert_eq!(mesh.get_link(1).unwrap().first_node(), 3);
    assert_eq!(mesh.links_connected_to(3).unwrap().first(), Some(&1));
    assert_eq!(mesh.nb_nodes(), 5);
    assert_eq!(mesh.get_node(1).unwrap().coord().x(), 4.);
    assert_eq!(mesh.get_node(5).unwrap().movability(), Freedom::Deleted);
}
#[test]
fn invalid_native_element_slots_return_explicit_rust_bounds_failure() {
    let mesh = triangulated();
    for index in [-1, 0, 3] {
        assert_eq!(
            mesh.get_element(index).unwrap_err().exception_type(),
            "Standard_OutOfRange"
        );
    }
    assert_eq!(
        mesh.substitute_element(2, &triangle([1, 2, 3]))
            .unwrap_err()
            .exception_type(),
        "Standard_OutOfRange"
    );
}
#[allow(dead_code)]
#[path = "../../tkbrep/tests/bintools_behavior/fakes.rs"]
mod dump_filesystem;
#[test]
fn dump_writes_real_brep_and_swallows_failed_open() {
    let mesh = mesh();
    let files = dump_filesystem::MemoryFileSystem::default();
    mesh.dump_on(&files, "vertices.brep");
    assert!(
        files
            .contents("vertices.brep")
            .unwrap()
            .starts_with(b"DBRep_DrawableShape\n")
    );
    mesh.add_link(&edge(1, 2)).unwrap();
    mesh.dump_on(&files, "edges.brep");
    assert_ne!(
        files.contents("vertices.brep"),
        files.contents("edges.brep")
    );
    mesh.dump_on(&dump_filesystem::RefusingFileSystem, "refused.brep");
}
#[test]
fn compact_attached_link_writes_following_element_slot() {
    let mesh = triangulated();
    mesh.add_link(&edge(5, 5)).unwrap();
    mesh.remove_element(1).unwrap();
    mesh.remove_link(1, true).unwrap();
    mesh.add_element(&triangle([2, 4, 6])).unwrap();
    mesh.add_element(&triangle([2, 5, 6])).unwrap();
    mesh.remove_element(4).unwrap();
    mesh.clear_deleted().unwrap();
    assert_eq!(mesh.get_element(3).unwrap().edges, [2, 4, 6]);
    assert_eq!(mesh.get_element(4).unwrap().edges, [2, 4, 1]);
    assert_eq!(mesh.get_element(4).unwrap().movability, Freedom::Free);
    assert!(!mesh.elements_of_domain().contains(&4));
}
