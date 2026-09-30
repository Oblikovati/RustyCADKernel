//! Shared construction and measurements for legacy suites unblocked by TKPrim.
#![allow(dead_code)]
use rustycad_tkbrep::topods::Shape;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::{Point3, Transform3};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeShape, BRepBuilderAPITransform};
use rustycad_tktopalgo::brep_gprop::BRepGProp;

pub fn legacy_box_sizes(dx: f64, dy: f64, dz: f64) -> Shape {
    let mut maker = BRepPrimAPIMakeBox::from_sizes(dx, dy, dz).unwrap();
    let shape = maker.shape().unwrap().clone();
    assert!(maker.is_done(), "Box creation failed");
    shape
}

pub fn legacy_box_at(corner: Point3) -> Shape {
    let mut maker = BRepPrimAPIMakeBox::at_point(corner, 10.0, 10.0, 10.0).unwrap();
    let shape = maker.shape().unwrap().clone();
    assert!(maker.is_done(), "Box creation failed");
    shape
}

pub fn legacy_transform(shape: &Shape, transform: Transform3) -> Shape {
    let mut maker = BRepBuilderAPITransform::with_shape(shape, transform, false, false).unwrap();
    let transformed = maker.shape().unwrap().clone();
    assert!(!transformed.is_null(), "Transformed shape is null");
    transformed
}

pub fn legacy_volume_properties(shape: &Shape) -> GProps {
    let mut properties = GProps::new();
    BRepGProp::volume_properties(shape, &mut properties, false, false, false).unwrap();
    properties
}
