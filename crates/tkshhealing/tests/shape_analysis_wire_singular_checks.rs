//! Branch regressions compared with the pinned OCCT CheckDegenerated implementation.
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Builder, Edge, Face, Vertex},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::{
    geom::{GeomCircle, GeomConicalSurface, GeomCurve, GeomSurface},
    top_abs::Orientation,
};
use rustycad_tkmath::gp::{Axis2, Axis2d, Axis3, Direction2, Direction3, Point2, Point3};
use rustycad_tkshhealing::{
    shape_analysis::ShapeAnalysisWire,
    shape_extend::{ShapeExtendStatus, ShapeExtendWireData},
};

const EXPECTED: &str =
    include_str!("../../../tools/oracle/tkshhealing_wire_singular_checks.expected");

#[test]
fn single_apex_needs_no_existing_pcurve() {
    let wire = Handle::new(ShapeExtendWireData::new());
    wire.add_edge(&edge_at(Point3::default(), Point3::default()), 0)
        .unwrap();
    inspect("apex_without_pcurve", &wire);
}

#[test]
fn closed_curve_midpoint_vetoes_existing_degenerate_classification() {
    let edge = edge_at(Point3::default(), Point3::default());
    let frame = Axis2::new(
        Point3::new(-1., 0., 0.),
        Direction3::new(0., 0., 1.).unwrap(),
    )
    .unwrap();
    let curve: Handle<dyn GeomCurve> = Handle::new(GeomCircle::new(frame, 1.).unwrap()).cast();
    let builder = BRepBuilder::default();
    builder.update_edge_curve(&edge, &curve, 1e-7).unwrap();
    builder
        .range(&edge, 0., std::f64::consts::TAU, true)
        .unwrap();
    let wire = Handle::new(ShapeExtendWireData::new());
    wire.add_edge(&edge, 0).unwrap();
    inspect("closed_curve_midpoint_veto", &wire);
}

#[test]
fn disconnected_edges_do_not_acquire_a_missing_singular_edge() {
    let wire = Handle::new(ShapeExtendWireData::new());
    wire.add_edge(
        &edge_at(Point3::new(2., 0., 0.), Point3::new(1., 0., 0.)),
        0,
    )
    .unwrap();
    wire.add_edge(&edge_at(Point3::default(), Point3::new(1., 0., 0.)), 0)
        .unwrap();
    inspect("disconnected_singular_vertex", &wire);
}

fn inspect(name: &str, wire: &Handle<ShapeExtendWireData>) {
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomConicalSurface::new(Axis3::default(), 0.5, 0.).unwrap()).cast();
    let mut face = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    if wire.nb_edges() == 2 {
        attach_apex_pcurves(wire, &face);
    }
    let analyzer = ShapeAnalysisWire::from_wire_data(wire, &face, 1e-7).unwrap();
    let (found, _, _) = analyzer.check_degenerated_at(wire.nb_edges()).unwrap();
    let statuses = [
        ShapeExtendStatus::Done1,
        ShapeExtendStatus::Done2,
        ShapeExtendStatus::Fail1,
        ShapeExtendStatus::Fail2,
    ];
    let flags = statuses.map(|status| u8::from(analyzer.last_check_status(status)));
    let row = format!(
        "{name} {} {} {} {} {}",
        u8::from(found),
        flags[0],
        flags[1],
        flags[2],
        flags[3]
    );
    assert_eq!(
        EXPECTED
            .lines()
            .find(|line| line.starts_with(&format!("{name} "))),
        Some(row.as_str())
    );
}

fn edge_at(first: Point3, last: Point3) -> Edge {
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    builder.make_edge(&mut edge).unwrap();
    let mut shape = edge.clone().into_shape();
    for (point, orientation) in [(first, Orientation::Forward), (last, Orientation::Reversed)] {
        let mut vertex = Vertex::default();
        builder.make_vertex_at(&mut vertex, point, 1e-7).unwrap();
        Builder
            .add(&mut shape, &vertex.oriented(orientation))
            .unwrap();
    }
    edge
}

fn attach_apex_pcurves(wire: &Handle<ShapeExtendWireData>, face: &Face) {
    let builder = BRepBuilder::default();
    for index in 1..=2 {
        let frame = Axis2d::new(
            Point2::new(f64::from(index) * 2., 0.),
            Direction2::new(1., 0.).unwrap(),
        );
        let curve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(frame)).cast();
        let edge = wire.edge(index).unwrap();
        builder
            .update_edge_pcurve_on_face(&edge, &curve, face, 1e-7)
            .unwrap();
        builder.range_on_face(&edge, face, 0., 1.).unwrap();
    }
}
