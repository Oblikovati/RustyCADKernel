//! Populated existing-mesh operations that precede Delaunay insertion.
use rustycad_tkernel::{
    ncollection::{DataMap, DynamicArray, Sequence},
    standard::Handle,
};
use rustycad_tkmath::gp::{Point2, Vector2, Xy};
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
fn first_supermesh_vertex_creates_three_native_fan_triangles() {
    let allocator = Handle::new(Default::default());
    let mesh = Handle::new(Mesh::new(&allocator, 4));
    let mut delaun = Delaun::from_mesh(&mesh, -1, -1, false).unwrap();
    delaun
        .build_super_fan([0., 0., 2., 2.], Xy::new(1., 1.))
        .unwrap();
    assert_eq!(mesh.nb_nodes(), 4);
    assert_eq!(mesh.nb_links(), 6);
    assert_eq!(mesh.elements_of_domain().extent(), 3);
}
#[test]
fn use_edge_preserves_native_always_false_behavior_for_signed_indices() {
    let mesh = fixture::mesh(0, 0);
    let delaun = Delaun::from_mesh(&mesh, -1, -1, false).unwrap();
    for index in [i32::MIN, -1, 0, 1, i32::MAX] {
        assert!(!delaun.use_edge(index));
    }
}
#[test]
fn frontier_reachability_stops_at_constraints_and_open_boundaries() {
    let free = Delaun::from_mesh(&fixture::mesh(0, 0), -1, -1, false).unwrap();
    assert!(!free.is_bound_to_frontier(1, 1).unwrap());
    let constrained = Delaun::from_mesh(&fixture::mesh(0, 31), -1, -1, false).unwrap();
    assert!(constrained.is_bound_to_frontier(1, 2).unwrap());
}
#[test]
fn cleanup_mesh_removes_unbounded_free_triangles_but_keeps_constrained_region() {
    let free_mesh = fixture::mesh(0, 0);
    Delaun::from_mesh(&free_mesh, -1, -1, false)
        .unwrap()
        .cleanup_mesh()
        .unwrap();
    assert_eq!(free_mesh.elements_of_domain().extent(), 0);
    assert_eq!(free_mesh.links_of_domain().extent(), 0);

    let constrained_mesh = fixture::mesh(0, 31);
    Delaun::from_mesh(&constrained_mesh, -1, -1, false)
        .unwrap()
        .cleanup_mesh()
        .unwrap();
    assert_eq!(constrained_mesh.elements_of_domain().extent(), 2);
    assert_eq!(constrained_mesh.links_of_domain().extent(), 5);
}
#[test]
fn insertion_order_matches_native_heap_ties_in_projected_coordinates() {
    let mesh = Handle::new(Mesh::new(&Handle::new(Default::default()), 4));
    for point in [
        Xy::new(3., 0.),
        Xy::new(0., 3.),
        Xy::new(2., 1.),
        Xy::new(1., 2.),
    ] {
        mesh.add_node(&Vertex::new(point, 0, Freedom::Free), true)
            .unwrap();
    }
    let delaun = Delaun::from_mesh(&mesh, -1, -1, false).unwrap();
    let mut indices = vec![1, 2, 3, 4];
    delaun.sort_vertex_indices(&mut indices).unwrap();
    assert_eq!(indices, [2, 4, 1, 3]);
}

#[test]
fn next_polygon_link_selects_visible_neighbor_and_preserves_empty_outputs() {
    let mesh = fixture::mesh(0, 0);
    let delaun = Delaun::from_mesh(&mesh, -1, -1, false).unwrap();
    let polygon = Sequence::default();
    let boxes = Sequence::default();
    let pivot = delaun.get_vertex(1).unwrap();
    let mut leprous = rustycad_tkernel::ncollection::Map::default();
    let mut dead = rustycad_tkernel::ncollection::Map::default();
    let mut next_pivot = -77;
    let mut next_direction = Vector2::new(81., 82.);
    let mut bounds = rustycad_tkmath::bnd::CompactBox2::default();

    let next = delaun
        .find_next_polygon_link(
            4,
            1,
            &pivot,
            Vector2::new(1., 0.),
            &boxes,
            &polygon,
            None,
            false,
            &mut leprous,
            &mut dead,
            &mut next_pivot,
            &mut next_direction,
            &mut bounds,
        )
        .unwrap();

    assert_eq!(next, -4);
    assert_eq!(next_pivot, 4);
    assert_eq!(next_direction, Vector2::new(0., 2.));
    assert!(leprous.contains(&1));
    assert!(leprous.contains(&4));
    assert!(leprous.contains(&5));
    assert_eq!(
        bounds.corner_min().x(),
        -rustycad_tkernel::precision::p_confusion()
    );
    assert_eq!(
        bounds.corner_max().x(),
        rustycad_tkernel::precision::p_confusion()
    );

    let mut blocked = rustycad_tkernel::ncollection::Map::default();
    for link in [1, 4, 5] {
        blocked.add(link);
    }
    let mut unchanged_pivot = -77;
    let mut unchanged_direction = Vector2::new(81., 82.);
    let mut unchanged_bounds = rustycad_tkmath::bnd::CompactBox2::default();
    assert_eq!(
        delaun
            .find_next_polygon_link(
                4,
                1,
                &pivot,
                Vector2::new(1., 0.),
                &boxes,
                &polygon,
                None,
                false,
                &mut leprous,
                &mut blocked,
                &mut unchanged_pivot,
                &mut unchanged_direction,
                &mut unchanged_bounds,
            )
            .unwrap(),
        0
    );
    assert_eq!(unchanged_pivot, -77);
    assert_eq!(unchanged_direction, Vector2::new(81., 82.));
    assert!(unchanged_bounds.is_void());
}

#[test]
fn polygon_loop_recovery_cleans_and_retriangulates_the_frontier_square() {
    let mesh = fixture::polygon_mesh(0, true);
    let mut delaun = Delaun::from_mesh(&mesh, -1, -1, false).unwrap();

    assert!(delaun.mesh_left_polygon(1, true, None).unwrap());
    assert_eq!(mesh.elements_of_domain().extent(), 2);
}

#[test]
fn polygon_loop_recovery_removes_interior_victim_triangles() {
    let mesh = fixture::polygon_mesh_with_center();
    let mut delaun = Delaun::from_mesh(&mesh, -1, -1, false).unwrap();

    assert!(delaun.mesh_left_polygon(1, true, None).unwrap());
    assert_eq!(delaun.result().elements_of_domain().extent(), 4);
}

#[test]
fn cleanup_polygon_removes_interior_victim_spokes_and_triangles() {
    let mesh = fixture::polygon_mesh_with_center();
    let mut delaun = Delaun::from_mesh(&mesh, -1, -1, false).unwrap();
    let polygon = Sequence::from_iter([1, 2, 3, 4]);

    delaun.cleanup_polygon_probe(&polygon).unwrap();

    assert!(delaun.result().elements_of_domain().is_empty());
    assert_eq!(delaun.result().links_of_domain().extent(), 4);
}

#[test]
fn internal_constraint_walks_each_missing_side() {
    let mesh = fixture::polygon_mesh_with_one_sided_fixed_diagonal();
    let mut delaun = Delaun::from_mesh(&mesh, -1, -1, false).unwrap();

    delaun.insert_internal_edges_probe().unwrap();

    assert_eq!(delaun.result().elements_of_domain().extent(), 2);
}

#[test]
fn process_constraints_recovers_an_empty_frontier_square() {
    let mesh = fixture::polygon_mesh(0, true);
    let mut delaun = Delaun::from_mesh(&mesh, -1, -1, false).unwrap();

    delaun.process_constraints().unwrap();

    assert_eq!(delaun.result().elements_of_domain().extent(), 2);
}

#[test]
fn create_triangles_fans_an_interior_vertex_across_a_convex_boundary() {
    let mesh = fixture::polygon_mesh(0, false);
    let center = mesh
        .add_node(&Vertex::from_uv(1., 1., Freedom::Free), true)
        .unwrap();
    let mut polygon = DataMap::default();
    for edge in 1..=4 {
        polygon.bind(edge, true);
    }
    let mut delaun = Delaun::from_mesh(&mesh, -1, -1, false).unwrap();

    delaun.create_triangles_probe(center, &mut polygon).unwrap();

    assert!(polygon.is_empty());
    assert_eq!(delaun.result().elements_of_domain().extent(), 4);
}

#[test]
fn batch_insertion_replaces_the_containing_circumcircle_cavity() {
    let mesh = fixture::mesh(0, 0);
    let vertex = mesh
        .add_node(&Vertex::from_uv(1., 0.5, Freedom::Free), true)
        .unwrap();
    let mut delaun = Delaun::from_mesh(&mesh, -1, -1, false).unwrap();
    delaun.init_circles_tool(8, 8).unwrap();

    delaun.insert_vertices_probe(&mut [vertex]).unwrap();

    assert_eq!(delaun.result().elements_of_domain().extent(), 4);
}

#[test]
fn batch_insertion_orders_and_replaces_two_triangle_cavities() {
    let mesh = fixture::mesh(0, 0);
    let mut vertices = [
        mesh.add_node(&Vertex::from_uv(1., 0.5, Freedom::Free), true)
            .unwrap(),
        mesh.add_node(&Vertex::from_uv(0.5, 1.5, Freedom::Free), true)
            .unwrap(),
    ];
    let mut delaun = Delaun::from_mesh(&mesh, -1, -1, false).unwrap();
    delaun.init_circles_tool(8, 8).unwrap();

    delaun.insert_vertices_probe(&mut vertices).unwrap();

    assert_eq!(delaun.result().elements_of_domain().extent(), 6);
}

#[test]
fn batch_insertion_skips_frontier_edge_and_outside_points() {
    for (flags, point, expected_links) in [(1, Xy::new(1., 0.), 1), (0, Xy::new(3., 3.), 0)] {
        let mesh = fixture::mesh(0, flags);
        let vertex = mesh
            .add_node(&Vertex::new(point, 0, Freedom::Free), true)
            .unwrap();
        let mut delaun = Delaun::from_mesh(&mesh, -1, -1, false).unwrap();
        delaun.init_circles_tool(8, 8).unwrap();

        delaun.insert_vertices_probe(&mut [vertex]).unwrap();

        assert!(delaun.result().elements_of_domain().is_empty());
        assert_eq!(delaun.result().links_of_domain().extent(), expected_links);
    }
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
