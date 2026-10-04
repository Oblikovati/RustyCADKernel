//! Allocator propagation from coherent meshes into node triangle rings.
use rustycad_tkernel::{ncollection::BaseAllocator, precision, standard::Handle};
use rustycad_tkmath::gp::{Point3, Xyz};
use rustycad_tkmath::poly::{CoherentNode, CoherentTriangulation, Triangle, Triangulation};

fn source_mesh() -> Handle<Triangulation> {
    let mesh = Triangulation::with_sizes(4, 1, false, false).unwrap();
    for (index, (x, y)) in [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0)]
        .into_iter()
        .enumerate()
    {
        mesh.set_node(index as i32 + 1, Point3::new(x, y, 0.0))
            .unwrap();
    }
    mesh.set_triangle(1, Triangle::new(1, 2, 3)).unwrap();
    Handle::new(mesh)
}

#[test]
fn mesh_constructor_allocator_owns_new_and_removed_incident_members() {
    let allocator = BaseAllocator::incremental(4096);
    let wrong = BaseAllocator::incremental(4096);
    let mut mesh = CoherentTriangulation::new_in(allocator.clone());
    assert_eq!(mesh.allocator(), &allocator);
    for index in 0..4 {
        mesh.set_node(Xyz::new(index as f64, 0.0, 0.0), index);
    }
    let triangle = mesh.add_triangle(0, 1, 3).unwrap().unwrap();
    assert_allocator_rejection(mesh.change_node(0).unwrap(), triangle, &wrong);
    assert_allocator_has_live_members(&allocator);
    assert!(mesh.remove_triangle(triangle).unwrap());
    mesh.add_triangle(0, 1, 3).unwrap().unwrap();
    assert_allocator_has_live_members(&allocator);
    drop(mesh);
    allocator.reset(false).unwrap();
}

#[test]
fn copied_mesh_uses_its_allocator_for_incident_rings_and_clear() {
    let allocator = BaseAllocator::incremental(4096);
    let wrong = BaseAllocator::incremental(4096);
    let mut mesh =
        CoherentTriangulation::from_triangulation_in(&source_mesh(), allocator.clone()).unwrap();
    assert_eq!(mesh.allocator(), &allocator);
    assert_allocator_rejection(mesh.change_node(0).unwrap(), 0, &wrong);
    assert_allocator_has_live_members(&allocator);
    assert!(mesh.node(0).unwrap().triangles().eq([0]));
    mesh.change_node(0).unwrap().clear_in(&allocator).unwrap();
    assert!(mesh.node(0).unwrap().is_free_node());
    drop(mesh);
    allocator.reset(false).unwrap();
}

#[test]
fn clone_constructor_allocator_owns_the_copied_incident_rings() {
    let source_allocator = BaseAllocator::incremental(4096);
    let clone_allocator = BaseAllocator::incremental(4096);
    let mesh =
        CoherentTriangulation::from_triangulation_in(&source_mesh(), source_allocator).unwrap();
    let mut clone = mesh
        .clone_triangulation_in(clone_allocator.clone())
        .unwrap()
        .unwrap();
    assert_eq!(clone.allocator(), &clone_allocator);
    assert_allocator_has_live_members(&clone_allocator);
    assert_allocator_rejection(
        clone.change_node(0).unwrap(),
        0,
        &BaseAllocator::incremental(4096),
    );
    drop(clone);
    clone_allocator.reset(false).unwrap();
}

#[test]
fn node_add_remove_preserve_ring_order_and_use_explicit_allocator() {
    let allocator = BaseAllocator::incremental(4096);
    let wrong = BaseAllocator::incremental(4096);
    let mut node = CoherentNode::default();
    node.add_triangle_in(8, allocator.clone()).unwrap();
    node.add_triangle_in(9, allocator.clone()).unwrap();
    assert_eq!(node.triangles().collect::<Vec<_>>(), [8, 9]);
    assert_allocator_rejection(&mut node, 8, &wrong);
    assert!(node.remove_triangle_in(8, &allocator).unwrap());
    assert_eq!(node.triangles().collect::<Vec<_>>(), [9]);
}

#[test]
fn node_clear_rejects_mismatch_then_resets_geometry_with_matching_allocator() {
    let allocator = BaseAllocator::incremental(4096);
    let wrong = BaseAllocator::incremental(4096);
    let mut node = CoherentNode::new(Xyz::new(2.0, 3.0, 4.0));
    node.set_uv(0.25, 0.75);
    node.set_index(14);
    node.add_triangle_in(9, allocator.clone()).unwrap();
    let error = node.clear_in(&wrong).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_ProgramError");
    assert!(!node.is_free_node());
    assert_eq!((node.get_u(), node.get_v()), (0.25, 0.75));
    node.clear_in(&allocator).unwrap();
    assert!(node.is_free_node());
    assert_eq!(
        (node.get_u(), node.get_v()),
        (precision::infinite(), precision::infinite())
    );
    assert_eq!((node.x(), node.y(), node.z()), (0.0, 0.0, 0.0));
    assert_eq!(node.get_index(), 14);
    allocator.reset(false).unwrap();
}

fn assert_allocator_rejection(node: &mut CoherentNode, triangle: usize, wrong: &BaseAllocator) {
    let error = node.remove_triangle_in(triangle, wrong).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_ProgramError");
}

fn assert_allocator_has_live_members(allocator: &BaseAllocator) {
    let error = allocator.reset(false).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_ProgramError");
}
