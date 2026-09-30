//! Native element identity, specialized hashes and adjacency-pair sentinel behavior.
use rustycad_tkernel::ncollection::HashKey;
use rustycad_tkmath::gp::Xy;
use rustycad_tkmesh::brep_mesh::{
    BRepMeshEdge, BRepMeshOrientedEdge, BRepMeshTriangle, BRepMeshVertex,
    DegreeOfFreedom as Freedom, PairOfIndex,
};
#[test]
fn native_constraint_discriminants_and_vertex_default() {
    let values = [
        Freedom::Free,
        Freedom::InVolume,
        Freedom::OnSurface,
        Freedom::OnCurve,
        Freedom::Fixed,
        Freedom::Frontier,
        Freedom::Deleted,
    ];
    for (index, value) in values.into_iter().enumerate() {
        assert_eq!(value as usize, index);
    }
    let vertex = BRepMeshVertex::default();
    assert_eq!(*vertex.coord(), Xy::default());
    assert_eq!(vertex.location_3d(), 0);
    assert_eq!(vertex.movability(), Freedom::Free);
    assert_eq!(vertex.occt_hash(), 0);
}
#[test]
fn vertex_coordinate_reference_and_initialization_preserve_association() {
    let mut vertex = BRepMeshVertex::new(Xy::new(1., 2.), 7, Freedom::Frontier);
    vertex.change_coord().set_x(4.);
    assert_eq!(vertex.coord().x(), 4.);
    assert_eq!(vertex.location_3d(), 7);
    vertex.initialize(Xy::new(-1., 3.), -2, Freedom::Fixed);
    assert_eq!(vertex.coord().y(), 3.);
    assert_eq!(vertex.location_3d(), -2);
    assert_eq!(vertex.movability(), Freedom::Fixed);
}
#[test]
fn vertex_equality_is_tolerant_and_deleted_is_not_reflexive() {
    let mut vertex = BRepMeshVertex::from_uv(0., 0., Freedom::Free);
    let near = BRepMeshVertex::new(Xy::new(5e-10, 0.), 99, Freedom::Fixed);
    assert_eq!(vertex, near);
    assert_ne!(vertex, BRepMeshVertex::from_uv(2e-9, 0., Freedom::Free));
    vertex.set_movability(Freedom::Deleted);
    assert!(!vertex.is_equal(&vertex));
    assert_ne!(vertex, near);
}
#[test]
fn oriented_and_undirected_links_preserve_different_equality() {
    let oriented = BRepMeshOrientedEdge::new(1, 2);
    assert_eq!((oriented.first_node(), oriented.last_node()), (1, 2));
    assert!(!oriented.is_equal(&BRepMeshOrientedEdge::new(2, 1)));
    assert_eq!(BRepMeshOrientedEdge::default().first_node(), -1);
    let mut edge = BRepMeshEdge::new(1, 2, Freedom::Free);
    let reverse = BRepMeshEdge::new(2, 1, Freedom::Fixed);
    assert_eq!(edge, reverse);
    assert!(!edge.is_same_orientation(&reverse));
    edge.set_movability(Freedom::Deleted);
    assert!(!edge.is_equal(&edge));
    assert!(edge.is_same_orientation(&edge));
    assert_eq!(BRepMeshEdge::default().movability(), Freedom::Deleted);
}
#[test]
fn link_hash_truncates_before_sorting_and_retains_host_byte_order() {
    let edge = BRepMeshEdge::new(65537, 2, Freedom::Free);
    assert_eq!(
        edge.occt_hash(),
        BRepMeshEdge::new(2, 1, Freedom::Fixed).occt_hash()
    );
    let [a, b] = 1u16.to_ne_bytes();
    let [c, d] = 2u16.to_ne_bytes();
    assert_eq!(edge.occt_hash(), u32::from_ne_bytes([a, b, c, d]) as usize);
    assert_eq!(
        BRepMeshOrientedEdge::new(1, 2).occt_hash(),
        edge.occt_hash()
    );
}
#[test]
fn triangle_equality_uses_cyclic_links_and_ignores_orientation() {
    let mut triangle = BRepMeshTriangle::default();
    assert_eq!(triangle.edges(), ([0; 3], [false; 3]));
    triangle.initialize([1, 2, 3], [true, false, true], Freedom::Frontier);
    let rotated = BRepMeshTriangle::new([2, 3, 1], [false; 3], Freedom::Fixed);
    assert_eq!(triangle, rotated);
    assert_eq!(triangle.occt_hash(), rotated.occt_hash());
    assert_ne!(
        triangle,
        BRepMeshTriangle::new([3, 2, 1], [false; 3], Freedom::Free)
    );
    triangle.set_movability(Freedom::Deleted);
    assert!(!triangle.is_equal(&triangle));
    triangle.edges[0] = 9;
    triangle.orientations[1] = true;
    triangle.movability = Freedom::Free;
    assert_eq!(triangle.edges(), ([9, 2, 3], [true; 3]));
    assert_eq!(triangle.movability(), Freedom::Free);
}
#[test]
fn pair_append_prepend_full_errors_removal_and_clear() {
    let mut pair = PairOfIndex::default();
    assert!(pair.is_empty());
    assert_eq!(pair.extent(), 0);
    pair.append(7).unwrap();
    pair.prepend(5).unwrap();
    assert_eq!(
        (pair.first_index(), pair.last_index(), pair.extent()),
        (5, 7, 2)
    );
    assert_eq!(
        pair.append(9).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    assert!(pair.prepend(9).is_err());
    pair.remove_index(1).unwrap();
    assert_eq!((pair.index(1).unwrap(), pair.index(2).unwrap()), (7, -1));
    pair.clear();
    assert_eq!((pair.first_index(), pair.last_index()), (-1, -1));
}
#[test]
fn pair_holes_and_negative_indices_keep_native_counterintuitive_state() {
    let mut pair = PairOfIndex::default();
    pair.set_index(2, 7).unwrap();
    assert!(pair.is_empty());
    assert_eq!(pair.extent(), 0);
    assert_eq!(pair.last_index(), 7);
    pair.append(5).unwrap();
    assert_eq!(pair.extent(), 2);
    pair.set_index(1, -3).unwrap();
    pair.append(-2).unwrap();
    assert_eq!(pair.first_index(), -2);
    assert_eq!(pair.index(2).unwrap(), 7);
    pair.remove_index(2).unwrap();
    assert_eq!(pair.last_index(), -2);
    for position in [-1, 0, 3] {
        assert!(pair.index(position).is_err());
        assert!(pair.set_index(position, 0).is_err());
        assert!(pair.remove_index(position).is_err());
    }
}
