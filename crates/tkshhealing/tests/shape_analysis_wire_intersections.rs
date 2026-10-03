//! Native pair intersections, overlap segments, spatial errors and output preservation.
mod support;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Edge, Face},
};
use rustycad_tkernel::{ncollection::Sequence, standard::Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::{
    geom::{GeomCurve, GeomLine, GeomPlane, GeomSurface},
    top_abs::Orientation,
};
use rustycad_tkgeomalgo::int_res2d::IntersectionPoint;
use rustycad_tkmath::gp::{Axis1, Axis2d, Axis3, Direction2, Direction3, Point2, Point3};
use rustycad_tkshhealing::{
    shape_analysis::ShapeAnalysisWire,
    shape_extend::{ShapeExtendStatus, ShapeExtendWireData},
};
const EXPECTED: &str =
    include_str!("../../../tools/oracle/tkshhealing_wire_intersections.expected");
const CASES: [&str; 11] = [
    "crossing",
    "endpoint",
    "one_endpoint",
    "overlap",
    "reversed",
    "spatial_error",
    "tolerated",
    "missing_pcurve",
    "zero_range",
    "missing_vertices",
    "unloaded",
];

#[test]
fn paired_edge_intersections_match_native_geometry_and_filtering() {
    for mode in 0..7 {
        inspect(mode);
    }
}

#[test]
fn paired_edge_early_exit_and_exception_output_timing_matches_native() {
    for mode in 7..11 {
        inspect(mode);
    }
}

fn inspect(mode: usize) {
    let analyzer = pair_analyzer(mode);
    let observed = pair_observation(&analyzer);
    let name = CASES[mode];
    let row = EXPECTED
        .lines()
        .find(|line| line.starts_with(&format!("{name} ")))
        .unwrap();
    let expected: Vec<f64> = row
        .split_whitespace()
        .skip(1)
        .map(|value| value.parse().unwrap())
        .collect();
    assert_eq!(observed, expected, "native pair case {name}");
    if mode != 9 {
        assert_eq!(
            analyzer.check_intersecting_edges(1, 0).unwrap(),
            expected[1] != 0.
        );
    }
}

fn pair_observation(analyzer: &ShapeAnalysisWire) -> Vec<f64> {
    let mut uv = Sequence::default();
    let mut xyz = Sequence::default();
    let mut errors = Sequence::default();
    uv.append(IntersectionPoint::default());
    xyz.append(Point3::new(77., 88., 99.));
    errors.append(66.);
    let result = analyzer.check_intersecting_edges_into(1, 0, &mut uv, &mut xyz, &mut errors);
    let (exception, found) = match result {
        Ok(found) => (0., found),
        Err(error) => {
            assert_eq!(error.exception_type(), "Standard_NullObject");
            (1., false)
        }
    };
    let mut observed = vec![
        exception,
        f64::from(found),
        f64::from(analyzer.last_check_status(ShapeExtendStatus::Done1)),
        f64::from(analyzer.last_check_status(ShapeExtendStatus::Fail3)),
        f64::from(uv.length()),
        f64::from(xyz.length()),
        f64::from(errors.length()),
    ];
    for index in 1..=uv.length() as usize {
        let point = uv.value(index).unwrap();
        let spatial = xyz.value(index).unwrap();
        observed.extend([
            point.param_on_first(),
            point.param_on_second(),
            spatial.x(),
            spatial.y(),
            spatial.z(),
            *errors.value(index).unwrap(),
        ]);
    }
    observed
}

fn pair_analyzer(mode: usize) -> ShapeAnalysisWire {
    let face = planar_face();
    let wire = Handle::new(ShapeExtendWireData::new());
    if mode != 10 {
        wire.add_edge(
            &edge_on(
                &face,
                Point2::new(-1., 0.),
                Point2::new(1., 0.),
                mode,
                false,
            ),
            0,
        )
        .unwrap();
        let (first, last) = second_endpoints(mode);
        wire.add_edge(&edge_on(&face, first, last, mode, true), 0)
            .unwrap();
    }
    ShapeAnalysisWire::from_wire_data(&wire, &face, 1e-7).unwrap()
}

fn second_endpoints(mode: usize) -> (Point2, Point2) {
    match mode {
        1 => (Point2::new(1., 0.), Point2::new(1., 2.)),
        2 => (Point2::new(0., 0.), Point2::new(0., 2.)),
        3 => (Point2::new(-0.5, 0.), Point2::new(0.5, 0.)),
        _ => (Point2::new(0., -1.), Point2::new(0., 1.)),
    }
}

fn planar_face() -> Face {
    let mut face = Face::default();
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    face
}

fn edge_on(face: &Face, first: Point2, last: Point2, mode: usize, second: bool) -> Edge {
    let z = if mode == 5 && second { 2. } else { 0. };
    let mut edge = endpoint_edge(first, last, z, mode);
    let direction = Direction2::new(last.x() - first.x(), last.y() - first.y()).unwrap();
    if mode != 7 || !second {
        let curve: Handle<dyn Geom2dCurve> =
            Handle::new(Geom2dLine::new(Axis2d::new(first, direction))).cast();
        BRepBuilder::default()
            .update_edge_pcurve_on_face(&edge, &curve, face, 1e-7)
            .unwrap();
        let span = if mode == 8 && second {
            0.
        } else {
            first.distance(last)
        };
        BRepBuilder::default()
            .range_on_face(&edge, face, 0., span)
            .unwrap();
    }
    if mode == 5 && second {
        attach_spatial_curve(&edge, first, last, direction, z);
    }
    if mode == 4 && second {
        edge.set_orientation(Orientation::Reversed);
    }
    edge
}

fn endpoint_edge(first: Point2, last: Point2, z: f64, mode: usize) -> Edge {
    let builder = BRepBuilder::default();
    if mode == 9 {
        let mut edge = Edge::default();
        builder.make_edge(&mut edge).unwrap();
        return edge;
    }
    let first = support::vertex_at(Point3::new(first.x(), first.y(), z));
    let last = support::vertex_at(Point3::new(last.x(), last.y(), z));
    if mode == 6 {
        builder.update_vertex_tolerance(&first, 100.).unwrap();
        builder.update_vertex_tolerance(&last, 100.).unwrap();
    }
    support::edge(&first, &last)
}

fn attach_spatial_curve(edge: &Edge, first: Point2, last: Point2, direction: Direction2, z: f64) {
    let axis = Axis1::new(
        Point3::new(first.x(), first.y(), z),
        Direction3::new(direction.x(), direction.y(), 0.).unwrap(),
    );
    let curve: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(axis)).cast();
    BRepBuilder::default()
        .update_edge_curve(edge, &curve, 1e-7)
        .unwrap();
    BRepBuilder::default()
        .range(edge, 0., first.distance(last), true)
        .unwrap();
}
