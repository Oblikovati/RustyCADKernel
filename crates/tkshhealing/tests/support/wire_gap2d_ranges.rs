use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{self, Face, Wire},
};
use rustycad_tkernel::{ncollection::Array1, standard::Handle};
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCurve, Geom2dEllipse, Geom2dLine,
    Geom2dOffsetCurve, Geom2dParabola, Geom2dTrimmedCurve,
};
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkmath::gp::{
    Axis2, Axis2d, Axis3, Axis22d, Circle3, Direction2, Direction3, Ellipse3, Point2, Point3,
};
use rustycad_tkshhealing::shape_fix::ShapeFixWire;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;

pub(super) fn set_bspline_line_pcurves(
    face: &Face,
    first_edge: &rustycad_tkbrep::topods::Edge,
    second_edge: &rustycad_tkbrep::topods::Edge,
) {
    let builder = BRepBuilder::default();
    let first = bspline_line(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0));
    let second = bspline_line(Point2::new(0.3, -0.2), Point2::new(1.3, 0.8));
    set_pcurve_range(&builder, first_edge, face, &first, 1.0);
    set_pcurve_range(&builder, second_edge, face, &second, 1.0);
}

pub(super) fn set_tangent_pcurves(
    face: &Face,
    first_edge: &rustycad_tkbrep::topods::Edge,
    second_edge: &rustycad_tkbrep::topods::Edge,
) {
    let builder = BRepBuilder::default();
    let parabola_poles = [
        Point2::new(0.0, 0.0),
        Point2::new(0.5, 0.5),
        Point2::new(1.0, 0.0),
    ];
    let parabola: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dBezierCurve::new(&Array1::from_shared(&parabola_poles, 1)).unwrap())
            .cast();
    let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::new(
        Point2::new(0.0, 0.25),
        Direction2::new(1.0, 0.0).unwrap(),
    )))
    .cast();
    set_pcurve_range(&builder, first_edge, face, &parabola, 1.0);
    set_pcurve_range(&builder, second_edge, face, &line, 1.0);
}

pub(super) fn set_tangent_bspline_pcurves(
    face: &Face,
    first_edge: &rustycad_tkbrep::topods::Edge,
    second_edge: &rustycad_tkbrep::topods::Edge,
) {
    let builder = BRepBuilder::default();
    let poles = [
        Point2::new(0.0, 0.0),
        Point2::new(0.5, 0.5),
        Point2::new(1.0, 0.0),
    ];
    let knots = [0.0, 1.0];
    let multiplicities = [3, 3];
    let spline: Handle<dyn Geom2dCurve> = Handle::new(
        Geom2dBSplineCurve::new(
            &Array1::from_shared(&poles, 1),
            &Array1::from_shared(&knots, 1),
            &Array1::from_shared(&multiplicities, 1),
            2,
            false,
        )
        .unwrap(),
    )
    .cast();
    let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::new(
        Point2::new(0.0, 0.25),
        Direction2::new(1.0, 0.0).unwrap(),
    )))
    .cast();
    set_pcurve_range(&builder, first_edge, face, &spline, 1.0);
    set_pcurve_range(&builder, second_edge, face, &line, 1.0);
}

pub(super) fn set_parabola_line_pcurves(
    face: &Face,
    first_edge: &rustycad_tkbrep::topods::Edge,
    second_edge: &rustycad_tkbrep::topods::Edge,
) {
    let builder = BRepBuilder::default();
    let parabola: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dParabola::new(Axis22d::default(), 0.25).unwrap()).cast();
    let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::new(
        Point2::new(0.2, 0.0),
        Direction2::new(1.0, 0.0).unwrap(),
    )))
    .cast();
    set_pcurve_range(&builder, first_edge, face, &parabola, 0.2);
    builder.range_on_face(first_edge, face, 0.1, 0.2).unwrap();
    set_pcurve_range(&builder, second_edge, face, &line, 1.0);
}

pub(super) fn set_intersecting_ellipse_pcurves(
    face: &Face,
    first_edge: &rustycad_tkbrep::topods::Edge,
    second_edge: &rustycad_tkbrep::topods::Edge,
) {
    let builder = BRepBuilder::default();
    let x = Direction2::new(1.0, 0.0).unwrap();
    let first_axis = Axis22d::new(Point2::new(0.0, 0.0), x, true).unwrap();
    let second_axis = Axis22d::new(Point2::new(5.0, 0.0), x, true).unwrap();
    let first: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dEllipse::new(first_axis, 3.0, 1.0).unwrap()).cast();
    let second: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dEllipse::new(second_axis, 3.0, 1.0).unwrap()).cast();
    set_pcurve_range(&builder, first_edge, face, &first, 0.1);
    set_pcurve_range(
        &builder,
        second_edge,
        face,
        &second,
        std::f64::consts::PI + 0.1,
    );
    builder.range_on_face(first_edge, face, 0.0, 0.1).unwrap();
    builder
        .range_on_face(
            second_edge,
            face,
            std::f64::consts::PI,
            std::f64::consts::PI + 0.1,
        )
        .unwrap();
}

pub(super) fn bspline_line(start: Point2, end: Point2) -> Handle<dyn Geom2dCurve> {
    let poles = [start, end];
    let knots = [0.0, 1.0];
    let multiplicities = [2, 2];
    Handle::new(
        Geom2dBSplineCurve::new(
            &Array1::from_shared(&poles, 1),
            &Array1::from_shared(&knots, 1),
            &Array1::from_shared(&multiplicities, 1),
            1,
            false,
        )
        .unwrap(),
    )
    .cast()
}

pub(super) fn assert_bspline_pcurve(
    tool: &ShapeFixWire,
    face: &Face,
    index: i32,
    expected: [f64; 2],
) {
    let edge = topods::edge(&tool.wire_data().unwrap().edge(index).unwrap().into_shape()).unwrap();
    let curve = BRepTool::curve_on_face(&edge, face).unwrap();
    assert!(!curve.pcurve.cast::<Geom2dBSplineCurve>().is_null());
    assert!((curve.first - expected[0]).abs() < 1e-12);
    assert!((curve.last - expected[1]).abs() < 1e-12);
}

pub(super) fn set_bezier_line_pcurves(
    face: &Face,
    first_edge: &rustycad_tkbrep::topods::Edge,
    second_edge: &rustycad_tkbrep::topods::Edge,
) {
    let builder = BRepBuilder::default();
    let first = bezier_line(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0));
    let second = bezier_line(Point2::new(0.3, -0.2), Point2::new(1.3, 0.8));
    set_pcurve_range(&builder, first_edge, face, &first, 1.0);
    set_pcurve_range(&builder, second_edge, face, &second, 1.0);
}

pub(super) fn bezier_line(start: Point2, end: Point2) -> Handle<dyn Geom2dCurve> {
    let poles = [start, end];
    Handle::new(Geom2dBezierCurve::new(&Array1::from_shared(&poles, 1)).unwrap()).cast()
}

pub(super) fn assert_bezier_pcurve(
    tool: &ShapeFixWire,
    face: &Face,
    index: i32,
    expected: [f64; 2],
) {
    let edge = topods::edge(&tool.wire_data().unwrap().edge(index).unwrap().into_shape()).unwrap();
    let curve = BRepTool::curve_on_face(&edge, face).unwrap();
    assert!(!curve.pcurve.cast::<Geom2dBezierCurve>().is_null());
    assert!((curve.first - expected[0]).abs() < 1e-12);
    assert!((curve.last - expected[1]).abs() < 1e-12);
}

pub(super) fn set_offset_line_pcurves(
    face: &Face,
    first_edge: &rustycad_tkbrep::topods::Edge,
    second_edge: &rustycad_tkbrep::topods::Edge,
) {
    let builder = BRepBuilder::default();
    let first = offset_line(Point2::new(0.0, 0.2), Direction2::new(1.0, 0.0).unwrap());
    let second = offset_line(Point2::new(0.3, -0.2), Direction2::new(1.0, 1.0).unwrap());
    set_pcurve_range(&builder, first_edge, face, &first, 1.0);
    set_pcurve_range(&builder, second_edge, face, &second, 2.0_f64.sqrt());
}

pub(super) fn set_trimmed_offset_line_pcurves(
    face: &Face,
    first_edge: &rustycad_tkbrep::topods::Edge,
    second_edge: &rustycad_tkbrep::topods::Edge,
) {
    let builder = BRepBuilder::default();
    let first = trimmed_offset_line(
        Point2::new(0.0, 0.2),
        Direction2::new(1.0, 0.0).unwrap(),
        1.0,
    );
    let second = trimmed_offset_line(
        Point2::new(0.3, -0.2),
        Direction2::new(1.0, 1.0).unwrap(),
        2.0_f64.sqrt(),
    );
    set_pcurve_range(&builder, first_edge, face, &first, 1.0);
    set_pcurve_range(&builder, second_edge, face, &second, 2.0_f64.sqrt());
}

fn trimmed_offset_line(
    origin: Point2,
    direction: Direction2,
    last: f64,
) -> Handle<dyn Geom2dCurve> {
    let offset = offset_line(origin, direction);
    Handle::new(Geom2dTrimmedCurve::new(&offset, 0.0, last, true, true).unwrap()).cast()
}

pub(super) fn offset_line(origin: Point2, direction: Direction2) -> Handle<dyn Geom2dCurve> {
    let basis: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dLine::new(Axis2d::new(origin, direction))).cast();
    Handle::new(Geom2dOffsetCurve::new(&basis, 0.2, false).unwrap()).cast()
}

pub(super) fn assert_offset_pcurve(
    tool: &ShapeFixWire,
    face: &Face,
    index: i32,
    expected: [f64; 2],
) {
    let edge = topods::edge(&tool.wire_data().unwrap().edge(index).unwrap().into_shape()).unwrap();
    let curve = BRepTool::curve_on_face(&edge, face).unwrap();
    assert!(!curve.pcurve.cast::<Geom2dOffsetCurve>().is_null());
    assert!((curve.first - expected[0]).abs() < 1e-12);
    assert!((curve.last - expected[1]).abs() < 1e-12);
}

pub(super) fn assert_trimmed_offset_pcurve(
    tool: &ShapeFixWire,
    face: &Face,
    index: i32,
    expected: [f64; 2],
) {
    let edge = topods::edge(&tool.wire_data().unwrap().edge(index).unwrap().into_shape()).unwrap();
    let curve = BRepTool::curve_on_face(&edge, face).unwrap();
    let trimmed = curve.pcurve.cast::<Geom2dTrimmedCurve>();
    assert!(!trimmed.is_null());
    assert!(
        !trimmed
            .get()
            .unwrap()
            .basis_curve()
            .cast::<Geom2dOffsetCurve>()
            .is_null()
    );
    assert!((curve.first - expected[0]).abs() < 1e-12);
    assert!((curve.last - expected[1]).abs() < 1e-12);
}

pub(super) fn set_trimmed_line_pcurves(
    face: &Face,
    first_edge: &rustycad_tkbrep::topods::Edge,
    second_edge: &rustycad_tkbrep::topods::Edge,
) {
    let builder = BRepBuilder::default();
    let first = trimmed_line(
        Point2::new(0.0, 0.0),
        Direction2::new(1.0, 0.0).unwrap(),
        1.0,
    );
    let second = trimmed_line(
        Point2::new(1.0, 0.2),
        Direction2::new(1.0, 1.0).unwrap(),
        2.0_f64.sqrt(),
    );
    set_pcurve_range(&builder, first_edge, face, &first, 1.0);
    set_pcurve_range(&builder, second_edge, face, &second, 2.0_f64.sqrt());
}

pub(super) fn set_pcurve_range(
    builder: &BRepBuilder,
    edge: &rustycad_tkbrep::topods::Edge,
    face: &Face,
    curve: &Handle<dyn Geom2dCurve>,
    last: f64,
) {
    builder
        .update_edge_pcurve_on_face(edge, curve, face, 1e-7)
        .unwrap();
    builder.range_on_face(edge, face, 0.0, last).unwrap();
}

pub(super) fn trimmed_line(
    origin: Point2,
    direction: Direction2,
    last: f64,
) -> Handle<dyn Geom2dCurve> {
    let line: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dLine::new(Axis2d::new(origin, direction))).cast();
    Handle::new(Geom2dTrimmedCurve::new(&line, 0.0, last, true, true).unwrap()).cast()
}

pub(super) fn assert_trimmed_pcurve(
    tool: &ShapeFixWire,
    face: &Face,
    index: i32,
    expected: [f64; 2],
) {
    let edge = topods::edge(&tool.wire_data().unwrap().edge(index).unwrap().into_shape()).unwrap();
    let curve = BRepTool::curve_on_face(&edge, face).unwrap();
    assert!(!curve.pcurve.cast::<Geom2dTrimmedCurve>().is_null());
    assert!((curve.first - expected[0]).abs() < 1e-12);
    assert!((curve.last - expected[1]).abs() < 1e-12);
}

pub(super) fn crossing_line_wire() -> (Face, Wire) {
    let first = edge_between(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0));
    let second = edge_between(Point3::new(1.0, 0.2, 0.0), Point3::new(2.0, 1.2, 0.0));
    wire_with_disconnected_edges(&first, &second)
}

pub(super) fn one_circular_arc_wire() -> (Face, Wire) {
    let circle = Circle3::new(Axis2::default(), 1.0).unwrap();
    let edge = BRepBuilderAPIMakeEdge::from_circ_params(circle, 0.0, 1.0)
        .unwrap()
        .edge()
        .unwrap();
    one_arc_wire(edge)
}

pub(super) fn one_elliptical_arc_wire() -> (Face, Wire) {
    let ellipse = Ellipse3::new(Axis2::default(), 3.0, 1.0).unwrap();
    let edge = BRepBuilderAPIMakeEdge::from_elips_params(ellipse, 0.0, 1.0)
        .unwrap()
        .edge()
        .unwrap();
    one_arc_wire(edge)
}

pub(super) fn one_arc_wire(edge: rustycad_tkbrep::topods::Edge) -> (Face, Wire) {
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

pub(super) fn intersecting_circle_wire() -> (Face, Wire) {
    let normal = Direction3::new(0.0, 0.0, 1.0).unwrap();
    let first = circle_arc(Point3::default(), normal, 0.0, 0.1);
    let second = circle_arc(
        Point3::new(1.8, 0.0, 0.0),
        normal,
        std::f64::consts::PI,
        std::f64::consts::PI + 0.1,
    );
    wire_with_disconnected_edges(&first, &second)
}

pub(super) fn circle_arc(
    center: Point3,
    normal: Direction3,
    first: f64,
    last: f64,
) -> rustycad_tkbrep::topods::Edge {
    let circle = Circle3::new(Axis2::new(center, normal).unwrap(), 1.0).unwrap();
    BRepBuilderAPIMakeEdge::from_circ_params(circle, first, last)
        .unwrap()
        .edge()
        .unwrap()
}

pub(super) fn edge_between(first: Point3, last: Point3) -> rustycad_tkbrep::topods::Edge {
    BRepBuilderAPIMakeEdge::from_points(first, last)
        .unwrap()
        .edge()
        .unwrap()
}

pub(super) fn wire_with_disconnected_edges(
    first: &rustycad_tkbrep::topods::Edge,
    second: &rustycad_tkbrep::topods::Edge,
) -> (Face, Wire) {
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    wire_with_surface(first, second, &surface)
}

pub(super) fn wire_with_surface(
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

pub(super) fn assert_line_observation(
    tool: &ShapeFixWire,
    face: &Face,
    index: i32,
    expected: [f64; 6],
) {
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
