//! Fixtures shared by the TKCAF GTest translations: documents, boxes and attribute lookups.
#![allow(dead_code)]
pub mod dnaming;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkcaf::tnaming::TNamingNamedShape;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tklcaf::tdf::TdfLabel;
use rustycad_tklcaf::tdoc_std::{TDocStdApplication, TDocStdDocument, TDocStdStandardApplication};
use rustycad_tkmath::gp::Point3;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;

/// `myApp->NewDocument("BinOcaf", myDoc)` of the fixtures (the Bin drivers are not ported
/// and no test reads or writes a file).
pub fn new_application_document() -> (Handle<TDocStdStandardApplication>, Handle<TDocStdDocument>) {
    let application = TDocStdStandardApplication::new();
    let mut document: Handle<TDocStdDocument> = Handle::default();
    application
        .new_tdoc_std_document(&ExtendedString::new("BinOcaf"), &mut document)
        .unwrap();
    (application, document)
}

/// `new TDocStd_Document("BinOcaf")`.
pub fn new_document() -> Handle<TDocStdDocument> {
    TDocStdDocument::new(&ExtendedString::new("BinOcaf")).unwrap()
}

/// `Main().FindChild(tag, true)`.
pub fn main_child(document: &Handle<TDocStdDocument>, tag: i32) -> TdfLabel {
    document.main().unwrap().find_child(tag, true).unwrap()
}

/// `BRepPrimAPI_MakeBox(dx, dy, dz).Shape()`.
pub fn make_box(dx: f64, dy: f64, dz: f64) -> Shape {
    BRepPrimAPIMakeBox::from_sizes(dx, dy, dz)
        .unwrap()
        .solid()
        .unwrap()
        .into_shape()
}

/// `BRepPrimAPI_MakeBox(P1, P2).Shape()`.
pub fn make_box_between(first: [f64; 3], second: [f64; 3]) -> Shape {
    let first = Point3::new(first[0], first[1], first[2]);
    let second = Point3::new(second[0], second[1], second[2]);
    BRepPrimAPIMakeBox::from_points(first, second)
        .unwrap()
        .solid()
        .unwrap()
        .into_shape()
}

/// `label.FindAttribute(TNaming_NamedShape::GetID(), aNS)`.
pub fn find_named_shape(label: &TdfLabel) -> Option<Handle<TNamingNamedShape>> {
    let mut named: Handle<TNamingNamedShape> = Handle::default();
    label
        .find_attribute_as(&TNamingNamedShape::get_id(), &mut named)
        .unwrap()
        .then_some(named)
}

/// The sub-shapes of a type in exploration order (`TopExp_Explorer`).
pub fn explore(shape: &Shape, shape_type: ShapeEnum) -> Vec<Shape> {
    let mut shapes = Vec::new();
    let mut explorer = Explorer::new(shape, shape_type, ShapeEnum::Shape).unwrap();
    while explorer.more() {
        shapes.push(explorer.current().clone());
        explorer.next().unwrap();
    }
    shapes
}

/// The `index`-th distinct sub-shape of a type (1-based), skipping the shape itself.
pub fn shape_at(shape: &Shape, shape_type: ShapeEnum, index: usize) -> Shape {
    let mut seen = MapOfShape::default();
    seen.add(shape.clone());
    let mut count = 0;
    for current in explore(shape, shape_type) {
        if !seen.add(current.clone()) {
            continue;
        }
        count += 1;
        if count == index {
            return current;
        }
    }
    Shape::default()
}
