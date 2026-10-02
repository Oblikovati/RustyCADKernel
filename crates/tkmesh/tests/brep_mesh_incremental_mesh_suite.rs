//! Rust mirrors for the full BRepMesh_IncrementalMesh GTest source.
#![allow(non_snake_case)]
use rustycad_tkbrep::{
    brep::BRepTool,
    topexp::Explorer,
    topods::{self, Builder, Compound, Shape, Wire},
};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::{
    gp::{
        Axis2, Axis3, Circle3, Cone3, Cylinder3, Direction3, Plane3, Point2, Point3, Sphere3,
        StandardDirection3, Vector3,
    },
    poly::mesh_purpose,
};
use rustycad_tkmesh::{brep_mesh::BRepMeshIncrementalMesh, imesh_tools::MeshParameters};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape, BRepBuilderAPIMakeWire,
};

#[test]
fn BRepMesh_IncrementalMeshTest__TrimmedCylinder_RespectsUVRangeAndDeflection() {
    let cylinder = Cylinder3::new(
        Axis3::with_standard(Point3::default(), StandardDirection3::Z),
        5.0,
    )
    .unwrap();
    let maker =
        BRepBuilderAPIMakeFace::from_cylinder_bounds(cylinder, (0.4, 5.7, -2.0, 7.0)).unwrap();
    let face = maker.face().unwrap();
    let shape = face.clone().into_shape();
    let parameters = MeshParameters {
        deflection: 0.05,
        ..Default::default()
    };
    BRepMeshIncrementalMesh::with_parameters(&shape, &parameters, &ProgressRange::default())
        .unwrap();
    assert_trimmed_cylinder_mesh(&face, parameters.deflection);
}

#[test]
fn BRepMesh_IncrementalMeshTest__OCC31461_AllowQualityDecreaseRebuildsMesh() {
    let sphere = Sphere3::new(
        Axis3::with_standard(Point3::default(), StandardDirection3::Z),
        10.0,
    )
    .unwrap();
    let maker = BRepBuilderAPIMakeFace::from_sphere(sphere).unwrap();
    let face = maker.face().unwrap();
    let shape = face.clone().into_shape();
    mesh_face(&shape, 0.01, false).unwrap();
    let fine = face_mesh_counts(&face);
    mesh_face(&shape, 0.1, true).unwrap();
    let coarse = face_mesh_counts(&face);
    assert!(coarse.0 < fine.0);
    assert!(coarse.1 < fine.1);
}

fn mesh_face(
    shape: &rustycad_tkbrep::topods::Shape,
    deflection: f64,
    allow_quality_decrease: bool,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    BRepMeshIncrementalMesh::with_parameters(
        shape,
        &MeshParameters {
            deflection,
            allow_quality_decrease,
            ..Default::default()
        },
        &ProgressRange::default(),
    )?;
    Ok(())
}

fn face_mesh_counts(face: &rustycad_tkbrep::topods::Face) -> (i32, i32) {
    let triangulation = BRepTool::triangulation(face, mesh_purpose::NONE).unwrap().0;
    (triangulation.nb_nodes(), triangulation.nb_triangles())
}

#[test]
fn BRepMesh_IncrementalMeshTest__OCC25445_AngularDeflectionChangesMesh() {
    let coarse = cone_mesh_counts(10.0_f64.to_radians()).unwrap();
    let fine = cone_mesh_counts(1.0_f64.to_radians()).unwrap();
    assert_ne!(coarse.0, fine.0);
    assert_ne!(coarse.1, fine.1);
}

fn cone_mesh_counts(angle: f64) -> Result<(i32, i32), rustycad_tkernel::standard::Failure> {
    let cone = Cone3::new(
        Axis3::with_standard(Point3::default(), StandardDirection3::Z),
        0.3,
        100.0,
    )?;
    let maker =
        BRepBuilderAPIMakeFace::from_cone_bounds(cone, (0.0, std::f64::consts::TAU, 0.0, 100.0))?;
    let face = maker.face()?;
    let shape = face.clone().into_shape();
    let result = BRepMeshIncrementalMesh::with_parameters(
        &shape,
        &MeshParameters {
            deflection: 0.01,
            angle,
            ..Default::default()
        },
        &ProgressRange::default(),
    )?;
    assert_eq!(result.get_status_flags(), 0);
    let triangulation = BRepTool::triangulation(&face, mesh_purpose::NONE)?.0;
    Ok((triangulation.nb_nodes(), triangulation.nb_triangles()))
}

#[test]
fn BRepMesh_IncrementalMeshTest__ClosedCylinder_SeamPolygonsReferenceValidMeshNodes() {
    let cylinder = Cylinder3::new(
        Axis3::with_standard(Point3::default(), StandardDirection3::Z),
        5.0,
    )
    .unwrap();
    let maker = BRepBuilderAPIMakeFace::from_cylinder_bounds(
        cylinder,
        (0.0, std::f64::consts::TAU, 0.0, 10.0),
    )
    .unwrap();
    let face = maker.face().unwrap();
    let shape = face.clone().into_shape();
    let mesher = BRepMeshIncrementalMesh::with_parameters(
        &shape,
        &MeshParameters {
            deflection: 0.1,
            ..Default::default()
        },
        &ProgressRange::default(),
    )
    .unwrap();
    assert_eq!(mesher.get_status_flags(), 0);
    assert_cylinder_seam_polygons(&face);
}

#[test]
fn BRepMesh_IncrementalMeshTest__OCC32692_UnboundedCylinderFaceReportsFailure() {
    let cylinder = Cylinder3::new(
        Axis3::with_standard(Point3::default(), StandardDirection3::Z),
        10.0,
    )
    .unwrap();
    let maker = BRepBuilderAPIMakeFace::from_cylinder(cylinder).unwrap();
    let face = maker.face().unwrap();
    let result = BRepMeshIncrementalMesh::with_parameters(
        &face.clone().into_shape(),
        &MeshParameters {
            deflection: 0.01,
            angle: 0.5,
            in_parallel: true,
            ..Default::default()
        },
        &ProgressRange::default(),
    );
    let mesher = result.expect("native IsDone remains true for failed face triangulation");
    assert_eq!(mesher.get_status_flags(), 69);
    let triangulation = BRepTool::triangulation(&face, mesh_purpose::NONE)
        .unwrap()
        .0;
    assert!(triangulation.is_null() || triangulation.nb_nodes() == 0);
    assert!(triangulation.is_null() || triangulation.nb_triangles() == 0);
}

#[test]
fn BRepMesh_IncrementalMeshTest__OCC26407_PlanarPolygonMeshStatus() {
    let points = planar_regression_points();
    let wire = closed_polygon_wire(&points).unwrap();
    let normal =
        Vector3::between(points[0], points[1]).crossed(Vector3::between(points[0], points[7]));
    let normal = Direction3::new(normal.x(), normal.y(), normal.z()).unwrap();
    let plane = Plane3::new(points[0], normal).unwrap();
    let face_maker = BRepBuilderAPIMakeFace::from_plane_wire(plane, &wire, true).unwrap();
    let face = face_maker.face().unwrap();
    let shape = face.clone().into_shape();
    let result = BRepMeshIncrementalMesh::with_parameters(
        &shape,
        &MeshParameters {
            deflection: 1e-7,
            ..Default::default()
        },
        &ProgressRange::default(),
    );
    let mesher = result.expect("planar polygon meshing should complete");
    assert_eq!(mesher.get_status_flags(), 0);
    let triangulation = BRepTool::triangulation(&face, mesh_purpose::NONE)
        .unwrap()
        .0;
    assert!(!triangulation.is_null());
    assert!(triangulation.nb_triangles() > 0);
}

#[test]
fn BRepMesh_IncrementalMeshTest__OCC30234_ZeroLengthFreeEdgeHasNoTriangles() {
    let circle = Circle3::new(Axis2::from_standard(StandardDirection3::Z), 0.0).unwrap();
    let mut maker = BRepBuilderAPIMakeEdge::from_circ(circle).unwrap();
    let shape = maker.shape().unwrap().clone();
    let parameters = MeshParameters {
        deflection: 0.005,
        relative: true,
        ..Default::default()
    };
    let result =
        BRepMeshIncrementalMesh::with_parameters(&shape, &parameters, &ProgressRange::default());
    assert!(
        result.is_ok(),
        "zero-length free-edge meshing failed: {}",
        result
            .err()
            .map(|failure| failure.exception_type().to_string())
            .unwrap_or_default()
    );
    assert_zero_length_edge_has_no_face_mesh(&shape);
}

#[test]
fn BRepMesh_IncrementalMeshTest__OCC31125_EmptyCompoundDoesNotCrash() {
    let mut empty_compound = Compound::default();
    Builder.make_compound(&mut empty_compound);
    let parameters = MeshParameters {
        deflection: 1.0,
        ..Default::default()
    };
    BRepMeshIncrementalMesh::with_parameters(
        &empty_compound,
        &parameters,
        &ProgressRange::default(),
    )
    .expect("empty compound meshing should not throw");
}

fn assert_zero_length_edge_has_no_face_mesh(shape: &Shape) {
    let mut faces = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    let mut node_count = 0;
    let mut triangle_count = 0;
    while faces.more() {
        let face = topods::face(faces.current()).unwrap();
        let triangulation = BRepTool::triangulation(&face, mesh_purpose::NONE)
            .unwrap()
            .0;
        assert!(!triangulation.is_null());
        node_count += triangulation.nb_nodes();
        triangle_count += triangulation.nb_triangles();
        faces.next().unwrap();
    }
    assert_eq!(node_count, 0);
    assert_eq!(triangle_count, 0);
}

fn planar_regression_points() -> [Point3; 8] {
    [
        Point3::new(587.9000000000001, 40.675817923051625, 88.5),
        Point3::new(807.8241820769484, 260.59999999999997, 88.5),
        Point3::new(644.1741820769485, 424.24999999999994, 88.50000000000001),
        Point3::new(629.978025792619, 424.25, 88.5),
        Point3::new(793.6280257926187, 260.59999999999985, 88.5),
        Point3::new(587.9000000000002, 54.87197420738135, 88.5),
        Point3::new(218.52197420738142, 424.25000000000006, 88.5),
        Point3::new(204.3258179230519, 424.24999999999994, 88.5),
    ]
}

fn closed_polygon_wire(points: &[Point3; 8]) -> Result<Wire, rustycad_tkernel::standard::Failure> {
    let mut maker = BRepBuilderAPIMakeWire::new();
    for index in 0..points.len() {
        let next = (index + 1) % points.len();
        let edge_maker = BRepBuilderAPIMakeEdge::from_points(points[index], points[next])?;
        maker.add_edge(&edge_maker.edge()?)?;
    }
    maker.wire()
}

fn assert_trimmed_cylinder_mesh(face: &rustycad_tkbrep::topods::Face, deflection: f64) {
    let triangulation = BRepTool::triangulation(face, mesh_purpose::NONE).unwrap().0;
    assert!(!triangulation.is_null());
    assert!(triangulation.has_uv_nodes());
    assert!(triangulation.nb_triangles() > 0);
    let (surface, location) = BRepTool::surface_with_location(face).unwrap();
    assert_cylinder_nodes(&triangulation, &surface, &location);
    assert_cylinder_deflection(&triangulation, &surface, &location, deflection);
}

fn assert_cylinder_nodes(
    triangulation: &rustycad_tkmath::poly::Triangulation,
    surface: &rustycad_tkernel::standard::Handle<dyn rustycad_tkg3d::geom::GeomSurface>,
    location: &rustycad_tkmath::toploc::Location,
) {
    for index in 1..=triangulation.nb_nodes() {
        let uv = triangulation.uv_node(index).unwrap();
        assert!((0.4 - 1e-9..=5.7 + 1e-9).contains(&uv.x()));
        assert!((-2.0 - 1e-9..=7.0 + 1e-9).contains(&uv.y()));
        let expected = surface
            .value(uv.x(), uv.y())
            .unwrap()
            .transformed(location.transformation());
        assert!(triangulation.node(index).unwrap().distance(&expected) <= 1e-7);
    }
}

fn assert_cylinder_deflection(
    triangulation: &rustycad_tkmath::poly::Triangulation,
    surface: &rustycad_tkernel::standard::Handle<dyn rustycad_tkg3d::geom::GeomSurface>,
    location: &rustycad_tkmath::toploc::Location,
    deflection: f64,
) {
    for index in 1..=triangulation.nb_triangles() {
        let (a, b, c) = triangulation.triangle(index).unwrap().get();
        let uv = triangle_uv_centroid(triangulation, a, b, c);
        let expected = surface
            .value(uv.x(), uv.y())
            .unwrap()
            .transformed(location.transformation());
        let actual = triangle_point_centroid(triangulation, a, b, c);
        assert!(actual.distance(&expected) <= deflection);
    }
}

fn triangle_uv_centroid(
    triangulation: &rustycad_tkmath::poly::Triangulation,
    a: i32,
    b: i32,
    c: i32,
) -> Point2 {
    let points = [
        triangulation.uv_node(a).unwrap(),
        triangulation.uv_node(b).unwrap(),
        triangulation.uv_node(c).unwrap(),
    ];
    Point2::new(
        points.iter().map(|point| point.x()).sum::<f64>() / 3.0,
        points.iter().map(|point| point.y()).sum::<f64>() / 3.0,
    )
}

fn triangle_point_centroid(
    triangulation: &rustycad_tkmath::poly::Triangulation,
    a: i32,
    b: i32,
    c: i32,
) -> Point3 {
    let points = [
        triangulation.node(a).unwrap(),
        triangulation.node(b).unwrap(),
        triangulation.node(c).unwrap(),
    ];
    Point3::new(
        points.iter().map(|point| point.x()).sum::<f64>() / 3.0,
        points.iter().map(|point| point.y()).sum::<f64>() / 3.0,
        points.iter().map(|point| point.z()).sum::<f64>() / 3.0,
    )
}

fn assert_cylinder_seam_polygons(face: &rustycad_tkbrep::topods::Face) {
    let (triangulation, location) = BRepTool::triangulation(face, mesh_purpose::NONE).unwrap();
    assert!(!triangulation.is_null());
    let shape = face.clone().into_shape();
    let mut explorer = Explorer::new(&shape, ShapeEnum::Edge, ShapeEnum::Shape).unwrap();
    let mut seam_count = 0;
    while explorer.more() {
        let edge = topods::edge(explorer.current()).unwrap();
        explorer.next().unwrap();
        if !BRepTool::is_closed_on_face(&edge, face).unwrap() {
            continue;
        }
        let forward = BRepTool::polygon_on_triangulation(&edge, &triangulation, &location).unwrap();
        let reversed = topods::edge(&edge.reversed()).unwrap();
        let backward =
            BRepTool::polygon_on_triangulation(&reversed, &triangulation, &location).unwrap();
        assert!(!forward.is_null());
        assert!(!backward.is_null());
        assert_polygon_node_indices(&forward, triangulation.nb_nodes());
        assert_polygon_node_indices(&backward, triangulation.nb_nodes());
        seam_count += 1;
    }
    assert!(seam_count > 0, "periodic face must expose a seam edge");
}

fn assert_polygon_node_indices(
    polygon: &rustycad_tkmath::poly::PolygonOnTriangulation,
    mesh_nodes: i32,
) {
    for index in 1..=polygon.nb_nodes() {
        let node = polygon.node(index).unwrap();
        assert!((1..=mesh_nodes).contains(&node));
    }
}
