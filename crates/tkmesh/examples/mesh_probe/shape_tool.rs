//! Native BRepMesh_ShapeTool probe twin.
use super::super::{flag, orientation, point2, point3, real};
use rustycad_tkbrep::{brep::BRepTool, topods};
use rustycad_tkernel::{ncollection::IncrementalAllocatorObject, standard::Handle};
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
#[path = "../../tests/support/shape_tool_fixture.rs"]
mod fixtures;
pub fn shape_tool_case(values: &[i32]) {
    let tolerance = [1e-7, 1., 100.][values[1] as usize];
    let span = [0.001, 1., 6.25][values[2] as usize];
    let (edge, face) = fixtures::tool_topology(values[0], tolerance, span);
    let edge = topods::edge(&edge.oriented(orientation(values[4]))).unwrap();
    let allocator = Handle::new(IncrementalAllocatorObject::default());
    let discrete = Handle::new(DiscreteEdge::new(&edge, &allocator).unwrap());
    let discrete_face: FaceHandle =
        Handle::new(DiscreteFace::new(&face, &allocator).unwrap()).cast();
    let pcurve = discrete.add_pcurve(discrete_face.downgrade(), orientation(values[4]));
    discrete.set_same_param(values[3] & 1 != 0);
    discrete.set_same_range(values[3] & 2 != 0);
    discrete.set_degenerated(values[3] & 4 != 0);
    ShapeTool::check_and_update_flags(&discrete.cast(), &pcurve).unwrap();
    flag(discrete.get_same_param());
    flag(discrete.get_same_range());
    flag(discrete.get_degenerated());
    real(ShapeTool::max_face_tolerance(&face).unwrap());
    query_shape_ranges(&edge, &face);
    let location = tool_location(values[5] != 0);
    point3(ShapeTool::use_location(Point3::new(2., 3., 4.), &location));
    let mut size = 17.;
    ShapeTool::box_max_dimension(&BoundingBox3::default(), &mut size).unwrap();
    real(size);
    let bounds = BoundingBox3::new(Point3::new(-2., -3., -4.), Point3::new(span, tolerance, 2.));
    ShapeTool::box_max_dimension(&bounds, &mut size).unwrap();
    real(size);
    println!();
}
fn query_shape_ranges(edge: &topods::Edge, face: &topods::Face) {
    for orient in [false, true] {
        let (curve, first, last) = ShapeTool::range_3d(edge, orient).unwrap();
        flag(!curve.is_null());
        if !curve.is_null() {
            real(first);
            real(last);
        }
        let (pcurve, first, last) = ShapeTool::range_on_face(edge, face, orient).unwrap();
        flag(!pcurve.is_null());
        if !pcurve.is_null() {
            real(first);
            real(last);
        }
        let mut first = Point2::new(19., 23.);
        let mut last = Point2::new(29., 31.);
        flag(ShapeTool::uv_points(edge, face, &mut first, &mut last, orient).unwrap());
        point2(first);
        point2(last);
    }
}
fn tool_location(located: bool) -> Location {
    if !located {
        return Location::default();
    }
    Location::new(Transform3::translation_vector(Vector3::new(2., 3., 4.)))
}
pub fn shape_polygons_case(located: bool, orient: i32) {
    let (edge, face) = fixtures::tool_topology(2, 1e-7, 1.);
    let edge = topods::edge(&edge.oriented(orientation(orient))).unwrap();
    let location = tool_location(located);
    let face = topods::face(&face.located(&location, false).unwrap()).unwrap();
    let mesh = Handle::new(Triangulation::with_sizes(3, 1, false, false).unwrap());
    for index in 1..=3 {
        mesh.set_node(index, Point3::new(f64::from(index), 5., 6.))
            .unwrap();
    }
    ShapeTool::add_in_face(&face, &mesh).unwrap();
    for index in 1..=3 {
        point3(mesh.node(index).unwrap());
    }
    flag(
        !BRepTool::triangulation(&face, mesh_purpose::NONE)
            .unwrap()
            .0
            .is_null(),
    );
    report_polygon_updates(&edge, &mesh, &location);
    ShapeTool::nullify_face(&face).unwrap();
    flag(
        BRepTool::triangulation(&face, mesh_purpose::NONE)
            .unwrap()
            .0
            .is_null(),
    );
    println!();
}
fn report_polygon_updates(edge: &topods::Edge, mesh: &Handle<Triangulation>, location: &Location) {
    let first = Handle::new(PolygonOnTriangulation::new(2, false).unwrap());
    let last = Handle::new(PolygonOnTriangulation::new(2, false).unwrap());
    ShapeTool::update_edge_on_triangulation(edge, &first, mesh, location).unwrap();
    flag(
        !BRepTool::polygon_on_triangulation(edge, mesh, location)
            .unwrap()
            .is_null(),
    );
    ShapeTool::update_edge_seam(edge, (&first, &last), mesh, location).unwrap();
    flag(BRepTool::is_closed_on_triangulation(edge, mesh, location).unwrap());
    ShapeTool::nullify_edge_on_triangulation(edge, mesh, location).unwrap();
    flag(
        BRepTool::polygon_on_triangulation(edge, mesh, location)
            .unwrap()
            .is_null(),
    );
    let polygon = Handle::new(Polygon3D::new(2, false).unwrap());
    ShapeTool::update_edge_polygon3d(edge, &polygon).unwrap();
    flag(!BRepTool::polygon3d(edge).unwrap().0.is_null());
    ShapeTool::nullify_edge_polygon3d(edge, &Location::default()).unwrap();
    flag(BRepTool::polygon3d(edge).unwrap().0.is_null());
}
