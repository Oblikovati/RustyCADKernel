//! Native notch selection, projection ties, and failure output lifetime.
mod support;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Edge, Face},
};
use rustycad_tkernel::{ncollection::Array1, standard::Handle};
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomCylindricalSurface, GeomSurface};
use rustycad_tkmath::gp::{Axis2d, Axis3, Direction2, Point2};
use rustycad_tkshhealing::{
    shape_analysis::ShapeAnalysisWire,
    shape_extend::{ShapeExtendStatus, ShapeExtendWireData},
};
const EXPECTED: &str = include_str!("../../../tools/oracle/tkshhealing_wire_notched.expected");

#[test]
fn notched_edges_match_pinned_native_geometry_and_outputs() {
    for (mode, row) in EXPECTED.lines().enumerate() {
        let analyzer = fixture_analyzer(mode);
        let (mut short_index, mut parameter) = (71, 83.);
        let result = analyzer
            .check_notched_edges(0, &mut short_index, &mut parameter, 1e-5)
            .unwrap();
        let mut observed = vec![f64::from(result), f64::from(short_index), parameter];
        for flag in [
            ShapeExtendStatus::Done,
            ShapeExtendStatus::Fail1,
            ShapeExtendStatus::Fail2,
            ShapeExtendStatus::Fail3,
        ] {
            observed.push(f64::from(analyzer.last_check_status(flag)));
        }
        let expected: Vec<f64> = row
            .split_whitespace()
            .skip(1)
            .map(|value| value.parse().unwrap())
            .collect();
        assert_eq!(observed, expected, "native notch case {row}");
    }
}

fn fixture_analyzer(mode: usize) -> ShapeAnalysisWire {
    let face = cylindrical_face();
    let wire = Handle::new(ShapeExtendWireData::new());
    if mode != 12 {
        let mut first = fixture_edge(&face, 0., 1., 2., mode, true);
        let length = if mode == 2 {
            3.
        } else if mode == 1 {
            2.
        } else {
            1.
        };
        let mut last = fixture_edge(&face, 2., -1., length, mode, false);
        if mode == 3 {
            first = Edge::unchecked(fixture_edge(&face, 2., -1., 2., mode, true).reversed());
            last = Edge::unchecked(fixture_edge(&face, 1., 1., 1., mode, false).reversed());
        }
        wire.add_edge(&first, 0).unwrap();
        wire.add_edge(&last, 0).unwrap();
    }
    ShapeAnalysisWire::from_wire_data(&wire, &face, 1e-7).unwrap()
}

fn cylindrical_face() -> Face {
    let mut face = Face::default();
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomCylindricalSurface::new(Axis3::default(), 1.).unwrap()).cast();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    face
}

fn fixture_edge(
    face: &Face,
    start: f64,
    direction: f64,
    length: f64,
    mode: usize,
    first: bool,
) -> Edge {
    let builder = BRepBuilder::default();
    let edge = if mode == 5 && first {
        let mut edge = Edge::default();
        builder.make_edge(&mut edge).unwrap();
        edge
    } else {
        let vertex = support::vertex(if mode == 6 && !first { 3. } else { 0. }, 0.);
        support::edge(&vertex, &vertex)
    };
    if mode != 7 || !first {
        let curve = fixture_curve(start, direction, mode, first);
        builder
            .update_edge_pcurve_on_face(&edge, &curve, face, 1e-7)
            .unwrap();
        builder.range_on_face(&edge, face, 0., length).unwrap();
    }
    if mode == 4 && first {
        builder.degenerated(&edge, true).unwrap();
    }
    edge
}

fn fixture_curve(start: f64, direction: f64, mode: usize, first: bool) -> Handle<dyn Geom2dCurve> {
    if mode >= 10 && !first {
        let poles = [
            Point2::new(2., 0.),
            Point2::new(if mode == 10 { 2. } else { 1.5 }, 0.),
            Point2::new(1.5, if mode == 10 { 0. } else { 2. }),
            Point2::new(1., 0.),
        ];
        return Handle::new(Geom2dBezierCurve::new(&Array1::from_shared(&poles, 1)).unwrap())
            .cast();
    }
    let origin = Point2::new(start, if mode == 9 && !first { 1. } else { 0. });
    let axis = if mode == 8 && !first {
        Direction2::new(0., 1.).unwrap()
    } else {
        Direction2::new(direction, 0.).unwrap()
    };
    Handle::new(Geom2dLine::new(Axis2d::new(origin, axis))).cast()
}
