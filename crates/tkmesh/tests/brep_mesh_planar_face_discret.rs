//! Exact native face-mesh output for a planar square with one square hole.
use rustycad_tkbrep::{
    brep::BRepTool,
    topexp::Explorer,
    topods::{Edge, Shape},
};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::{
    gp::{Plane3, Point2, Point3},
    poly::mesh_purpose,
};
use rustycad_tkmesh::{brep_mesh::BRepMeshIncrementalMesh, imesh_tools::MeshParameters};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape, BRepBuilderAPIMakeWire,
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

#[test]
fn box_faces_match_native_plane_mesh_count() {
    let shape = BRepPrimAPIMakeBox::from_sizes(10.0, 10.0, 10.0)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    BRepMeshIncrementalMesh::with_parameters(
        &shape,
        &MeshParameters {
            deflection: 0.5,
            ..Default::default()
        },
        &ProgressRange::default(),
    )
    .unwrap();
    let expected_nodes = box_native_nodes();
    let expected_triangles = [
        [(2, 1, 3), (2, 3, 4)],
        [(2, 1, 3), (2, 3, 4)],
        [(4, 2, 1), (4, 1, 3)],
        [(4, 2, 1), (4, 1, 3)],
        [(4, 2, 1), (4, 1, 3)],
        [(4, 2, 1), (4, 1, 3)],
    ];
    let mut explorer = Explorer::new(&shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    let mut face_count = 0;
    while explorer.more() {
        let face = rustycad_tkbrep::topods::face(explorer.current()).unwrap();
        let triangulation = BRepTool::triangulation(&face, mesh_purpose::NONE)
            .unwrap()
            .0;
        assert_native_box_face(
            &triangulation,
            &expected_nodes[face_count],
            &expected_triangles[face_count],
        );
        face_count += 1;
        explorer.next().unwrap();
    }
    assert_eq!(face_count, 6);
}

fn box_native_nodes() -> [[[f64; 5]; 4]; 6] {
    [
        [
            [0., 0., 0., 0., 0.],
            [0., 0., 10., 10., 0.],
            [0., 10., 0., 0., -10.],
            [0., 10., 10., 10., -10.],
        ],
        [
            [10., 0., 0., 0., 0.],
            [10., 0., 10., 10., 0.],
            [10., 10., 0., 0., -10.],
            [10., 10., 10., 10., -10.],
        ],
        [
            [0., 0., 0., 0., 0.],
            [10., 0., 0., 0., 10.],
            [0., 0., 10., 10., 0.],
            [10., 0., 10., 10., 10.],
        ],
        [
            [0., 10., 0., 0., 0.],
            [10., 10., 0., 0., 10.],
            [0., 10., 10., 10., 0.],
            [10., 10., 10., 10., 10.],
        ],
        [
            [0., 0., 0., 0., 0.],
            [0., 10., 0., 0., 10.],
            [10., 0., 0., 10., 0.],
            [10., 10., 0., 10., 10.],
        ],
        [
            [0., 0., 10., 0., 0.],
            [0., 10., 10., 0., 10.],
            [10., 0., 10., 10., 0.],
            [10., 10., 10., 10., 10.],
        ],
    ]
}

fn assert_native_box_face(
    triangulation: &rustycad_tkmath::poly::Triangulation,
    expected_nodes: &[[f64; 5]; 4],
    expected_triangles: &[(i32, i32, i32); 2],
) {
    assert_eq!(triangulation.nb_nodes(), 4);
    assert_eq!(triangulation.nb_triangles(), 2);
    for (offset, [x, y, z, u, v]) in expected_nodes.iter().enumerate() {
        let index = offset as i32 + 1;
        assert_eq!(triangulation.node(index).unwrap(), Point3::new(*x, *y, *z));
        assert_eq!(triangulation.uv_node(index).unwrap(), Point2::new(*u, *v));
    }
    for (offset, expected) in expected_triangles.iter().enumerate() {
        assert_eq!(
            triangulation.triangle(offset as i32 + 1).unwrap().get(),
            *expected
        );
    }
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
