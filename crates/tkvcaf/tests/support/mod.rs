//! Shared fixtures of the TKVCAF port tests: documents, named shapes on labels, constraints
//! and a viewer over the null graphic driver.
#![allow(dead_code)]

use rustycad_tkbrep::topods::Shape;
use rustycad_tkcaf::tdata_xtd::{TDataXtdConstraint, TDataXtdConstraintEnum, TDataXtdPlane};
use rustycad_tkcaf::tnaming::{TNamingBuilder, TNamingNamedShape};
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tklcaf::tdata_std::TDataStdReal;
use rustycad_tklcaf::tdf::TdfLabel;
use rustycad_tklcaf::tdoc_std::TDocStdDocument;
use rustycad_tkmath::gp::{Axis2, Circle3, Direction3, Plane3, Point3};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tkservice::null_driver::NullGraphicDriver;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeShape};
use rustycad_tkv3d::v3d::V3dViewer;

/// `new TDocStd_Document("BinOcaf")`.
pub fn new_document() -> Handle<TDocStdDocument> {
    TDocStdDocument::new(&ExtendedString::from_utf8("BinOcaf")).unwrap()
}

/// `Main().FindChild(tag, true)`.
pub fn main_child(document: &Handle<TDocStdDocument>, tag: i32) -> TdfLabel {
    document.main().unwrap().find_child(tag, true).unwrap()
}

/// A viewer over the null graphic driver.
pub fn null_viewer() -> Handle<V3dViewer> {
    V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap()
}

/// `BRepPrimAPI_MakeBox(dx, dy, dz).Shape()`.
pub fn make_box(dx: f64, dy: f64, dz: f64) -> Shape {
    BRepPrimAPIMakeBox::from_sizes(dx, dy, dz)
        .unwrap()
        .solid()
        .unwrap()
        .into_shape()
}

/// The edge of a full circle of `radius` about the Z axis through `center`.
pub fn circle_edge(center: Point3, radius: f64) -> Shape {
    let frame = Axis2::new(center, Direction3::new(0.0, 0.0, 1.0).unwrap()).unwrap();
    let circle = Circle3::new(frame, radius).unwrap();
    let mut maker = BRepBuilderAPIMakeEdge::from_circ(circle).unwrap();
    maker.shape().unwrap().clone()
}

/// The edge of the segment from `start` to `end`.
pub fn segment_edge(start: Point3, end: Point3) -> Shape {
    let mut maker = BRepBuilderAPIMakeEdge::from_points(start, end).unwrap();
    maker.shape().unwrap().clone()
}

/// A named shape generated with `shape` on a new child of `parent`.
pub fn named_shape(parent: &TdfLabel, shape: &Shape) -> Handle<TNamingNamedShape> {
    let label = parent.new_child().unwrap();
    let builder = TNamingBuilder::new(&label).unwrap();
    builder.generated(shape).unwrap();
    builder.named_shape()
}

/// The named shape of a `TDataXtd_Plane` of the XY plane on a new child of `parent`.
pub fn xy_plane(parent: &TdfLabel) -> Handle<TNamingNamedShape> {
    let label = parent.new_child().unwrap();
    TDataXtdPlane::set_plane(&label, &Plane3::default()).unwrap();
    let mut named: Handle<TNamingNamedShape> = Handle::default();
    label
        .find_attribute_as(&TNamingNamedShape::get_id(), &mut named)
        .unwrap();
    named
}

/// A verified constraint of `kind` on `label` over `geometries`, with a value attribute.
pub fn constraint(
    label: &TdfLabel,
    kind: TDataXtdConstraintEnum,
    geometries: &[Handle<TNamingNamedShape>],
    value: f64,
) -> Handle<TDataXtdConstraint> {
    let constraint = TDataXtdConstraint::set(label).unwrap();
    constraint.set_geometries(kind, geometries).unwrap();
    let real = TDataStdReal::set(&label.new_child().unwrap(), value).unwrap();
    constraint.set_value(&real).unwrap();
    constraint.set_verified(true).unwrap();
    constraint
}
