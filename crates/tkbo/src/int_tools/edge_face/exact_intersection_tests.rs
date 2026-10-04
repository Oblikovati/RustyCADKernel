use super::exact_intersection_parameter;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkmath::gp::{Plane3, Point3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace};

#[test]
fn exact_fallback_keeps_last_intersection_inside_the_requested_range() {
    let edge =
        BRepBuilderAPIMakeEdge::from_points(Point3::new(0., 0., -1.), Point3::new(0., 0., 1.))
            .unwrap()
            .edge()
            .unwrap();
    let face = BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (-2., 2., -2., 2.))
        .unwrap()
        .face()
        .unwrap();
    let curve = BRepAdaptorCurve::with_edge(&edge).unwrap();
    let surface = BRepAdaptorSurface::with_face(&face, true).unwrap();

    assert_eq!(
        exact_intersection_parameter(&curve, &surface, 0., 1.).unwrap(),
        Some(1.)
    );
    assert_eq!(
        exact_intersection_parameter(&curve, &surface, 0., 0.5).unwrap(),
        None
    );
}
