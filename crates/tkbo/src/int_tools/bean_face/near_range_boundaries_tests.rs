use super::compute;
use crate::int_tools::{BeanFaceIntersector, Context};
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::{Plane3, Point3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace};

#[test]
fn near_plane_boundaries_seed_native_overlap_range() {
    let edge = near_plane_edge();
    let face = plane_face();
    let mut bean = initialized_bean(&edge, &face);
    bean.range_manager.set_boundaries(0.0, 1.0, 0);

    compute(&mut bean).unwrap();

    assert_eq!(bean.range_manager.length(), 3);
    assert_eq!(bean.range_manager.flag(1).unwrap(), 0);
    assert_eq!(bean.range_manager.flag(2).unwrap(), 2);
    assert_eq!(bean.range_manager.flag(3).unwrap(), 0);
    assert_range(&bean, 1, (0.0, 0.0));
    assert_range(&bean, 2, (0.0, 1.0));
    assert_range(&bean, 3, (1.0, 1.0));
}

fn near_plane_edge() -> Edge {
    BRepBuilderAPIMakeEdge::from_points(
        Point3::new(0.0, 0.0, 1.0e-7),
        Point3::new(1.0, 0.0, 1.0e-7),
    )
    .unwrap()
    .edge()
    .unwrap()
}

fn plane_face() -> Face {
    BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (-1.0, 2.0, -1.0, 1.0))
        .unwrap()
        .face()
        .unwrap()
}

fn initialized_bean(edge: &Edge, face: &Face) -> BeanFaceIntersector {
    let curve = BRepAdaptorCurve::with_edge(edge).unwrap();
    let surface = BRepAdaptorSurface::with_face(face, false).unwrap();
    let mut bean = BeanFaceIntersector::from_adaptors(&curve, &surface, 1.0e-7, 1.0e-7).unwrap();
    bean.set_bean_parameters(0.0, 1.0);
    bean.set_surface_parameters(-1.0, 2.0, -1.0, 1.0);
    bean.set_context(&Handle::new(Context::default()));
    bean
}

fn assert_range(bean: &BeanFaceIntersector, index: i32, expected: (f64, f64)) {
    let range = bean.range_manager.range(index).unwrap();
    assert_eq!((range.first(), range.last()), expected);
}
