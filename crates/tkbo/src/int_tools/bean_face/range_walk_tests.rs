//! Private range-state comparison uses unchanged linked native algorithms.
use super::BeanFaceIntersector;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkmath::gp::{Plane3, Point3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace};
use std::fmt::Write;

fn walk_edge(mode: i32) -> Edge {
    let (first, last) = if mode == 0 {
        (Point3::new(0.25, 0.5, -1.), Point3::new(0.25, 0.5, 1.))
    } else {
        let z = match mode {
            1 => 0.,
            2 => 2.5e-7,
            _ => 1e-6,
        };
        (Point3::new(-0.5, 0., z), Point3::new(0.5, 0., z))
    };
    BRepBuilderAPIMakeEdge::from_points(first, last)
        .unwrap()
        .edge()
        .unwrap()
}

fn walk_fixture_bounds(mode: i32, flags: i32, last: f64) -> BeanFaceIntersector {
    let face = BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (-1., 1., -1., 1.))
        .unwrap()
        .face()
        .unwrap();
    let mut solver = BeanFaceIntersector::from_shapes(&walk_edge(mode), &face).unwrap();
    solver.set_bean_parameters(0., last);
    solver
        .range_manager
        .set_ranges(&[0., last * 0.25, last * 0.5, last * 0.75, last], 0);
    if flags == 1 {
        solver.range_manager.set_flag(2, 1).unwrap();
    }
    if flags == 2 {
        solver.range_manager.set_flag(3, 2).unwrap();
    }
    solver
}

fn append_walk(mode: i32, flags: i32, seed: f64, increasing: bool, rows: &mut String) {
    let mut solver = walk_fixture_bounds(mode, flags, 2.);
    let (mut u, mut v) = (0., 0.);
    solver.distance_uv(seed, &mut u, &mut v).unwrap();
    solver
        .compute_range_from_start_point(increasing, seed, u, v)
        .unwrap();
    write!(
        rows,
        "{mode} {flags} {seed:.12} {} {}",
        i32::from(increasing),
        solver.range_manager.length()
    )
    .unwrap();
    for index in 1..=solver.range_manager.length() {
        let range = solver.range_manager.range(index).unwrap();
        write!(
            rows,
            " {} {:.12} {:.12}",
            solver.range_manager.flag(index).unwrap(),
            range.first(),
            range.last()
        )
        .unwrap();
    }
    rows.push('\n');
}

#[test]
fn native_adaptive_walk_preserves_flags_direction_boundaries_and_empty_seeds() {
    let mut rows = String::new();
    for (mode, flags) in (0..4).flat_map(|mode| (0..3).map(move |flags| (mode, flags))) {
        for (seed, increasing) in [-0.5, 0., 0.5, 1., 1.5, 2., 2.5]
            .into_iter()
            .flat_map(|seed| [false, true].map(|increasing| (seed, increasing)))
        {
            append_walk(mode, flags, seed, increasing, &mut rows);
        }
    }
    assert_eq!(rows.lines().count(), 168);
    assert_eq!(
        rows,
        include_str!("../../../../../tools/oracle/tkbo_bean_range_walk.expected")
    );
}

fn append_coincidence_case(mode: i32, flags: i32, rows: &mut String) {
    let mut solver = walk_fixture_bounds(mode, flags, 1.);
    for pass in 0..2 {
        let result = solver.test_compute_coinside().unwrap();
        write!(
            rows,
            "{mode} {flags} {pass} {} {}",
            i32::from(result),
            solver.range_manager.length()
        )
        .unwrap();
        for index in 1..=solver.range_manager.length() {
            let range = solver.range_manager.range(index).unwrap();
            write!(
                rows,
                " {} {:.12} {:.12}",
                solver.range_manager.flag(index).unwrap(),
                range.first(),
                range.last()
            )
            .unwrap();
        }
        rows.push('\n');
    }
}

#[test]
fn native_coincidence_sampling_preserves_flagged_ranges_and_empty_markers() {
    let mut rows = String::new();
    for mode in 0..4 {
        for flags in 0..3 {
            append_coincidence_case(mode, flags, &mut rows);
        }
    }
    assert_eq!(rows.lines().count(), 24);
    assert_eq!(
        rows,
        include_str!("../../../../../tools/oracle/tkbo_bean_coincidence.expected")
    );
}
