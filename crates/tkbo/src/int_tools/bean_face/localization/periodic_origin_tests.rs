//! Per-sample periodic origins are pinned to native `ElCLib::InPeriod` behavior.
use crate::int_tools::{BeanFaceIntersector, Range};
use rustycad_tkbrep::{
    brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface},
    topods::Face,
};
use rustycad_tkmath::gp::{Axis2, Axis3, Circle3, Torus3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace};
use std::f64::consts::TAU;

#[test]
fn periodic_parameters_use_each_sample_range_as_the_native_origin() {
    let edge = periodic_edge();
    let face = periodic_face();
    let curve = BRepAdaptorCurve::with_edge(&edge).unwrap();
    let surface = BRepAdaptorSurface::with_face(&face, true).unwrap();
    let mut bean = BeanFaceIntersector::from_adaptors(&curve, &surface, 1e-7, 1e-7).unwrap();
    bean.set_bean_parameters(0.0, TAU);
    bean.set_surface_parameters(0.0, TAU, 0.0, TAU);

    let actual = super::super::periodic_parameters(
        &bean,
        [
            Range::new(1.0, 2.0),
            Range::new(2.0, 3.0),
            Range::new(4.0, 5.0),
        ],
        0.1,
        0.2,
        0.3,
    )
    .unwrap();
    assert_native_periodic_parameters(actual);
}

fn periodic_edge() -> rustycad_tkbrep::topods::Edge {
    let circle = Circle3::new(Axis2::default(), 1.0).unwrap();
    BRepBuilderAPIMakeEdge::from_circ_params(circle, 0.0, TAU)
        .unwrap()
        .edge()
        .unwrap()
}

fn periodic_face() -> Face {
    BRepBuilderAPIMakeFace::from_torus_bounds(
        Torus3::new(Axis3::default(), 5.0, 2.0).unwrap(),
        (0.0, TAU, 0.0, TAU),
    )
    .unwrap()
    .face()
    .unwrap()
}

fn assert_native_periodic_parameters(actual: (f64, f64, f64)) {
    let expected =
        include_str!("../../../../../../tools/oracle/tkbo_bean_periodic_origins.expected");
    let fields: Vec<_> = expected.split_whitespace().collect();
    assert_eq!(&fields[..3], &["1", "1", "1"]);
    for (value, token) in [actual.0, actual.1, actual.2].into_iter().zip(&fields[3..]) {
        assert!((value - token.parse::<f64>().unwrap()).abs() <= 1e-12);
    }
}
