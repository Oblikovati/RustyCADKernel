//! Replays the pinned ShapeAnalysis_WireVertex native oracle.
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::{Point3, Xyz};
use rustycad_tkshhealing::shape_analysis::ShapeAnalysisWireVertex;
use rustycad_tkshhealing::shape_extend::ShapeExtendWireData;
use rustycad_tktopalgo::brep_lib::BRepLibMakeEdge;
const ORACLE: &str = include_str!("../../../tools/oracle/tkshhealing_wire_vertex.expected");

fn pair(points: [[f64; 3]; 4]) -> Handle<ShapeExtendWireData> {
    let wire = Handle::new(ShapeExtendWireData::new());
    for segment in points.as_chunks::<2>().0 {
        let point = |p: [f64; 3]| Point3::new(p[0], p[1], p[2]);
        let edge = BRepLibMakeEdge::from_points(point(segment[0]), point(segment[1])).unwrap();
        wire.add_edge(&edge.edge().unwrap(), 0).unwrap();
    }
    wire
}
fn straight_pair() -> Handle<ShapeExtendWireData> {
    pair([[0., 0., 0.], [1., 0., 0.], [1., 0., 0.], [2., 0., 0.]])
}
fn assert_row(label: &str, tool: &ShapeAnalysisWireVertex) {
    let line = ORACLE
        .lines()
        .find(|line| line.split_whitespace().next() == Some(label))
        .unwrap();
    let expected: Vec<f64> = line
        .split_whitespace()
        .skip(1)
        .map(|x| x.parse().unwrap())
        .collect();
    let (status, position, previous, following) = tool.data(1).unwrap();
    let actual = [
        f64::from(tool.is_done()),
        tool.precision(),
        f64::from(tool.nb_edges()),
        f64::from(status),
        position.x(),
        position.y(),
        position.z(),
        previous,
        following,
    ];
    assert_eq!(actual.as_slice(), expected, "{label}");
    assert_eq!(tool.status(1).unwrap(), status);
    assert_eq!(tool.position(1).unwrap(), position);
    assert_eq!(tool.u_previous(1).unwrap(), previous);
    assert_eq!(tool.u_following(1).unwrap(), following);
}
#[test]
fn geometric_statuses_match_native() {
    let cases = [
        (
            "same-coordinates",
            [[0., 0., 0.], [1., 0., 0.], [1., 0., 0.], [2., 0., 0.]],
            1e-7,
        ),
        (
            "close",
            [[0., 0., 0.], [1., 0., 0.], [1.0001, 0., 0.], [2., 0., 0.]],
            1e-3,
        ),
        (
            "start",
            [[0., 0., 0.], [2., 0., 0.], [1.5, 0., 0.], [3., 0., 0.]],
            1e-7,
        ),
        (
            "end",
            [[0., 0., 0.], [1., 0., 0.], [1., -0.5, 0.], [1., 1., 0.]],
            1e-7,
        ),
        (
            "disjoined",
            [[0., 0., 0.], [1., 0., 0.], [3., 1., 0.], [4., 1., 0.]],
            1e-7,
        ),
    ];
    for (label, points, precision) in cases {
        let mut tool = ShapeAnalysisWireVertex::new();
        tool.init_wire_data(&pair(points), 0.75);
        tool.set_precision(precision);
        tool.analyze().unwrap();
        assert_row(label, &tool);
    }
}
#[test]
fn initialization_copy_and_missing_curve_preserve_native_state() {
    let mut tool = ShapeAnalysisWireVertex::new();
    let wire = straight_pair();
    tool.init_wire_data(&wire, 0.75);
    assert_row("ignored-init-precision", &tool);
    tool.set_inters(1, Xyz::new(3., 4., 5.), 7., 8.).unwrap();
    tool.set_start(1, Xyz::new(6., 7., 8.), 9.).unwrap();
    assert_row("start-slot", &tool);
    let mut copied = tool.clone();
    copied.set_end(1, Xyz::new(1., 2., 3.), 10.).unwrap();
    copied.set_precision(0.2);
    assert_row("shared-records", &tool);
    BRepBuilder::default()
        .update_edge_curve(&wire.edge(1).unwrap(), &Handle::default(), 0.)
        .unwrap();
    tool.analyze().unwrap();
    assert_row("missing-curve-retains", &tool);
    tool.init_wire_data(&Handle::new(ShapeExtendWireData::new()), 1.);
    assert_row("empty-init-retains", &tool);
    assert!(tool.wire_data() == wire);
}
#[test]
fn status_setters_and_native_iteration_criteria() {
    let mut tool = ShapeAnalysisWireVertex::new();
    tool.load_wire_data(&straight_pair());
    tool.set_same_vertex(1).unwrap();
    tool.set_same_coords(2).unwrap();
    let observed = [
        tool.next_criter(1, 0).unwrap(),
        tool.next_status(0, 0).unwrap(),
        tool.next_criter(0, 0).unwrap(),
        tool.next_criter(2, 0).unwrap(),
        tool.next_criter(3, 0).unwrap(),
        tool.next_criter(4, 0).unwrap(),
        tool.next_criter(-1, 0).unwrap(),
        tool.next_criter(99, 0).unwrap(),
    ];
    let expected: Vec<i32> = ORACLE
        .lines()
        .find(|x| x.starts_with("criteria "))
        .unwrap()
        .split_whitespace()
        .skip(1)
        .map(|x| x.parse().unwrap())
        .collect();
    assert_eq!(observed.as_slice(), expected);
    tool.set_close(1).unwrap();
    assert_eq!(tool.next_status(2, 0).unwrap(), 1);
    tool.set_disjoined(2).unwrap();
    assert_eq!(tool.next_criter(-1, 0).unwrap(), 2);
    assert_eq!(tool.next_status(2, 1).unwrap(), 0);
    assert!(tool.status(0).is_err());
    assert!(tool.set_close(3).is_err());
}
#[test]
fn unloaded_and_wire_initializers_preserve_precision() {
    let mut tool = ShapeAnalysisWireVertex::new();
    tool.analyze().unwrap();
    assert!(!tool.is_done());
    assert_eq!(tool.next_status(0, 0).unwrap(), 0);
    assert_eq!(tool.next_criter(0, 0).unwrap(), 0);
    assert_eq!(tool.next_status(0, -1).unwrap(), 0);
    assert_eq!(tool.next_criter(0, -1).unwrap(), 0);
    let wire = straight_pair().wire().unwrap();
    tool.set_precision(0.25);
    tool.init(&wire, 0.75).unwrap();
    assert_eq!(tool.precision(), 0.25);
    assert_eq!(tool.nb_edges(), 2);
    tool.analyze().unwrap();
    assert!(tool.is_done());
    tool.load(&wire).unwrap();
    assert!(!tool.is_done());
    assert_eq!(tool.precision(), 0.25);
}
