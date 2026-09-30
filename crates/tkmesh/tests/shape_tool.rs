//! Mesh topology utility behavior with actual curves, pcurves and polygons.
#[path = "support/shape_tool_fixture.rs"]
mod fixtures;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkernel::{ncollection::IncrementalAllocatorObject, standard::Handle};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::{
    bnd::BoundingBox3,
    gp::{Point2, Point3, Transform3, Vector3},
    poly::{Polygon3D, PolygonOnTriangulation, Triangulation, mesh_purpose},
    toploc::Location,
};
use rustycad_tkmesh::{
    brep_mesh::ShapeTool,
    brep_mesh_data::{DiscreteEdge, DiscreteFace},
    imesh_data::{FaceHandle, MeshEdge},
};

#[test]
fn bounds_locations_ranges_and_uv_queries_preserve_native_values() {
    let mut size = 17.;
    ShapeTool::box_max_dimension(&BoundingBox3::default(), &mut size).unwrap();
    assert_eq!(size, 17.);
    let bounds = BoundingBox3::new(Point3::new(-2., -3., -4.), Point3::new(3., 2., 5.));
    ShapeTool::box_max_dimension(&bounds, &mut size).unwrap();
    assert_eq!(size, 9.);
    let location = Location::new(Transform3::translation_vector(Vector3::new(2., 3., 4.)));
    assert_eq!(
        ShapeTool::use_location(Point3::default(), &location),
        Point3::new(2., 3., 4.)
    );
    let (edge, face) = fixtures::tool_topology(2, 0.01, 1.);
    assert_eq!(ShapeTool::max_face_tolerance(&face).unwrap(), 0.01);
    let reverse = rustycad_tkbrep::topods::edge(&edge.oriented(Orientation::Reversed)).unwrap();
    let (_, first, last) = ShapeTool::range_3d(&reverse, true).unwrap();
    assert_eq!((first, last), (1., 0.));
    let mut uv_first = Point2::new(19., 23.);
    let mut uv_last = uv_first;
    assert!(ShapeTool::uv_points(&reverse, &face, &mut uv_first, &mut uv_last, true).unwrap());
    assert_eq!(uv_first, Point2::new(1., 0.));
    assert_eq!(uv_last, Point2::default());
}
#[test]
fn geometric_flags_deteriorate_without_recovering_prior_failures() {
    let allocator = Handle::new(IncrementalAllocatorObject::default());
    for kind in 0..=4 {
        let (edge, face) = fixtures::tool_topology(kind, 2., 0.5);
        let model = Handle::new(DiscreteEdge::new(&edge, &allocator).unwrap());
        let discrete_face: FaceHandle =
            Handle::new(DiscreteFace::new(&face, &allocator).unwrap()).cast();
        let pcurve = model.add_pcurve(discrete_face.downgrade(), Orientation::Forward);
        ShapeTool::check_and_update_flags(&model.cast(), &pcurve).unwrap();
        assert_eq!(model.get_degenerated(), kind != 2);
        model.set_same_param(false);
        model.set_same_range(false);
        model.set_degenerated(true);
        ShapeTool::check_and_update_flags(&model.cast(), &pcurve).unwrap();
        assert!(!model.get_same_param());
        assert!(!model.get_same_range());
        assert!(model.get_degenerated());
    }
}
#[test]
fn stored_face_and_edge_polygons_use_native_placements_and_removal() {
    let (edge, face) = fixtures::tool_topology(2, 1e-7, 1.);
    let location = Location::new(Transform3::translation_vector(Vector3::new(2., 3., 4.)));
    let face = rustycad_tkbrep::topods::face(&face.located(&location, false).unwrap()).unwrap();
    let mesh = Handle::new(Triangulation::with_sizes(3, 1, false, false).unwrap());
    for index in 1..=3 {
        mesh.set_node(index, Point3::new(f64::from(index), 5., 6.))
            .unwrap();
    }
    ShapeTool::add_in_face(&face, &mesh).unwrap();
    assert_eq!(mesh.node(1).unwrap(), Point3::new(-1., 2., 2.));
    assert!(
        !BRepTool::triangulation(&face, mesh_purpose::NONE)
            .unwrap()
            .0
            .is_null()
    );
    polygon_updates(&edge, &mesh, &location);
    ShapeTool::nullify_face(&face).unwrap();
    assert!(
        BRepTool::triangulation(&face, mesh_purpose::NONE)
            .unwrap()
            .0
            .is_null()
    );
}
fn polygon_updates(
    edge: &rustycad_tkbrep::topods::Edge,
    mesh: &Handle<Triangulation>,
    location: &Location,
) {
    let first = Handle::new(PolygonOnTriangulation::new(2, false).unwrap());
    let last = Handle::new(PolygonOnTriangulation::new(2, false).unwrap());
    ShapeTool::update_edge_on_triangulation(edge, &first, mesh, location).unwrap();
    assert!(
        !BRepTool::polygon_on_triangulation(edge, mesh, location)
            .unwrap()
            .is_null()
    );
    ShapeTool::update_edge_seam(edge, (&first, &last), mesh, location).unwrap();
    assert!(BRepTool::is_closed_on_triangulation(edge, mesh, location).unwrap());
    ShapeTool::nullify_edge_on_triangulation(edge, mesh, location).unwrap();
    assert!(
        BRepTool::polygon_on_triangulation(edge, mesh, location)
            .unwrap()
            .is_null()
    );
    let polygon = Handle::new(Polygon3D::new(2, false).unwrap());
    ShapeTool::update_edge_polygon3d(edge, &polygon).unwrap();
    assert!(!BRepTool::polygon3d(edge).unwrap().0.is_null());
    ShapeTool::nullify_edge_polygon3d(edge, &Location::default()).unwrap();
    assert!(BRepTool::polygon3d(edge).unwrap().0.is_null());
}
