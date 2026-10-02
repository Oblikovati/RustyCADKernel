//! Exact native face-mesh output for a planar square with one square hole.
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Edge, Shape},
};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkmath::{
    gp::{Plane3, Point3},
    poly::mesh_purpose,
};
use rustycad_tkmesh::{brep_mesh::BRepMeshIncrementalMesh, imesh_tools::MeshParameters};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeWire,
};

#[test]
fn square_hole_matches_pinned_native_node_and_triangle_counts() {
    let shape = square_hole();
    let parameters = MeshParameters {
        deflection: 0.1,
        ..Default::default()
    };
    BRepMeshIncrementalMesh::with_parameters(&shape, &parameters, &ProgressRange::default())
        .unwrap();
    let face = rustycad_tkbrep::topods::face(&shape).unwrap();
    let triangulation = BRepTool::triangulation(&face, mesh_purpose::NONE)
        .unwrap()
        .0;
    assert!(!triangulation.is_null());
    assert_eq!(
        triangulation.nb_nodes(),
        8,
        "pinned OCCT reports eight pcurve nodes"
    );
    assert_eq!(
        triangulation.nb_triangles(),
        2,
        "pinned OCCT retains two inner-loop triangles"
    );
    let expected = [
        (-5.0, -5.0),
        (5.0, -5.0),
        (5.0, 5.0),
        (-5.0, 5.0),
        (-1.0, -1.0),
        (1.0, -1.0),
        (1.0, 1.0),
        (-1.0, 1.0),
    ];
    for (offset, (x, y)) in expected.into_iter().enumerate() {
        let index = offset as i32 + 1;
        assert_eq!(triangulation.node(index).unwrap(), Point3::new(x, y, 0.0));
        assert_eq!(
            triangulation.uv_node(index).unwrap(),
            rustycad_tkmath::gp::Point2::new(x, y)
        );
    }
    assert_eq!(triangulation.triangle(1).unwrap().get(), (7, 5, 6));
    assert_eq!(triangulation.triangle(2).unwrap().get(), (7, 8, 5));
}

fn square_hole() -> Shape {
    let outer = square_wire(-5.0, 5.0);
    let inner = square_wire(-1.0, 1.0);
    let mut face =
        BRepBuilderAPIMakeFace::from_plane_wire(Plane3::default(), &outer, true).unwrap();
    face.add(&inner).unwrap();
    face.face().unwrap().into_shape()
}

fn square_wire(minimum: f64, maximum: f64) -> rustycad_tkbrep::topods::Wire {
    let points = [
        Point3::new(minimum, minimum, 0.0),
        Point3::new(maximum, minimum, 0.0),
        Point3::new(maximum, maximum, 0.0),
        Point3::new(minimum, maximum, 0.0),
    ];
    let edges: Vec<Edge> = (0..4)
        .map(|index| {
            BRepBuilderAPIMakeEdge::from_points(points[index], points[(index + 1) % 4])
                .unwrap()
                .edge()
                .unwrap()
        })
        .collect();
    BRepBuilderAPIMakeWire::from_four_edges(&edges[0], &edges[1], &edges[2], &edges[3])
        .unwrap()
        .wire()
        .unwrap()
}
