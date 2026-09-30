//! ShapeConstruct_Curve parity cases; OCCT 3d097a0 keeps copies after segmentation failure.
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Handle, real::epsilon};
use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomLine};
use rustycad_tkmath::gp::{Axis1, Axis2, Axis2d, Axis22d, Direction3, Point2, Point3};
use rustycad_tkshhealing::shape_construct::ShapeConstructCurve;
include!("shape_construct_curve/adjust.rs");
include!("shape_construct_curve/convert.rs");
include!("shape_construct_curve/knots.rs");

fn construct_line_3d() -> Handle<dyn GeomCurve> {
    let axis = Axis1::new(Point3::new(2., 3., 4.), Direction3::X);
    let line = GeomLine::new(axis);
    Handle::new(line).cast()
}

fn construct_line_2d() -> Handle<dyn Geom2dCurve> {
    let axis = Axis2d::default();
    let line = Geom2dLine::new(axis);
    Handle::new(line).cast()
}

fn construct_spline_3d() -> Handle<rustycad_tkg3d::geom::GeomBSplineCurve> {
    let line = construct_line_3d();
    ShapeConstructCurve
        .convert_to_bspline(&line, 0., 1., 1e-7)
        .unwrap()
}

fn construct_spline_2d() -> Handle<rustycad_tkg2d::geom2d::Geom2dBSplineCurve> {
    let line = construct_line_2d();
    ShapeConstructCurve
        .convert_to_bspline_2d(&line, 0., 1., 1e-7)
        .unwrap()
}
