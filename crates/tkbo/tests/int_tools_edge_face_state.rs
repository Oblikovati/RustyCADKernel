//! Native replay for retained completion/results across success and invalid input.
use rustycad_tkbo::int_tools::EdgeFace;
use rustycad_tkbrep::{brep::BRepBuilder, topods::Edge};
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::{Plane3, Point3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace};

fn append_state(solver: &EdgeFace, stage: i32, values: &mut Vec<f64>) {
    values.extend([
        f64::from(stage),
        f64::from(solver.is_done()),
        f64::from(solver.error_status()),
        solver.common_parts().length() as f64,
        f64::from(solver.context().is_null()),
        solver.fuzzy_value(),
        f64::from(solver.is_coincidence_checked_quickly()),
    ]);
}

fn crossing_edge() -> Edge {
    BRepBuilderAPIMakeEdge::from_points(Point3::new(0., 0., -1.), Point3::new(0., 0., 1.))
        .unwrap()
        .edge()
        .unwrap()
}

fn prepare_crossing(solver: &mut EdgeFace) -> Edge {
    let edge = crossing_edge();
    let face = BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (-5., 5., -5., 5.))
        .unwrap()
        .face()
        .unwrap();
    solver.set_edge(&edge);
    solver.set_face(&face);
    solver.set_range_bounds(0., 1.);
    edge
}

fn error_stages(solver: &mut EdgeFace, edge: &Edge, values: &mut Vec<f64>) {
    let builder = BRepBuilder::default();
    builder.degenerated(edge, true).unwrap();
    solver.perform().unwrap();
    append_state(solver, 3, values);
    let mut empty = Edge::default();
    builder.make_edge(&mut empty).unwrap();
    solver.set_edge(&empty);
    solver.perform().unwrap();
    append_state(solver, 4, values);
    solver.set_fuzzy_value(-1.);
    append_state(solver, 5, values);
}

fn quick_recovery(solver: &mut EdgeFace, values: &mut Vec<f64>) {
    solver.set_fuzzy_value(1e-4);
    solver.set_context(&Handle::default());
    solver.use_quick_coincidence_check(true);
    let contained =
        BRepBuilderAPIMakeEdge::from_points(Point3::new(-1., 0., 0.), Point3::new(1., 0., 0.))
            .unwrap()
            .edge()
            .unwrap();
    solver.set_edge(&contained);
    solver.perform().unwrap();
    append_state(solver, 6, values);
}

#[test]
fn native_state_retains_results_and_done_across_error_checks() {
    let mut solver = EdgeFace::new();
    let mut actual = Vec::new();
    append_state(&solver, 0, &mut actual);
    let edge = prepare_crossing(&mut solver);
    for stage in 1..=2 {
        solver.perform().unwrap();
        append_state(&solver, stage, &mut actual);
    }
    error_stages(&mut solver, &edge, &mut actual);
    quick_recovery(&mut solver, &mut actual);
    let expected: Vec<f64> = include_str!("../../../tools/oracle/tkbo_edge_face_state.expected")
        .split_whitespace()
        .map(|word| word.parse().unwrap())
        .collect();
    assert_eq!(actual, expected);
}
