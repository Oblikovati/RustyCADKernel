//! Native line-domain intersection behavior for `ShapeFix_Wire::FixGap2d(..., false)`.
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{self, Face, Wire},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomPlane, GeomRectangularTrimmedSurface, GeomSurface};
use rustycad_tkmath::gp::{Axis2, Axis3, Circle3, Ellipse3, Point3};
use rustycad_tkshhealing::shape_fix::{FixRoot, ShapeFixWire};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;

#[test]
fn intersecting_line_pcurves_extend_only_to_the_shared_parameter() {
    let (face, wire) = crossing_line_wire();
    let tool = ShapeFixWire::new();
    tool.load(&wire).unwrap();
    tool.set_face(&face).unwrap();
    assert!(tool.fix_gap_2d(2, false).unwrap());
    assert!(!tool.context().is_null());
    assert_line_observation(&tool, &face, 1, [0.0, 0.8, 0.0, 0.0, 0.8, 0.0]);
    assert_line_observation(
        &tool,
        &face,
        2,
        [-0.282842712474619, 2.0_f64.sqrt(), 0.8, 0.0, 2.0, 1.2],
    );
}

#[test]
fn parallel_line_pcurves_repair_their_nearest_endpoints() {
    let first = edge_between(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0));
    let second = edge_between(Point3::new(1.1, 0.2, 0.0), Point3::new(2.1, 0.2, 0.0));
    let (face, wire) = wire_with_disconnected_edges(&first, &second);
    let tool = ShapeFixWire::new();
    tool.load(&wire).unwrap();
    tool.set_face(&face).unwrap();
    assert!(tool.fix_gap_2d(2, false).unwrap());
    assert_line_observation(&tool, &face, 1, [0.0, 1.05, 0.0, 0.0, 1.05, 0.0]);
    assert_line_observation(&tool, &face, 2, [-0.05, 1.0, 1.05, 0.2, 2.1, 0.2]);
}

#[test]
fn reversed_first_line_uses_the_oriented_range_end() {
    let mut first = edge_between(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0));
    first.reverse();
    let second = edge_between(Point3::new(1.0, 0.2, 0.0), Point3::new(2.0, 1.2, 0.0));
    let (face, wire) = wire_with_disconnected_edges(&first, &second);
    let tool = ShapeFixWire::new();
    tool.load(&wire).unwrap();
    tool.set_face(&face).unwrap();
    assert!(tool.fix_gap_2d(2, false).unwrap());
    assert_line_observation(&tool, &face, 1, [0.8, 1.0, 0.8, 0.0, 1.0, 0.0]);
    assert_line_observation(
        &tool,
        &face,
        2,
        [-0.282842712474619, 2.0_f64.sqrt(), 0.8, 0.0, 2.0, 1.2],
    );
}

#[test]
fn finite_surface_keeps_a_nearest_hit_when_boundary_lies_beyond_gap_segment() {
    let first = edge_between(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0));
    let second = edge_between(Point3::new(1.1, 0.2, 0.0), Point3::new(2.1, 0.2, 0.0));
    let surface: Handle<dyn GeomSurface> = Handle::new(
        GeomRectangularTrimmedSurface::new(
            &Handle::new(GeomPlane::new(Axis3::default())).cast(),
            (0.0, 1.0),
            (0.0, 1.0),
            (true, true),
        )
        .unwrap(),
    )
    .cast();
    let (face, wire) = wire_with_surface(&first, &second, &surface);
    let tool = ShapeFixWire::new();
    tool.load(&wire).unwrap();
    tool.set_face(&face).unwrap();
    assert!(tool.fix_gap_2d(2, false).unwrap());
    assert_line_observation(&tool, &face, 1, [0.0, 1.0, 0.0, 0.0, 1.0, 0.0]);
    assert_line_observation(&tool, &face, 2, [-0.05, 1.0, 1.05, 0.2, 2.1, 0.2]);
}

#[test]
fn one_edge_circular_pcurve_expands_both_parameter_ends() {
    let (face, wire) = one_circular_arc_wire();
    let tool = ShapeFixWire::new();
    tool.load(&wire).unwrap();
    tool.set_face(&face).unwrap();
    assert!(tool.fix_gap_2d(1, false).unwrap());
    let result_wire = tool.wire().unwrap();
    let result_shape = result_wire.into_shape();
    let result_iter = rustycad_tkbrep::topods::ShapeIterator::new(&result_shape, false, false);
    let result_edge = result_iter.value().unwrap();
    let result_edge = topods::edge(result_edge).unwrap();
    let result_curve = BRepTool::curve_on_face(&result_edge, &face).unwrap();
    assert!((result_curve.first + 2.641592653589793).abs() < 1e-12);
    assert!((result_curve.last - 3.641592653589793).abs() < 1e-12);
    assert_line_observation(
        &tool,
        &face,
        1,
        [
            -2.641592653589793,
            3.641592653589793,
            -0.8775825618903728,
            -0.479425538604203,
            -0.8775825618903728,
            -0.4794255386042027,
        ],
    );
}

#[test]
fn one_edge_elliptical_pcurve_expands_both_parameter_ends() {
    let (face, wire) = one_elliptical_arc_wire();
    let tool = ShapeFixWire::new();
    tool.load(&wire).unwrap();
    tool.set_face(&face).unwrap();
    assert!(tool.fix_gap_2d(1, false).unwrap());
    assert_line_observation(
        &tool,
        &face,
        1,
        [
            -2.641592653589793,
            3.641592653589793,
            -2.6327476856711183,
            -0.479425538604203,
            -2.6327476856711183,
            -0.4794255386042027,
        ],
    );
}

fn crossing_line_wire() -> (Face, Wire) {
    let first = edge_between(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0));
    let second = edge_between(Point3::new(1.0, 0.2, 0.0), Point3::new(2.0, 1.2, 0.0));
    wire_with_disconnected_edges(&first, &second)
}

fn one_circular_arc_wire() -> (Face, Wire) {
    let circle = Circle3::new(Axis2::default(), 1.0).unwrap();
    let edge = BRepBuilderAPIMakeEdge::from_circ_params(circle, 0.0, 1.0)
        .unwrap()
        .edge()
        .unwrap();
    one_arc_wire(edge)
}

fn one_elliptical_arc_wire() -> (Face, Wire) {
    let ellipse = Ellipse3::new(Axis2::default(), 3.0, 1.0).unwrap();
    let edge = BRepBuilderAPIMakeEdge::from_elips_params(ellipse, 0.0, 1.0)
        .unwrap()
        .edge()
        .unwrap();
    one_arc_wire(edge)
}

fn one_arc_wire(edge: rustycad_tkbrep::topods::Edge) -> (Face, Wire) {
    let builder = BRepBuilder::default();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    builder.add(&mut wire, &edge.into_shape()).unwrap();
    let mut face = Face::default();
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    builder
        .make_face_on_surface(&mut face, &plane, 1e-7)
        .unwrap();
    builder.add(&mut face, &wire).unwrap();
    (face, wire)
}

fn edge_between(first: Point3, last: Point3) -> rustycad_tkbrep::topods::Edge {
    BRepBuilderAPIMakeEdge::from_points(first, last)
        .unwrap()
        .edge()
        .unwrap()
}

fn wire_with_disconnected_edges(
    first: &rustycad_tkbrep::topods::Edge,
    second: &rustycad_tkbrep::topods::Edge,
) -> (Face, Wire) {
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    wire_with_surface(first, second, &surface)
}

fn wire_with_surface(
    first: &rustycad_tkbrep::topods::Edge,
    second: &rustycad_tkbrep::topods::Edge,
    surface: &Handle<dyn GeomSurface>,
) -> (Face, Wire) {
    let builder = BRepBuilder::default();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    builder.add(&mut wire, &first.clone().into_shape()).unwrap();
    builder
        .add(&mut wire, &second.clone().into_shape())
        .unwrap();
    let mut face = Face::default();
    builder
        .make_face_on_surface(&mut face, surface, 1e-7)
        .unwrap();
    builder.add(&mut face, &wire).unwrap();
    (face, wire)
}

fn assert_line_observation(tool: &ShapeFixWire, face: &Face, index: i32, expected: [f64; 6]) {
    let edge = topods::edge(&tool.wire_data().unwrap().edge(index).unwrap().into_shape()).unwrap();
    let curve = BRepTool::curve_on_face(&edge, face).unwrap();
    let start = curve.pcurve.value(curve.first).unwrap();
    let end = curve.pcurve.value(curve.last).unwrap();
    let observed = [
        curve.first,
        curve.last,
        start.x(),
        start.y(),
        end.x(),
        end.y(),
    ];
    for (actual, expected) in observed.into_iter().zip(expected) {
        assert!(
            (actual - expected).abs() < 1e-12,
            "line p-curve observation {actual} differs from native value {expected}"
        );
    }
}
