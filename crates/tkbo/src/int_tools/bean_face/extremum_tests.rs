use super::compute;
use crate::int_tools::{BeanFaceIntersector, Context};
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::{Axis2, Circle3, Direction3, Plane3, Point3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace};

#[test]
fn parallel_extremum_bisects_to_near_face_interval() {
    let edge = edge_across_trimmed_face();
    let face = trimmed_plane();
    let mut bean = initialize(&edge, &face);
    bean.set_surface_parameters(0.4, 0.6, -1.0, 1.0);
    bean.range_manager.set_boundaries(0.0, 1.0, 0);

    compute(&mut bean).unwrap();

    assert_eq!(bean.range_manager.length(), 5);
    assert_ranges(
        &bean,
        &[
            (0, 0.0, 0.0),
            (2, 0.0, 0.5),
            (0, 0.5, 0.5),
            (2, 0.5, 1.0),
            (0, 1.0, 1.0),
        ],
    );
    assert_eq!(bean.minimal_square_distance(), 0.0);
}

#[test]
fn nonparallel_extrema_seed_both_circle_plane_crossings() {
    let edge = circle_crossing_plane();
    let face = wide_plane();
    let mut bean = initialize_with_bounds(&edge, &face, (0.0, std::f64::consts::TAU));
    bean.range_manager
        .set_boundaries(0.0, std::f64::consts::TAU, 0);

    compute(&mut bean).unwrap();

    assert_eq!(bean.range_manager.length(), 9);
    assert_eq!(bean.range_manager.flag(2).unwrap(), 2);
    assert_eq!(bean.range_manager.flag(4).unwrap(), 2);
    assert_eq!(bean.range_manager.flag(6).unwrap(), 2);
    assert_eq!(bean.range_manager.flag(8).unwrap(), 2);
    assert_eq!(bean.minimal_square_distance(), 0.0);
    assert_ranges(
        &bean,
        &[
            (0, 0.0, 0.1001672),
            (2, 0.1001672, 0.1001674),
            (0, 0.1001674, 0.1001674),
            (2, 0.1001674, 0.1001676),
            (0, 0.1001676, 3.0414250),
            (2, 3.0414250, 3.0414252),
            (0, 3.0414252, 3.0414252),
            (2, 3.0414252, 3.0414254),
            (0, 3.0414254, std::f64::consts::TAU),
        ],
    );
}

fn circle_crossing_plane() -> Edge {
    let axis = Axis2::with_x_direction(
        Point3::new(0., 0., 0.1),
        Direction3::new(0., 1., 0.).unwrap(),
        Direction3::new(1., 0., 0.).unwrap(),
    )
    .unwrap();
    let circle = Circle3::new(axis, 1.).unwrap();
    BRepBuilderAPIMakeEdge::from_circ_params(circle, 0., std::f64::consts::TAU)
        .unwrap()
        .edge()
        .unwrap()
}

fn wide_plane() -> Face {
    BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (-2., 2., -2., 2.))
        .unwrap()
        .face()
        .unwrap()
}

fn assert_ranges(bean: &BeanFaceIntersector, expected: &[(i32, f64, f64)]) {
    for (index, (flag, first, last)) in expected.iter().enumerate() {
        let native_index = index as i32 + 1;
        let actual = bean.range_manager.range(native_index).unwrap();
        assert_eq!(bean.range_manager.flag(native_index).unwrap(), *flag);
        assert!((actual.first() - first).abs() < 1e-6);
        assert!((actual.last() - last).abs() < 1e-6);
    }
}

fn edge_across_trimmed_face() -> Edge {
    BRepBuilderAPIMakeEdge::from_points(Point3::new(0., 0., 0.), Point3::new(1., 0., 0.))
        .unwrap()
        .edge()
        .unwrap()
}

fn trimmed_plane() -> Face {
    BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (0.4, 0.6, -1., 1.))
        .unwrap()
        .face()
        .unwrap()
}

fn initialize(edge: &Edge, face: &Face) -> BeanFaceIntersector {
    initialize_with_bounds(edge, face, (0., 1.))
}

fn initialize_with_bounds(edge: &Edge, face: &Face, range: (f64, f64)) -> BeanFaceIntersector {
    let curve = BRepAdaptorCurve::with_edge(edge).unwrap();
    let surface = BRepAdaptorSurface::with_face(face, false).unwrap();
    let mut bean = BeanFaceIntersector::from_adaptors(&curve, &surface, 1e-7, 1e-7).unwrap();
    bean.set_bean_parameters(range.0, range.1);
    bean.set_surface_parameters(-2., 2., -2., 2.);
    bean.set_context(&Handle::new(Context::default()));
    bean
}
