use super::compute;
use crate::int_tools::BeanFaceIntersector;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkmath::gp::Point3;
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeOneAxis, BRepPrimAPIMakeSphere};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;

#[test]
fn exact_sphere_crossing_marks_both_native_point_ranges() {
    let edge = crossing_edge();
    let face = sphere_face();
    let mut bean = initialized_bean(&edge, &face);
    bean.range_manager.set_boundaries(-2.0, 2.0, 0);

    compute(&mut bean).unwrap();

    assert_eq!(bean.range_manager.length(), 5);
    assert_eq!(bean.range_manager.flag(2).unwrap(), 2);
    assert_eq!(bean.range_manager.flag(4).unwrap(), 2);
    assert_range(&bean, 1, (-2.0, 0.999_999_7));
    assert_range(&bean, 2, (0.999_999_7, 1.0));
    assert_range(&bean, 3, (1.0, 1.0));
    assert_range(&bean, 4, (1.0, 1.000_000_3));
    assert_range(&bean, 5, (1.000_000_3, 2.0));
    assert_eq!(bean.minimum_square_distance, 0.0);
}

fn crossing_edge() -> Edge {
    BRepBuilderAPIMakeEdge::from_points(Point3::new(-2.0, 0.0, 0.0), Point3::new(2.0, 0.0, 0.0))
        .unwrap()
        .edge()
        .unwrap()
}

fn sphere_face() -> Face {
    BRepPrimAPIMakeSphere::new(1.0).unwrap().face().unwrap()
}

fn initialized_bean(edge: &Edge, face: &Face) -> BeanFaceIntersector {
    let curve = BRepAdaptorCurve::with_edge(edge).unwrap();
    let surface = BRepAdaptorSurface::with_face(face, true).unwrap();
    let mut bean = BeanFaceIntersector::from_adaptors(&curve, &surface, 1e-7, 1e-7).unwrap();
    bean.set_bean_parameters(-2.0, 2.0);
    bean.set_surface_parameters(
        0.0,
        2.0 * std::f64::consts::PI,
        -0.5 * std::f64::consts::PI,
        0.5 * std::f64::consts::PI,
    );
    bean
}

fn assert_range(bean: &BeanFaceIntersector, index: i32, expected: (f64, f64)) {
    let range = bean.range_manager.range(index).unwrap();
    assert!((range.first() - expected.0).abs() < 1e-12);
    assert!((range.last() - expected.1).abs() < 1e-12);
}

#[test]
fn periodic_correction_retains_native_outside_trim_parameter() {
    let mut bean = initialized_bean(&crossing_edge(), &sphere_face());
    bean.set_surface_parameters(0.0, 1.0, -0.5, 0.5);
    // OCCT accepts periodic correction without clamping to a narrower trim.
    assert_eq!(
        super::corrected_parameters(&bean, 2.0, 0.0).unwrap(),
        Some((2.0, 0.0))
    );
}
