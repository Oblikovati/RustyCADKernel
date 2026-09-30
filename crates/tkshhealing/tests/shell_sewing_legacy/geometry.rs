//! Original geometric fixtures shared by legacy tests and differential probe.
use super::healing_support::{polygon, shell};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkmath::gp::Point3;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape};

/// Native MakePolygon and MakeFace with its default OnlyPlane=false; `rectangle(0.,1.)`.
pub fn rectangle(first: f64, last: f64) -> Shape {
    let wire = polygon(&[
        (first, 0., 0.),
        (last, 0., 0.),
        (last, 1., 0.),
        (first, 1., 0.),
    ]);
    BRepBuilderAPIMakeFace::from_wire(&wire, false)
        .unwrap()
        .face()
        .unwrap()
        .into_shape()
}
/// Two independent adjacent faces; `open_shell(1e-4)`.
pub fn open_shell(gap: f64) -> Shape {
    shell(&[rectangle(0., 1.), rectangle(1. + gap, 2.)]).into_shape()
}
/// Native box builder at the requested location; `box_shape(0.,(2.,3.,4.))`.
pub fn box_shape(origin: f64, size: (f64, f64, f64)) -> Shape {
    BRepPrimAPIMakeBox::at_point(Point3::new(origin, 0., 0.), size.0, size.1, size.2)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}
