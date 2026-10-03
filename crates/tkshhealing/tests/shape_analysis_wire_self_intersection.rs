//! Native self-intersection sequence clearing, endpoint filtering and geometry.
mod support;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Edge, Face},
};
use rustycad_tkernel::{
    ncollection::{Array1, Sequence},
    standard::Handle,
};
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCurve};
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkgeomalgo::int_res2d::IntersectionPoint;
use rustycad_tkmath::gp::{Axis3, Point2, Point3};
use rustycad_tkshhealing::{
    shape_analysis::ShapeAnalysisWire,
    shape_extend::{ShapeExtendStatus, ShapeExtendWireData},
};
const EXPECTED: &str =
    include_str!("../../../tools/oracle/tkshhealing_wire_self_intersection.expected");

#[test]
fn self_intersections_match_native_loop_and_vertex_filtering() {
    inspect("loop", true, 1e-7, true, 1., true);
    inspect("tolerated", true, 100., true, 1., true);
}

#[test]
fn self_intersection_early_exits_clear_both_sequences() {
    inspect("no_vertices", false, 1e-7, true, 1., true);
    inspect("no_pcurve", true, 1e-7, false, 1., true);
    inspect("zero_range", false, 1e-7, true, 0., true);
    inspect("unloaded", true, 1e-7, true, 1., false);
}

fn inspect(name: &str, vertices: bool, tolerance: f64, pcurve: bool, last: f64, loaded: bool) {
    let face = planar_face();
    let wire = Handle::new(ShapeExtendWireData::new());
    if loaded {
        wire.add_edge(&loop_edge(&face, vertices, tolerance, pcurve, last), 0)
            .unwrap();
    }
    let analyzer = ShapeAnalysisWire::from_wire_data(&wire, &face, 1e-7).unwrap();
    let observed = intersection_observation(&analyzer);
    let row = EXPECTED
        .lines()
        .find(|line| line.starts_with(&format!("{name} ")))
        .unwrap();
    let expected: Vec<f64> = row
        .split_whitespace()
        .skip(1)
        .map(|value| value.parse().unwrap())
        .collect();
    assert_eq!(observed, expected, "native self-intersection case {name}");
}

fn intersection_observation(analyzer: &ShapeAnalysisWire) -> Vec<f64> {
    let mut uv = Sequence::<IntersectionPoint>::default();
    let mut xyz = Sequence::<Point3>::default();
    uv.append(IntersectionPoint::default());
    xyz.append(Point3::new(77., 88., 99.));
    let found = analyzer
        .check_self_intersecting_edge_into(0, &mut uv, &mut xyz)
        .unwrap();
    let mut observed = vec![
        f64::from(found),
        f64::from(analyzer.last_check_status(ShapeExtendStatus::Done1)),
        f64::from(analyzer.last_check_status(ShapeExtendStatus::Fail1)),
        f64::from(analyzer.last_check_status(ShapeExtendStatus::Fail2)),
        f64::from(uv.length()),
        f64::from(xyz.length()),
    ];
    for index in 1..=uv.length() {
        let point = uv.value(index as usize).unwrap();
        let spatial = xyz.value(index as usize).unwrap();
        observed.extend([
            point.param_on_first(),
            point.param_on_second(),
            spatial.x(),
            spatial.y(),
            spatial.z(),
        ]);
    }
    observed.push(f64::from(analyzer.check_self_intersecting_edge(0).unwrap()));
    observed.push(f64::from(analyzer.check_self_intersection().unwrap()));
    for flag in [
        ShapeExtendStatus::Done1,
        ShapeExtendStatus::Done2,
        ShapeExtendStatus::Done3,
        ShapeExtendStatus::Fail1,
        ShapeExtendStatus::Fail2,
        ShapeExtendStatus::Fail3,
    ] {
        observed.push(f64::from(analyzer.status_self_intersection(flag)));
    }
    observed
}

fn planar_face() -> Face {
    let mut face = Face::default();
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    face
}

fn loop_edge(face: &Face, vertices: bool, tolerance: f64, pcurve: bool, last: f64) -> Edge {
    let edge = fixture_edge(vertices, tolerance);
    if pcurve {
        let poles = [
            Point2::new(0., 0.),
            Point2::new(3., 3.),
            Point2::new(-3., 3.),
            Point2::new(1., 0.),
        ];
        let curve: Handle<dyn Geom2dCurve> =
            Handle::new(Geom2dBezierCurve::new(&Array1::from_shared(&poles, 1)).unwrap()).cast();
        BRepBuilder::default()
            .update_edge_pcurve_on_face(&edge, &curve, face, 1e-7)
            .unwrap();
        BRepBuilder::default()
            .range_on_face(&edge, face, 0., last)
            .unwrap();
    }
    edge
}

fn fixture_edge(vertices: bool, tolerance: f64) -> Edge {
    let builder = BRepBuilder::default();
    if !vertices {
        let mut edge = Edge::default();
        builder.make_edge(&mut edge).unwrap();
        return edge;
    }
    let (first, last) = (support::vertex(0., 0.), support::vertex(1., 0.));
    builder.update_vertex_tolerance(&first, tolerance).unwrap();
    builder.update_vertex_tolerance(&last, tolerance).unwrap();
    support::edge(&first, &last)
}
