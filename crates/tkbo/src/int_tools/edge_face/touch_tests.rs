use super::super::EdgeFace;
use super::check_touch_vertex;
use crate::int_tools::Context;
use rustycad_tkbrep::{
    brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface},
    topods::Face,
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::{Axis2, Axis3, Circle3, Cylinder3, Direction3, Plane3, Point3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace};

#[test]
fn vertex_touch_refinement_uses_extrema_parameter() {
    let edge =
        BRepBuilderAPIMakeEdge::from_points(Point3::new(-2., 1., 0.), Point3::new(2., 1., 0.))
            .unwrap()
            .edge()
            .unwrap();
    let cylinder = Cylinder3::new(Axis3::default(), 1.).unwrap();
    let face: Face = BRepBuilderAPIMakeFace::from_cylinder_bounds(
        cylinder,
        (-std::f64::consts::PI, std::f64::consts::PI, -2., 2.),
    )
    .unwrap()
    .face()
    .unwrap();
    let mut intersector = configured_intersector(&edge, &face);

    let parameter = check_touch_vertex(&mut intersector, 1.999_225_55, 2.000_774_45).unwrap();

    assert_eq!(parameter, Some(2.));
}

#[test]
fn circle_plane_endpoint_touches_keep_native_endpoint_parameters() {
    let circle = Circle3::new(Axis2::default(), 1.).unwrap();
    let edge = BRepBuilderAPIMakeEdge::from_circ(circle)
        .unwrap()
        .edge()
        .unwrap();
    let plane = Plane3::new(Point3::new(1., 0., 0.), Direction3::X).unwrap();
    let face = BRepBuilderAPIMakeFace::from_plane_bounds(plane, (-2., 2., -2., 2.))
        .unwrap()
        .face()
        .unwrap();
    let mut intersector = configured_intersector(&edge, &face);

    assert_eq!(
        check_touch_vertex(&mut intersector, 0., 0.000_774_45).unwrap(),
        None
    );
    assert_eq!(
        check_touch_vertex(&mut intersector, 6.282_410_857, std::f64::consts::TAU).unwrap(),
        None
    );
}

fn configured_intersector(edge: &rustycad_tkbrep::topods::Edge, face: &Face) -> EdgeFace {
    let mut intersector = EdgeFace::new();
    intersector.edge = edge.clone();
    intersector.face = face.clone();
    intersector.context = Handle::new(Context::default());
    intersector.curve = Handle::new(BRepAdaptorCurve::with_edge(edge).unwrap());
    intersector.surface = Handle::new(BRepAdaptorSurface::with_face(face, true).unwrap());
    intersector.criteria = 3e-7;
    intersector
}

#[test]
fn touch_adaptors_preserve_interval_and_distinct_surface_bounds() {
    use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
    let edge =
        BRepBuilderAPIMakeEdge::from_points(Point3::new(0., 0., -1.), Point3::new(0., 0., 1.))
            .unwrap()
            .edge()
            .unwrap();
    let face = BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (-2., 2., -3., 3.))
        .unwrap()
        .face()
        .unwrap();
    let solver = configured_intersector(&edge, &face);
    let (curve, bounded) = super::touch_adaptors(&solver, 0.25, 1.5, true).unwrap();
    assert_eq!(
        (
            curve.first_parameter().unwrap(),
            curve.last_parameter().unwrap()
        ),
        (0.25, 1.5)
    );
    assert_eq!(
        super::surface_bounds(&bounded).unwrap(),
        ((-2., 2.), (-3., 3.))
    );
    let (_, full) = super::touch_adaptors(&solver, 0.25, 1.5, false).unwrap();
    assert!(rustycad_tkernel::precision::is_infinite(
        full.first_u_parameter().unwrap()
    ));
    assert!(rustycad_tkernel::precision::is_infinite(
        full.last_v_parameter().unwrap()
    ));
}
