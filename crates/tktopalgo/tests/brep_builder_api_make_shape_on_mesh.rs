//! Behavior tests of `BRepBuilderAPI_MakeShapeOnMesh` (no upstream suite uses it): the
//! shared topology of a closed mesh, the empty and null meshes, the skipped degenerated
//! triangles and the failure of the edge maker on nodes closer than the confusion.
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::poly::{Triangle, Triangulation};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeShape, BRepBuilderAPIMakeShapeOnMesh,
};

fn mesh(nodes: &[[f64; 3]], triangles: &[[i32; 3]]) -> Handle<Triangulation> {
    let len = |n: usize| i32::try_from(n).unwrap();
    let mesh =
        Triangulation::with_sizes(len(nodes.len()), len(triangles.len()), false, false).unwrap();
    for (i, [x, y, z]) in (1..).zip(nodes) {
        mesh.set_node(i, Point3::new(*x, *y, *z)).unwrap();
    }
    for (i, [a, b, c]) in (1..).zip(triangles) {
        mesh.set_triangle(i, Triangle::new(*a, *b, *c)).unwrap();
    }
    Handle::new(mesh)
}

fn count(shape: &Shape, kind: ShapeEnum) -> usize {
    let mut map = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(shape, kind, &mut map).unwrap();
    map.extent()
}

const TETRA: [[f64; 3]; 4] = [
    [0.0, 0.0, 0.0],
    [1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, 0.0, 1.0],
];

#[test]
fn a_closed_mesh_gives_faces_sharing_vertices_and_edges() {
    let tetra = mesh(&TETRA, &[[1, 3, 2], [1, 2, 4], [2, 3, 4], [3, 1, 4]]);
    let mut maker = BRepBuilderAPIMakeShapeOnMesh::new(&tetra);
    maker.build(&ProgressRange::default()).unwrap();
    assert!(maker.is_done());
    let shape = maker.shape().unwrap().clone();
    assert_eq!(shape.shape_type().unwrap(), ShapeEnum::Compound);
    assert_eq!(count(&shape, ShapeEnum::Face), 4);
    assert_eq!(count(&shape, ShapeEnum::Edge), 6);
    assert_eq!(count(&shape, ShapeEnum::Vertex), 4);
}

#[test]
fn a_null_or_triangle_free_mesh_builds_nothing() {
    for empty in [Handle::default(), mesh(&TETRA, &[])] {
        let mut maker = BRepBuilderAPIMakeShapeOnMesh::new(&empty);
        maker.build(&ProgressRange::default()).unwrap();
        assert!(!maker.is_done());
        let error = maker.shape().unwrap_err();
        assert_eq!(error.exception_type(), "StdFail_NotDone");
    }
}

#[test]
fn degenerated_and_collinear_triangles_get_no_face() {
    let nodes = [
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [2.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 0.0, 0.0],
    ];
    let triangles = [[1, 2, 4], [1, 1, 2], [1, 2, 3], [2, 5, 4]];
    let mut maker = BRepBuilderAPIMakeShapeOnMesh::new(&mesh(&nodes, &triangles));
    let shape = maker.shape().unwrap().clone();
    // Only the first triangle has a face: a repeated index, collinear nodes and
    // coincident nodes are skipped.
    assert_eq!(count(&shape, ShapeEnum::Face), 1);
}

#[test]
fn nodes_closer_than_the_confusion_fail_the_edge_maker() {
    let nodes = [[0.0, 0.0, 0.0], [1.0e-8, 0.0, 0.0], [0.0, 1.0, 0.0]];
    let mut maker = BRepBuilderAPIMakeShapeOnMesh::new(&mesh(&nodes, &[[1, 2, 3]]));
    let error = maker.build(&ProgressRange::default()).unwrap_err();
    assert_eq!(error.exception_type(), "StdFail_NotDone");
    assert!(!maker.is_done());
}
