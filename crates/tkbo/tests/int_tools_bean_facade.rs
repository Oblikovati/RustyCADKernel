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

fn analytic_edge(mode: i32) -> Edge {
    use rustycad_tkmath::gp::{Axis2, Circle3};
    if mode >= 7 {
        return conic_edge(mode);
    }
    if (2..4).contains(&mode) || mode == 6 {
        let x = if mode == 2 { 1. } else { 2. };
        return BRepBuilderAPIMakeEdge::from_points(
            Point3::new(x, 0., -1.),
            Point3::new(x, 0., 1.),
        )
        .unwrap()
        .edge()
        .unwrap();
    }
    let mut axis = Axis2::default();
    if mode < 2 {
        axis.set_location(Point3::new(0., 0., f64::from(mode)));
    }
    let radius = if mode < 2 {
        0.5
    } else if mode == 4 {
        1.
    } else {
        2.
    };
    BRepBuilderAPIMakeEdge::from_circ_params(Circle3::new(axis, radius).unwrap(), 0., 1.)
        .unwrap()
        .edge()
        .unwrap()
}

fn analytic_face(mode: i32) -> Face {
    use rustycad_tkmath::gp::{Axis3, Cylinder3, Sphere3};
    if !(2..7).contains(&mode) {
        return source_face();
    }
    let bounds = (0., std::f64::consts::TAU, -1., 1.);
    let maker = if mode == 6 {
        BRepBuilderAPIMakeFace::from_sphere_bounds(
            Sphere3::new(Axis3::default(), 1.).unwrap(),
            bounds,
        )
    } else {
        BRepBuilderAPIMakeFace::from_cylinder_bounds(
            Cylinder3::new(Axis3::default(), 1.).unwrap(),
            bounds,
        )
    };
    maker.unwrap().face().unwrap()
}

#[test]
fn analytic_facade_matches_native_coincidence_and_separation() {
    let mut rows = String::new();
    for mode in 0..10 {
        let mut solver =
            BeanFaceIntersector::from_shapes(&analytic_edge(mode), &analytic_face(mode)).unwrap();
        solver.set_bean_parameters(0., 1.);
        for pass in 0..2 {
            solver.perform().unwrap();
            snapshot(&solver, mode, 0, pass, &mut rows);
        }
    }
    assert_eq!(rows.lines().count(), 20);
    assert_eq!(
        rows,
        include_str!("../../../tools/oracle/tkbo_bean_analytic_facade.expected")
    );
}

fn conic_edge(mode: i32) -> Edge {
    use rustycad_tkmath::gp::{Axis2, Ellipse3, Hyperbola3, Parabola3};
    let maker = match mode {
        7 => BRepBuilderAPIMakeEdge::from_elips_params(
            Ellipse3::new(Axis2::default(), 2., 1.).unwrap(),
            0.,
            1.,
        ),
        8 => BRepBuilderAPIMakeEdge::from_hypr_params(
            Hyperbola3::new(Axis2::default(), 2., 1.).unwrap(),
            0.,
            1.,
        ),
        _ => BRepBuilderAPIMakeEdge::from_parab_params(
            Parabola3::new(Axis2::default(), 1.).unwrap(),
            0.,
            1.,
        ),
    };
    maker.unwrap().edge().unwrap()
}

fn distance_rows(
    solver: &BeanFaceIntersector,
    face: &Face,
    mode: i32,
    bounded: i32,
    rows: &mut String,
) -> usize {
    let mut fallbacks = 0;
    for parameter in [0., 0.25, 1., 2.] {
        let distance = solver.distance(parameter).unwrap();
        let (mut u, mut v) = (77., 88.);
        let with_uv = solver.distance_uv(parameter, &mut u, &mut v).unwrap();
        writeln!(
            rows,
            "{mode} {bounded} {parameter:.12} {distance:.12} {with_uv:.12} {u:.12} {v:.12}"
        )
        .unwrap();
        let cached = solver.context().get().unwrap().proj_ps(face).unwrap();
        let projector = cached.read();
        fallbacks += usize::from(!projector.is_done() || projector.nb_points().unwrap() == 0);
    }
    fallbacks
}

#[test]
fn bean_distance_matches_native_projection_boundary_fallback_and_uv_clamping() {
    let mut rows = String::new();
    let mut fallbacks = 0;
    for mode in 0..11 {
        let (edge, face) = distance_shapes(mode);
        let mut solver = BeanFaceIntersector::from_shapes(&edge, &face).unwrap();
        fallbacks += distance_rows(&solver, &face, mode, 0, &mut rows);
        solver.set_surface_parameters(0.1, 0.2, 0.1, 0.2);
        fallbacks += distance_rows(&solver, &face, mode, 1, &mut rows);
    }
    assert_eq!(fallbacks, 18);
    assert_eq!(rows.lines().count(), 88);
    assert_eq!(
        rows,
        include_str!("../../../tools/oracle/tkbo_bean_distance.expected")
    );
}

fn distance_shapes(mode: i32) -> (Edge, Face) {
    use rustycad_tkmath::gp::{Axis3, Sphere3};
    if mode != 10 {
        return (analytic_edge(mode), analytic_face(mode));
    }
    let edge =
        BRepBuilderAPIMakeEdge::from_points(Point3::new(0., 0., -1.), Point3::new(0., 0., 1.))
            .unwrap()
            .edge()
            .unwrap();
    let bounds = (
        0.,
        std::f64::consts::TAU,
        -std::f64::consts::FRAC_PI_2,
        std::f64::consts::FRAC_PI_2,
    );
    let face = BRepBuilderAPIMakeFace::from_sphere_bounds(
        Sphere3::new(Axis3::default(), 1.).unwrap(),
        bounds,
    )
    .unwrap()
    .face()
    .unwrap();
    (edge, face)
}
