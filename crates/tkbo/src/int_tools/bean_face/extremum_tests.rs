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
    assert_native_ranges(&bean, "range ");
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
    assert_native_ranges(&bean, "circle_range ");
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

fn assert_native_ranges(bean: &BeanFaceIntersector, prefix: &str) {
    let fixture = include_str!("../../../../../tools/oracle/tkbo_bean_using_extremum.expected");
    let rows: Vec<_> = fixture
        .lines()
        .filter(|line| line.starts_with(prefix))
        .collect();
    assert_eq!(rows.len(), bean.range_manager.length() as usize);
    for row in rows {
        assert_native_range(bean, row);
    }
}

fn assert_native_range(bean: &BeanFaceIntersector, row: &str) {
    let columns: Vec<_> = row.split_whitespace().collect();
    let index = columns[1].parse::<i32>().unwrap();
    let flag = columns[2].parse::<i32>().unwrap();
    let first = columns[3].parse::<f64>().unwrap();
    let last = columns[4].parse::<f64>().unwrap();
    let actual = bean.range_manager.range(index).unwrap();
    assert_eq!(bean.range_manager.flag(index).unwrap(), flag);
    assert!(
        (actual.first() - first).abs() < 1e-12,
        "first interval bound: {row}"
    );
    assert!(
        (actual.last() - last).abs() < 1e-12,
        "last interval bound: {row}"
    );
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
