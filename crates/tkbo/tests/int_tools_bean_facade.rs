//! Stateful line/plane BeanFace facade compared against pinned OCCT.
use rustycad_tkbo::int_tools::{BeanFaceIntersector, Range};
use rustycad_tkbrep::{
    brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface},
    topods::{Edge, Face},
};
use rustycad_tkernel::{ncollection::Sequence, standard::Handle};
use rustycad_tkmath::gp::{Plane3, Point3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace};
use std::fmt::Write;

fn source_edge(mode: i32) -> Edge {
    let (first, last) = match mode {
        0 => ([0.25, 0.5, -1.], [0.25, 0.5, 1.]),
        1 => ([-0.5, 0., 0.], [0.5, 0., 0.]),
        2 => ([-0.5, 0., 2.5e-7], [0.5, 0., 2.5e-7]),
        3 => ([-0.5, 0., 1e-6], [0.5, 0., 1e-6]),
        4 => ([2., 0., -1.], [2., 0., 1.]),
        _ => ([-1., 0., -1.], [1., 0., 1.]),
    };
    BRepBuilderAPIMakeEdge::from_points(
        Point3::new(first[0], first[1], first[2]),
        Point3::new(last[0], last[1], last[2]),
    )
    .unwrap()
    .edge()
    .unwrap()
}

fn source_face() -> Face {
    BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (-1., 1., -1., 1.))
        .unwrap()
        .face()
        .unwrap()
}

fn snapshot(
    solver: &BeanFaceIntersector,
    mode: i32,
    constructor: i32,
    stage: i32,
    rows: &mut String,
) {
    let mut copied = Sequence::default();
    copied.append(Range::new(99., 100.));
    solver.result_into(&mut copied);
    assert_eq!(copied.length(), solver.result().length());
    write!(
        rows,
        "{mode} {constructor} {stage} {} {} {} {}",
        i32::from(solver.is_done()),
        i32::from(solver.context().is_null()),
        i32::from(solver.minimal_square_distance() == f64::MAX),
        copied.length()
    )
    .unwrap();
    for range in copied.iter() {
        write!(rows, " {:.12} {:.12}", range.first(), range.last()).unwrap();
    }
    rows.push('\n');
}

fn run_case(mode: i32, constructor: i32, rows: &mut String) {
    let edge = source_edge(mode);
    let face = source_face();
    let curve = BRepAdaptorCurve::with_edge(&edge).unwrap();
    let surface = BRepAdaptorSurface::with_face(&face, true).unwrap();
    let mut solver = if constructor == 0 {
        BeanFaceIntersector::from_shapes(&edge, &face).unwrap()
    } else {
        BeanFaceIntersector::from_adaptors(&curve, &surface, 1e-7, 1e-7).unwrap()
    };
    solver.set_bean_parameters(0., 2.);
    solver.set_surface_parameters(-1., 1., -1., 1.);
    snapshot(&solver, mode, constructor, 0, rows);
    for stage in 1..=2 {
        solver.perform().unwrap();
        snapshot(&solver, mode, constructor, stage, rows);
    }
    if constructor == 0 {
        solver.init_shapes(&edge, &face).unwrap();
    } else {
        solver.init_adaptors(&curve, &surface, 1e-7, 1e-7).unwrap();
    }
    snapshot(&solver, mode, constructor, 3, rows);
    solver.perform().unwrap();
    snapshot(&solver, mode, constructor, 4, rows);
    solver.set_context(&Handle::default());
    solver.perform().unwrap();
    snapshot(&solver, mode, constructor, 5, rows);
}

#[test]
fn bean_facade_matches_native_constructor_tolerances_and_reuse() {
    let mut rows = String::new();
    for mode in 0..6 {
        for constructor in 0..2 {
            run_case(mode, constructor, &mut rows);
        }
    }
    assert_eq!(rows.lines().count(), 72);
    assert_eq!(
        rows,
        include_str!("../../../tools/oracle/tkbo_bean_facade.expected")
    );
}

#[test]
fn default_bean_state_retains_native_sentinels() {
    let solver = BeanFaceIntersector::new();
    assert!(!solver.is_done());
    assert!(solver.context().is_null());
    assert_eq!(solver.result().length(), 0);
    assert_eq!(solver.minimal_square_distance(), f64::MAX);
}
