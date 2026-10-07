//! Behavior tests of ShapeCustom_BSplineRestriction (the native comparison is the sc_modif
//! kind 5 requests of tools/tkshhealing_corpus_shapecustom.py).
mod healing_support;

use healing_support::square_face;
use rustycad_tkbrep::brep_tools::{BRepToolsModification, NewSurface};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Vertex};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBSplineSurface, GeomCurve, GeomLine};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Axis1, Direction3, Point3};
use rustycad_tkshhealing::shape_custom::{
    ShapeCustomBSplineRestriction, ShapeCustomRestrictionParameters,
};

fn first_vertex(face: &rustycad_tkbrep::topods::Face) -> Vertex {
    let shape = face.clone().into_shape();
    let explorer = Explorer::new(&shape, ShapeEnum::Vertex, ShapeEnum::Shape).unwrap();
    topods::vertex(explorer.current()).unwrap()
}

#[test]
fn plane_is_kept_without_the_plane_mode() {
    let face = square_face(0.);
    let restriction = ShapeCustomBSplineRestriction::new();
    assert!(
        !restriction
            .new_surface(&face, &mut NewSurface::default())
            .unwrap()
    );
    let mut point = Point3::new(7., 7., 7.);
    let mut tolerance = -1.;
    assert!(
        !restriction
            .new_point(&first_vertex(&face), &mut point, &mut tolerance)
            .unwrap()
    );
    assert!(tolerance > 0.);
}

#[test]
fn plane_mode_gives_a_bilinear_spline_on_the_face_box() {
    let face = square_face(0.);
    let restriction = ShapeCustomBSplineRestriction::new();
    restriction.restriction_parameters().set_convert_plane(true);
    let mut result = NewSurface::default();
    assert!(restriction.new_surface(&face, &mut result).unwrap());
    let spline = result.surface.cast::<GeomBSplineSurface>();
    let spline = spline.get().expect("a B-spline surface");
    assert_eq!(
        (spline.u_degree(), spline.v_degree(), spline.nb_u_poles()),
        (1, 1, 2)
    );
    assert_eq!(result.tolerance, 1e-7);
}

#[test]
fn line_with_convert_curve3d_becomes_a_degree_one_spline() {
    let restriction = ShapeCustomBSplineRestriction::new();
    restriction
        .restriction_parameters()
        .set_convert_curve3d(true);
    let z = Direction3::new(0., 0., 1.).unwrap();
    let line: Handle<dyn GeomCurve> =
        Handle::new(GeomLine::new(Axis1::new(Point3::new(0., 0., 0.), z))).cast();
    let mut result: Handle<dyn GeomCurve> = Handle::default();
    let mut tol = 1e-7;
    assert!(
        restriction
            .convert_curve(&line, &mut result, false, (0., 2.), &mut tol, true)
            .unwrap()
    );
    let spline = result.cast::<GeomBSplineCurve>();
    let spline = spline.get().expect("a B-spline curve");
    assert_eq!((spline.degree(), spline.nb_poles()), (1, 2));
    assert_eq!(spline.pole(2).unwrap(), Point3::new(0., 0., 2.));
}

#[test]
fn null_restriction_parameters_raise_the_native_null_dereference() {
    let restriction = ShapeCustomBSplineRestriction::new();
    restriction.set_restriction_parameters(&Handle::<ShapeCustomRestrictionParameters>::default());
    let failure = restriction
        .new_surface(&square_face(0.), &mut NewSurface::default())
        .unwrap_err();
    assert!(failure.is_kind("Standard_NullObject"));
}

#[test]
fn accessors_round_trip() {
    let restriction = ShapeCustomBSplineRestriction::new();
    restriction.set_approx_surface_flag(false);
    restriction.set_approx_curve_3d_flag(false);
    restriction.set_approx_curve_2d_flag(false);
    assert!(!restriction.approx_surface_flag());
    assert!(!restriction.approx_curve_3d_flag() && !restriction.approx_curve_2d_flag());
    assert_eq!(restriction.max_errors(), (1e-7, 1e-7, 1e-9));
}
