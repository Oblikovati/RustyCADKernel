//! BRep behaviour the upstream suites do not reach: the preserved native quirks, the
//! builder errors, the Rust deviations for undefined native behaviour and the ProjLib
//! gap of `CurveOnPlane` (docs/tkbrep-progress.md).
mod support;
use rustycad_tkbrep::brep::{BRepBuilder, BRepCurveOn2Surfaces, BRepTEdge, BRepTFace, BRepTool};
use rustycad_tkbrep::topods::{self, Builder, Edge, Face, Shape, Vertex};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomPlane, GeomSurface};
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::{
    Axis1, Axis2d, Axis3, Direction2, Direction3, Point2, Point3, Transform3,
};
use rustycad_tkmath::poly::{Triangle, Triangulation, mesh_purpose};
use rustycad_tkmath::toploc::Location;
use support::{make_edge, make_vertex};

fn plane() -> Handle<dyn GeomSurface> {
    Handle::new(GeomPlane::new(Axis3::default())).into_cast()
}

fn x_line() -> Handle<dyn GeomCurve> {
    Handle::new(GeomLine::new(Axis1::new(Point3::default(), Direction3::X))).into_cast()
}

fn u_line() -> Handle<dyn Geom2dCurve> {
    let axis = Axis2d::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0).unwrap());
    Handle::new(Geom2dLine::new(axis)).into_cast()
}

fn face_on_plane() -> Face {
    let mut face = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &plane(), 1e-7)
        .unwrap();
    face
}

fn triangulation() -> Handle<Triangulation> {
    let nodes = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];
    let triangles = [Triangle::new(1, 2, 3)];
    let t = Triangulation::from_nodes(
        &Array1::from_shared(&nodes, 1),
        &Array1::from_shared(&triangles, 1),
    );
    Handle::new(t.unwrap())
}

fn edge_tshape(edge: &Edge) -> Handle<BRepTEdge> {
    edge.tshape().cast::<BRepTEdge>()
}

#[test]
fn locked_shapes_refuse_every_update() {
    let builder = BRepBuilder::default();
    let edge = make_edge(Point3::default(), Point3::new(1.0, 0.0, 0.0));
    edge.set_locked(true).unwrap();
    let errors = [
        builder.update_edge_tolerance(&edge, 1.0).unwrap_err(),
        builder.same_range(&edge, false).unwrap_err(),
        builder.range(&edge, 0.0, 1.0, false).unwrap_err(),
        builder
            .update_edge_curve(&edge, &x_line(), 1e-7)
            .unwrap_err(),
    ];
    for error in errors {
        assert_eq!(error.exception_type(), "TopoDS_LockedShape");
    }
    let mut locked = edge.clone();
    assert_eq!(
        builder.make_edge(&mut locked).unwrap_err().exception_type(),
        "TopoDS_LockedShape"
    );
}

#[test]
fn builder_raises_the_native_domain_errors() {
    let builder = BRepBuilder::default();
    let edge = make_edge(Point3::default(), Point3::new(1.0, 0.0, 0.0));
    let vertex = make_vertex(Point3::default());
    let range = builder.range_on_surface(&edge, &plane(), &Location::default(), 0.0, 1.0);
    assert_eq!(range.unwrap_err().exception_type(), "Standard_DomainError");
    let infinite = builder.update_vertex_on_edge(&vertex, 1e101, &edge, 1e-7);
    assert_eq!(
        infinite.unwrap_err().exception_type(),
        "Standard_DomainError"
    );
    let no_pcurve = builder.update_vertex_on_edge_surface(
        &vertex,
        0.5,
        &edge,
        &plane(),
        &Location::default(),
        1e-7,
    );
    assert_eq!(
        no_pcurve.unwrap_err().exception_type(),
        "Standard_DomainError"
    );
}

#[test]
fn pcurve_and_curve_updates_follow_the_native_lists() {
    let builder = BRepBuilder::default();
    let edge = make_edge(Point3::default(), Point3::new(2.0, 0.0, 0.0));
    let face = face_on_plane();
    let pcurve = u_line();
    builder
        .update_edge_pcurve_on_face(&edge, &pcurve, &face, 1e-5)
        .unwrap();
    let found = BRepTool::curve_on_face(&edge, &face).unwrap();
    // The new p-curve takes the range of the 3D curve.
    assert!(found.is_stored && found.pcurve == pcurve);
    assert!(BRepTool::first_curve_on_surface(&edge).unwrap().pcurve == pcurve);
    assert_eq!((found.first, found.last), (0.0, 2.0));
    assert_eq!(BRepTool::edge_tolerance(&edge).unwrap(), 1e-5);
    builder
        .update_edge_pcurve_on_face(&edge, &Handle::default(), &face, 1e-7)
        .unwrap();
    assert_eq!(edge_tshape(&edge).get().unwrap().curves().len(), 1);
}

#[test]
fn seam_polygons_are_stored_at_the_identity_location() {
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    builder.make_edge(&mut edge).unwrap();
    let moved = Location::new(Transform3::translation([0.0, 0.0, 1.0]));
    let edge = Edge::unchecked(edge.located(&moved, false).unwrap());
    let polygon = Handle::new(rustycad_tkmath::poly::Polygon2D::new(2).unwrap());
    builder
        .update_edge_polygons_on_surface(
            &edge,
            (&polygon, &polygon),
            &plane(),
            &Location::default(),
        )
        .unwrap();
    let curves = edge_tshape(&edge).get().unwrap().curves();
    assert!(curves[0].get().unwrap().location().is_identity());
    // The search uses the edge-relative location, so it misses the stored polygon.
    assert!(
        BRepTool::polygon_on_surface(&edge, &plane(), &Location::default())
            .unwrap()
            .is_null()
    );
}

#[test]
fn regularities_and_their_continuity() {
    let builder = BRepBuilder::default();
    let edge = make_edge(Point3::default(), Point3::new(1.0, 0.0, 0.0));
    let (s1, s2) = (plane(), plane());
    let identity = Location::default();
    builder
        .continuity(&edge, (&s1, &s2), (&identity, &identity), Continuity::G1)
        .unwrap();
    builder
        .continuity(&edge, (&s2, &s1), (&identity, &identity), Continuity::C2)
        .unwrap();
    assert_eq!(BRepTool::max_continuity(&edge).unwrap(), Continuity::C2);
    assert!(
        BRepTool::has_continuity_on_surfaces(&edge, (&s1, &s2), (&identity, &identity)).unwrap()
    );
    let regularity = BRepCurveOn2Surfaces::new(&s1, &s2, &identity, &identity, Continuity::C0);
    assert_eq!(
        regularity.d0(0.0).unwrap_err().exception_type(),
        "Standard_NullObject"
    );
}

#[test]
fn internal_vertices_get_points_on_the_curves() {
    let builder = BRepBuilder::default();
    let mut edge = make_edge(Point3::default(), Point3::new(2.0, 0.0, 0.0));
    let inner = make_vertex(Point3::new(1.0, 0.0, 0.0));
    Builder
        .add(
            &mut edge,
            &inner.oriented(rustycad_tkg3d::top_abs::Orientation::Internal),
        )
        .unwrap();
    builder
        .update_vertex_on_edge(&inner, 1.0, &edge, 1e-7)
        .unwrap();
    let points = inner
        .tshape()
        .cast::<rustycad_tkbrep::brep::BRepTVertex>()
        .get()
        .unwrap()
        .points();
    assert!(points[0].get().unwrap().is_point_on_curve());
    assert_eq!(BRepTool::parameter(&inner, &edge).unwrap(), 1.0);
    let lonely = make_vertex(Point3::new(5.0, 5.0, 5.0));
    assert_eq!(
        BRepTool::parameter(&lonely, &edge)
            .unwrap_err()
            .exception_type(),
        "Standard_NoSuchObject"
    );
}

#[test]
fn triangulation_lists_follow_the_native_active_bits() {
    let face = BRepTFace::new();
    let (first, second) = (triangulation(), triangulation());
    face.set_triangulations(&[first.clone(), second.clone()], &second)
        .unwrap();
    assert!(face.active_triangulation() == second);
    assert_eq!(
        second.get().unwrap().mesh_purpose() & mesh_purpose::ACTIVE,
        mesh_purpose::ACTIVE
    );
    let error = face.set_triangulations(&[first.clone(), Handle::default()], &Handle::default());
    assert_eq!(error.unwrap_err().exception_type(), "Standard_ProgramError");
    // The Active bits of the members before the null one were cleared.
    assert_eq!(
        first.get().unwrap().mesh_purpose() & mesh_purpose::ACTIVE,
        0
    );
    assert!(face.triangulation(mesh_purpose::ANY_FALLBACK) == first);
}

#[test]
fn casts_of_foreign_or_null_tshapes_raise() {
    let null = Edge::default();
    assert_eq!(
        BRepTool::edge_tolerance(&null)
            .unwrap_err()
            .exception_type(),
        "Standard_NullObject"
    );
    let mut wire = rustycad_tkbrep::topods::Wire::default();
    Builder.make_wire(&mut wire);
    let not_an_edge = Edge::unchecked(Shape::from(wire));
    let error = BRepTool::edge_tolerance(&not_an_edge).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_TypeMismatch");
    let vertex = Vertex::default();
    assert_eq!(
        BRepTool::pnt(&vertex).unwrap_err().exception_type(),
        "Standard_NullObject"
    );
}

#[test]
fn planar_projection_gives_the_plane_pcurve() {
    // A line edge in the XOY plane without a stored p-curve: CurveOnSurface projects its
    // 3D curve on the plane, and the p-curve evaluates to the XY of the 3D points.
    let edge = make_edge(Point3::default(), Point3::new(1.0, 0.0, 0.0));
    let found = BRepTool::curve_on_surface(&edge, &plane(), &Location::default()).unwrap();
    assert!(!found.is_stored);
    assert_eq!((found.first, found.last), (0.0, 1.0));
    let point = found.pcurve.get().unwrap().value(0.5).unwrap();
    assert!((point.x() - 0.5).abs() <= 1e-12 && point.y().abs() <= 1e-12);
    // A non-planar surface or an edge without 3D curve returns a null curve.
    let mut bare = Edge::default();
    BRepBuilder::default().make_edge(&mut bare).unwrap();
    let found = BRepTool::curve_on_plane(&bare, &plane(), &Location::default()).unwrap();
    assert!(found.0.is_null() && (found.1, found.2) == (0.0, 0.0));
}

#[test]
fn empty_copies_keep_geometry_but_not_polygons() {
    let builder = BRepBuilder::default();
    let edge = make_edge(Point3::default(), Point3::new(1.0, 0.0, 0.0));
    let polygon = Handle::new(rustycad_tkmath::poly::Polygon3D::new(2, false).unwrap());
    builder
        .update_edge_polygon3d(&edge, &polygon, &Location::default())
        .unwrap();
    let copy = edge.tshape().get().unwrap().empty_copy().unwrap();
    let copy = copy.cast::<BRepTEdge>();
    let curves = copy.get().unwrap().curves();
    assert_eq!(curves.len(), 1);
    assert!(curves[0].get().unwrap().is_curve3d());
    let face = face_on_plane();
    let face_copy = topods::face(&face.empty_copied().unwrap()).unwrap();
    assert!(BRepTool::surface(&face_copy).unwrap() == plane_of(&face));
}

fn plane_of(face: &Face) -> Handle<dyn GeomSurface> {
    BRepTool::surface_with_location(face).unwrap().0
}
