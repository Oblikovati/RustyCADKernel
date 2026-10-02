//! Native parity for the no-candidate `ShapeUpgrade_UnifySameDomain::Build` path.
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Shape, Shell, Wire};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomCylindricalSurface, GeomSurface};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Axis3, Direction2, Plane3, Point2, Point3, Vector2};
use rustycad_tkmath::toploc::Location;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tkshhealing::shape_upgrade::ShapeUpgradeUnifySameDomain;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape,
    BRepBuilderAPIMakeWire, BRepBuilderAPISewing,
};
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;

#[test]
fn basic_box_unification_matches_native_no_merge_result() {
    let input = BRepPrimAPIMakeBox::from_sizes(10.0, 10.0, 10.0)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    let unifier = ShapeUpgradeUnifySameDomain::new(&input, true, true, true);
    unifier.set_angular_tolerance(1e-6);
    unifier.set_linear_tolerance(1e-5);
    unifier.allow_internal_edges(true);
    unifier.initialize(&input, true, true, true);

    unifier.build().unwrap();

    let result = unifier.shape();
    assert_eq!(face_count(&result), 6);
    assert!(
        BRepCheckAnalyzer::new(&result, true, false, false)
            .unwrap()
            .is_valid()
            .unwrap()
    );
}

fn face_count(shape: &Shape) -> usize {
    rustycad_tkbrep::topexp::Explorer::new(
        shape,
        rustycad_tkg3d::top_abs::ShapeEnum::Face,
        rustycad_tkg3d::top_abs::ShapeEnum::Shape,
    )
    .unwrap()
    .count()
}

#[test]
fn internal_open_path_terminates_and_keeps_its_face() {
    let input = shell_with_internal_path(
        &[
            Point3::new(2.0, 2.0, 0.0),
            Point3::new(5.0, 5.0, 0.0),
            Point3::new(8.0, 2.0, 0.0),
        ],
        false,
    );
    let unifier = ShapeUpgradeUnifySameDomain::new(&input, true, true, true);
    unifier.set_angular_tolerance(1e-6);
    unifier.set_linear_tolerance(1e-5);
    unifier.allow_internal_edges(true);

    unifier.build().unwrap();

    let result = unifier.shape();
    assert!(!result.is_null());
    assert!(face_count(&result) >= 1);
}

#[test]
fn closed_internal_wire_keeps_nonnull_result() {
    let input = shell_with_internal_path(
        &[
            Point3::new(5.0, 5.0, 0.0),
            Point3::new(15.0, 5.0, 0.0),
            Point3::new(10.0, 15.0, 0.0),
        ],
        true,
    );
    let unifier = ShapeUpgradeUnifySameDomain::new(&input, true, true, true);
    unifier.set_angular_tolerance(1e-6);
    unifier.set_linear_tolerance(1e-5);
    unifier.allow_internal_edges(true);

    unifier.build().unwrap();

    assert!(!unifier.shape().is_null());
}

fn shell_with_internal_path(points: &[Point3], closed: bool) -> Shape {
    let extent = if closed { 20.0 } else { 10.0 };
    let outer = rectangle_wire(0.0, extent, 0.0, extent);
    let mut face = BRepBuilderAPIMakeFace::from_plane_wire(Plane3::default(), &outer, true)
        .unwrap()
        .face()
        .unwrap();
    let mut internal = BRepBuilderAPIMakeWire::new();
    let edge_count = if closed {
        points.len()
    } else {
        points.len() - 1
    };
    for index in 0..edge_count {
        let edge =
            BRepBuilderAPIMakeEdge::from_points(points[index], points[(index + 1) % points.len()])
                .unwrap()
                .edge()
                .unwrap();
        internal.add_edge(&edge).unwrap();
    }
    let mut internal = internal.wire().unwrap();
    internal.set_orientation(Orientation::Internal);
    BRepBuilder::default().add(&mut face, &internal).unwrap();

    let mut shell = Shell::default();
    BRepBuilder::default().make_shell(&mut shell);
    BRepBuilder::default()
        .add(&mut shell, &face.into_shape())
        .unwrap();
    shell.into_shape()
}

#[test]
fn missing_source_pcurve_does_not_abort_transformation() {
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomCylindricalSurface::new(Axis3::default(), 1.0).unwrap()).cast();
    let left_panel = cylindrical_panel(&surface, 0.0, 0.5, false);
    let right_panel = cylindrical_panel(&surface, 0.5, 1.0, true);
    let mut shell = Shell::default();
    let builder = BRepBuilder::default();
    builder.make_shell(&mut shell);
    builder
        .add(&mut shell, &left_panel.face.clone().into_shape())
        .unwrap();
    builder
        .add(&mut shell, &right_panel.face.clone().into_shape())
        .unwrap();
    let input = shell.into_shape();
    let missing_edge = right_panel.missing_edge.as_ref().unwrap();
    let pcurve = BRepTool::curve_on_face(missing_edge, &right_panel.face).unwrap();
    assert!(!pcurve.is_stored);
    assert_eq!(face_count(&input), 2);

    let unifier = ShapeUpgradeUnifySameDomain::new(&input, false, true, false);
    unifier.allow_internal_edges(true);
    unifier.build().unwrap();

    let result = unifier.shape();
    assert!(!result.is_null());
    assert_eq!(face_count(&result), 2);
}

fn cylindrical_panel(
    surface: &Handle<dyn GeomSurface>,
    u_min: f64,
    u_max: f64,
    omit_bottom_pcurve: bool,
) -> CylinderPanel {
    let builder = BRepBuilder::default();
    let uv_points = [
        Point2::new(u_min, 0.0),
        Point2::new(u_max, 0.0),
        Point2::new(u_max, 1.0),
        Point2::new(u_min, 1.0),
    ];
    let vertices = uv_points.map(|uv| {
        let mut vertex = rustycad_tkbrep::topods::Vertex::default();
        builder
            .make_vertex_at(
                &mut vertex,
                surface.get().unwrap().value(uv.x(), uv.y()).unwrap(),
                precision::confusion(),
            )
            .unwrap();
        vertex
    });
    let mut wire_builder = BRepBuilderAPIMakeWire::new();
    let mut missing = None;
    let mut valid = None;
    for index in 0..4 {
        let next = (index + 1) % 4;
        let edge = BRepBuilderAPIMakeEdge::from_vertices(&vertices[index], &vertices[next])
            .unwrap()
            .edge()
            .unwrap();
        if !(omit_bottom_pcurve && index == 0) {
            let curve: Handle<dyn Geom2dCurve> =
                Handle::new(pcurve_line(uv_points[index], uv_points[next])).cast();
            builder
                .update_edge_pcurve(
                    &edge,
                    &curve,
                    surface,
                    &Location::default(),
                    precision::confusion(),
                    None,
                )
                .unwrap();
            let range = Vector2::between(uv_points[index], uv_points[next]).magnitude();
            builder
                .range_on_surface(&edge, surface, &Location::default(), 0.0, range)
                .unwrap();
            if index == 1 {
                valid = Some(edge.clone());
            }
        } else {
            missing = Some(edge.clone());
        }
        wire_builder.add_edge(&edge).unwrap();
    }
    let wire = wire_builder.wire().unwrap();
    let mut face = rustycad_tkbrep::topods::Face::default();
    builder
        .make_face_on_surface(&mut face, surface, precision::confusion())
        .unwrap();
    builder.add(&mut face, &wire.into_shape()).unwrap();
    CylinderPanel {
        face,
        missing_edge: missing,
        valid_pcurve_edge: valid,
    }
}

struct CylinderPanel {
    face: rustycad_tkbrep::topods::Face,
    missing_edge: Option<rustycad_tkbrep::topods::Edge>,
    valid_pcurve_edge: Option<rustycad_tkbrep::topods::Edge>,
}

fn pcurve_line(first: Point2, last: Point2) -> Geom2dLine {
    Geom2dLine::from_point(
        first,
        Direction2::from_vector(Vector2::between(first, last)).unwrap(),
    )
}

#[test]
fn missing_pcurve_does_not_abort_unrelated_planar_merge() {
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomCylindricalSurface::new(Axis3::default(), 1.0).unwrap()).cast();
    let left = cylindrical_panel(&surface, 0.0, 0.5, false);
    let right = cylindrical_panel(&surface, 0.5, 1.0, true);
    let missing = right.missing_edge.as_ref().unwrap();
    let stored = right.valid_pcurve_edge.as_ref().unwrap();
    assert!(
        !BRepTool::curve_on_face(missing, &right.face)
            .unwrap()
            .is_stored
    );
    assert!(
        BRepTool::curve_on_face(stored, &right.face)
            .unwrap()
            .is_stored
    );

    let planar_pair = sewn_planar_pair();
    let builder = BRepBuilder::default();
    let mut shell = Shell::default();
    builder.make_shell(&mut shell);
    for face in [left.face, right.face] {
        builder.add(&mut shell, &face.into_shape()).unwrap();
    }
    for face in
        rustycad_tkbrep::topexp::Explorer::new(&planar_pair, ShapeEnum::Face, ShapeEnum::Shape)
            .unwrap()
    {
        builder.add(&mut shell, &face).unwrap();
    }
    let input = shell.into_shape();
    assert_eq!(face_count(&input), 4);

    let unifier = ShapeUpgradeUnifySameDomain::new(&input, false, true, false);
    unifier.allow_internal_edges(true);
    unifier.build().unwrap();

    assert_eq!(face_count(&unifier.shape()), 3);
}

fn sewn_planar_pair() -> Shape {
    let first = planar_face(3.0, 4.0);
    let second = planar_face(4.0, 5.0);
    let mut sewing = BRepBuilderAPISewing::default();
    sewing.add(&first).unwrap();
    sewing.add(&second).unwrap();
    sewing.perform(&ProgressRange::default()).unwrap();
    sewing.sewed_shape().clone()
}

fn planar_face(x_min: f64, x_max: f64) -> Shape {
    let wire = rectangle_wire(x_min, x_max, 0.0, 1.0);
    BRepBuilderAPIMakeFace::from_plane_wire(Plane3::default(), &wire, true)
        .unwrap()
        .face()
        .unwrap()
        .into_shape()
}

#[test]
fn adjacent_coplanar_faces_with_internal_wires_merge_to_one_face() {
    let first = face_with_inner_segment(0.0, 5.0, 1.0, 4.0);
    let second = face_with_inner_segment(5.0, 10.0, 6.0, 9.0);
    let mut sewing = BRepBuilderAPISewing::default();
    sewing.add(&first).unwrap();
    sewing.add(&second).unwrap();
    sewing.perform(&ProgressRange::default()).unwrap();
    let input = sewing.sewed_shape();
    assert_eq!(face_count(input), 2);

    let unifier = ShapeUpgradeUnifySameDomain::new(input, true, true, true);
    unifier.set_angular_tolerance(1e-6);
    unifier.set_linear_tolerance(1e-5);
    unifier.allow_internal_edges(true);
    unifier.build().unwrap();

    let result = unifier.shape();
    assert_eq!(face_count(&result), 1);
    assert_eq!(internal_wire_count(&result), 2);
}

fn face_with_inner_segment(x_min: f64, x_max: f64, inner_min: f64, inner_max: f64) -> Shape {
    let outer = rectangle_wire(x_min, x_max, 0.0, 10.0);
    let plane = Plane3::default();
    let mut face = BRepBuilderAPIMakeFace::from_plane_wire(plane, &outer, true)
        .unwrap()
        .face()
        .unwrap();
    let mut inner = BRepBuilderAPIMakeWire::from_edge(
        &BRepBuilderAPIMakeEdge::from_points(
            Point3::new(inner_min, 5.0, 0.0),
            Point3::new(inner_max, 5.0, 0.0),
        )
        .unwrap()
        .edge()
        .unwrap(),
    )
    .unwrap()
    .wire()
    .unwrap();
    inner.set_orientation(Orientation::Internal);
    BRepBuilder::default().add(&mut face, &inner).unwrap();
    face.into_shape()
}

fn rectangle_wire(x_min: f64, x_max: f64, y_min: f64, y_max: f64) -> Wire {
    let points = [
        Point3::new(x_min, y_min, 0.0),
        Point3::new(x_max, y_min, 0.0),
        Point3::new(x_max, y_max, 0.0),
        Point3::new(x_min, y_max, 0.0),
    ];
    let mut wire = BRepBuilderAPIMakeWire::new();
    for index in 0..4 {
        let edge = BRepBuilderAPIMakeEdge::from_points(points[index], points[(index + 1) % 4])
            .unwrap()
            .edge()
            .unwrap();
        wire.add_edge(&edge).unwrap();
    }
    wire.wire().unwrap()
}

fn internal_wire_count(shape: &Shape) -> usize {
    rustycad_tkbrep::topexp::Explorer::new(shape, ShapeEnum::Wire, ShapeEnum::Shape)
        .unwrap()
        .filter(|wire| wire.orientation() == Orientation::Internal)
        .count()
}
