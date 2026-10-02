//! Populated existing-mesh operations that precede Delaunay insertion.
use rustycad_tkernel::{
    ncollection::{DynamicArray, Sequence},
    standard::Handle,
};
use rustycad_tkmath::gp::{Point2, Xy};
use rustycad_tkmesh::brep_mesh::{
    BRepMeshDataStructureOfDelaun as Mesh, BRepMeshDelaun as Delaun, BRepMeshVertex as Vertex,
    DegreeOfFreedom as Freedom,
};
#[path = "support/delaun_core.rs"]
mod fixture;
#[test]
fn containment_marks_free_boundaries_and_rejects_constrained_boundaries() {
    let mesh = fixture::mesh(0, 0);
    let delaun = Delaun::from_mesh(&mesh, -1, -1, false).unwrap();
    let vertex = Vertex::from_uv(1., 0., Freedom::Free);
    let mut edge = 99;
    assert!(delaun.contains(1, &vertex, 1e-18, &mut edge).unwrap());
    assert_eq!(edge, 1);
    let constrained = Delaun::from_mesh(&fixture::mesh(0, 1), -1, -1, false).unwrap();
    assert!(!constrained.contains(1, &vertex, 1e-18, &mut edge).unwrap());
    assert_eq!(edge, 0);
    assert!(
        !delaun
            .contains(1, &Vertex::from_uv(-1., 1., Freedom::Free), 0., &mut edge)
            .unwrap()
    );
}
#[test]
fn winding_requires_explicit_closure_and_accepts_both_orientations() {
    for kind in [0, 1] {
        let mesh = fixture::mesh(kind, 0);
        let index = mesh
            .add_node(&Vertex::from_uv(1., 1., Freedom::Free), true)
            .unwrap();
        let delaun = Delaun::from_mesh(&mesh, -1, -1, false).unwrap();
        let mut polygon = DynamicArray::with_increment(5);
        for i in 1..=4 {
            polygon.append(i);
        }
        assert!(!delaun.polygon_contains_vertex(index, &polygon).unwrap());
        polygon.append(1);
        assert!(delaun.polygon_contains_vertex(index, &polygon).unwrap());
        assert!(delaun.polygon_contains_vertex(1, &polygon).unwrap());
    }
}
#[test]
fn area_and_intersection_follow_oriented_mesh_edges() {
    let delaun = Delaun::from_mesh(&fixture::mesh(0, 0), -1, -1, false).unwrap();
    let mut polygon = Sequence::default();
    for i in 1..=4 {
        polygon.append(i);
    }
    assert_eq!(delaun.polygon_area(&polygon, 1, 4).unwrap(), 4.);
    assert_eq!(delaun.polygon_area(&polygon, 4, 1).unwrap(), 0.);
    let mut point = Point2::new(7., 8.);
    delaun
        .edge_intersection(
            &delaun.get_edge(1).unwrap(),
            &delaun.get_edge(2).unwrap(),
            true,
            true,
            &mut point,
        )
        .unwrap();
    // Native endpoint contact leaves the output point untouched.
    assert_eq!(point, Point2::new(7., 8.));
}
#[test]
fn auxiliary_copy_removes_real_triangles_links_and_nodes() {
    let mesh = fixture::mesh(0, 0);
    let mut delaun = Delaun::from_mesh(&mesh, -1, -1, true).unwrap();
    assert!(!delaun.circles().is_empty());
    let mut auxiliary = DynamicArray::with_increment(3);
    auxiliary.append(1);
    delaun.set_aux_vertices(&auxiliary);
    auxiliary.clear(false);
    delaun.remove_aux_elements().unwrap();
    assert!(mesh.elements_of_domain().is_empty());
    assert!(mesh.links_of_domain().is_empty());
    assert_eq!(delaun.get_vertex(1).unwrap().movability(), Freedom::Deleted);
    assert!(!delaun.circles().is_empty());
    assert!(delaun.result() == &mesh);
}
#[test]
fn native_edge_classification_counts_frontiers_and_single_use_edges() {
    let delaun = Delaun::from_mesh(&fixture::mesh(0, 31), 2, 4, false).unwrap();
    assert_eq!(delaun.frontier().unwrap().extent(), 3);
    assert_eq!(delaun.internal_edges().unwrap().extent(), 2);
    assert_eq!(delaun.free_edges().unwrap().extent(), 4);
    assert!(!delaun.use_edge(1));
    assert!(!delaun.use_edge(-1));
}
#[test]
fn frontier_and_free_polygon_loops_recover_the_same_two_triangles() {
    for kind in 0..=1 {
        for frontier in [false, true] {
            let mesh = fixture::polygon_mesh(kind, frontier);
            let mut delaun = Delaun::from_mesh(&mesh, -1, -1, false).unwrap();
            let mut links = Sequence::default();
            for index in 1..=4 {
                links.append(index);
            }
            delaun.mesh_polygon_links(&mut links).unwrap();
            assert_eq!(mesh.elements_of_domain().extent(), 2);
            assert_eq!(mesh.links_of_domain().extent(), 5);
            assert!(links.is_empty());
        }
    }
}
#[test]
fn supermesh_adds_the_native_three_vertices_and_oriented_edges() {
    let allocator = Handle::new(Default::default());
    let mesh = Handle::new(Mesh::new(&allocator, 3));
    let mut delaun = Delaun::from_mesh(&mesh, -1, -1, false).unwrap();
    let (nodes, edges) = delaun.build_super_mesh([0., 0., 2., 2.]).unwrap();
    assert_eq!(nodes, [1, 2, 3]);
    assert_eq!(edges, [1, 2, 3]);
    assert_eq!(delaun.get_vertex(nodes[0]).unwrap().coord().x(), 1.);
    assert_eq!(delaun.get_vertex(nodes[0]).unwrap().coord().y(), 4.);
    assert_eq!(delaun.get_edge(edges[0]).unwrap().first_node(), nodes[0]);
    assert_eq!(delaun.get_edge(edges[2]).unwrap().last_node(), nodes[0]);
}
#[test]
fn degenerate_triangles_and_null_mesh_are_explicit() {
    let mut edge = 99;
    let delaun = Delaun::from_mesh(&fixture::mesh(2, 0), 1, 1, false).unwrap();
    assert!(
        !delaun
            .contains(
                1,
                &Vertex::new(Xy::default(), 0, Freedom::Free),
                0.,
                &mut edge
            )
            .unwrap()
    );
    assert_eq!(edge, 0);
    let null = Delaun::from_mesh(&Handle::default(), 1, 1, false).unwrap();
    assert!(null.result().is_null());
    assert!(null.circles().is_empty());
    assert_eq!(
        null.get_vertex(1).unwrap_err().exception_type(),
        "Standard_NullObject"
    );
}

#[test]
fn collinear_circle_cleanup_retains_explicit_native_crash_gap() {
    let mesh = fixture::mesh(3, 0);
    let mut delaun = Delaun::from_mesh(&mesh, -1, 1, true).unwrap();
    assert!(delaun.circles().is_empty());
    let mut auxiliary = DynamicArray::with_increment(3);
    auxiliary.append(1);
    delaun.set_aux_vertices(&auxiliary);
    assert_eq!(
        delaun.remove_aux_elements().unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    assert_eq!(mesh.elements_of_domain().extent(), 2);
}
