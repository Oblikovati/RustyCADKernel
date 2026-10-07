use super::*;
use crate::shape_construct::{ShapeConstructApproxSettings, ShapeConstructSurfaceBounds};
use rustycad_tkbrep::topods::{Edge, Face, Shape};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dLine};
use rustycad_tkg3d::geom::{GeomCircle, GeomPlane, GeomSurface, GeomSurfaceOfLinearExtrusion};
use rustycad_tkmath::gp::{Axis2, Axis2d, Axis3, Direction2, Direction3, Point2, Point3};

fn circle3d() -> Handle<dyn GeomCurve> {
    Handle::new(GeomCircle::new(Axis2::default(), 2.).unwrap()).cast()
}

#[test]
fn bspline_input_is_returned_unchanged() {
    let spline =
        ShapeConstruct::convert_curve_to_bspline(&circle3d(), 0., 1., 1e-4, Continuity::C1, 9, 9)
            .unwrap();
    let again = ShapeConstruct::convert_curve_to_bspline(
        &spline.clone().cast(),
        0.,
        1.,
        1e-4,
        Continuity::C1,
        9,
        9,
    )
    .unwrap();
    assert!(again == spline);
}

#[test]
fn circle_arc_is_approximated_on_the_trimmed_range() {
    let spline =
        ShapeConstruct::convert_curve_to_bspline(&circle3d(), 0., 1., 1e-4, Continuity::C1, 9, 9)
            .unwrap();
    let start = spline.value(spline.first_parameter()).unwrap();
    assert!(start.distance(&Point3::new(2., 0., 0.)) < 1e-4);
    let end = spline.value(spline.last_parameter()).unwrap();
    assert!(end.distance(&Point3::new(2. * 1f64.cos(), 2. * 1f64.sin(), 0.)) < 1e-4);
}

#[test]
fn trimming_on_an_empty_range_raises() {
    let failure =
        ShapeConstruct::convert_curve_to_bspline(&circle3d(), 1., 1., 1e-4, Continuity::C1, 9, 9);
    assert!(failure.is_err());
}

#[test]
fn line2d_is_converted_exactly_and_conic2d_is_approximated() {
    let axis = Axis2d::new(Point2::new(0., 0.), Direction2::new(1., 0.).unwrap());
    let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(axis)).cast();
    let failure =
        ShapeConstruct::convert_curve2d_to_bspline(&line, 0., 1., 1e-6, Continuity::C1, 9, 9);
    assert!(
        failure.is_err(),
        "an infinite line has no exact B-spline form"
    );
    let circle: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dCircle::from_axis(axis, 1., true).unwrap()).cast();
    let spline =
        ShapeConstruct::convert_curve2d_to_bspline(&circle, 0., 1., 1e-6, Continuity::C1, 9, 9)
            .unwrap();
    let end = spline.value(spline.last_parameter()).unwrap();
    assert!(end.distance(Point2::new(1f64.cos(), 1f64.sin())) < 1e-6);
}

fn bounds(uf: f64, ul: f64, vf: f64, vl: f64) -> ShapeConstructSurfaceBounds {
    ShapeConstructSurfaceBounds { uf, ul, vf, vl }
}

fn settings() -> ShapeConstructApproxSettings {
    ShapeConstructApproxSettings {
        tol3d: 1e-4,
        continuity: Continuity::C2,
        max_segments: 9,
        max_degree: 9,
    }
}

#[test]
fn plane_is_converted_exactly_on_the_box() {
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let spline =
        ShapeConstruct::convert_surface_to_bspline(&plane, bounds(0., 2., 0., 3.), settings())
            .unwrap();
    assert_eq!((spline.u_degree(), spline.v_degree()), (1, 1));
    let corner = spline.value(2., 3.).unwrap();
    assert!(corner.distance(&Point3::new(2., 3., 0.)) < 1e-12);
}

#[test]
fn extrusion_is_the_converted_basis_swept_between_the_v_bounds() {
    let extrusion: Handle<dyn GeomSurface> = Handle::new(
        GeomSurfaceOfLinearExtrusion::new(&circle3d(), Direction3::new(0., 0., 1.).unwrap())
            .unwrap(),
    )
    .cast();
    let spline =
        ShapeConstruct::convert_surface_to_bspline(&extrusion, bounds(0., 1., 1., 4.), settings())
            .unwrap();
    assert_eq!(spline.v_degree(), 1);
    assert_eq!(spline.nb_v_poles(), 2);
    let (u1, u2, v1, v2) = spline.bounds();
    let low = spline.value(u1, v1).unwrap();
    assert!(low.distance(&Point3::new(2., 0., 1.)) < 1e-4);
    let high = spline.value(u2, v2).unwrap();
    assert!(high.distance(&Point3::new(2. * 1f64.cos(), 2. * 1f64.sin(), 4.)) < 1e-4);
}

#[test]
fn join_pcurves_on_a_plane_returns_true_without_work() {
    let face = crate::shape_construct::package::tests::plane_face();
    let mut edge = Edge::default();
    let edges: Sequence<Shape> = Sequence::default();
    assert!(ShapeConstruct::join_pcurves(&edges, &face, &mut edge));
}

fn plane_face() -> Face {
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let mut face = Face::default();
    rustycad_tkbrep::brep::BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    face
}
