//! Shared fixtures of the TKXCAF tests: XDE documents through a `TDocStd_Application` and
//! box shapes.
#![allow(dead_code)]

use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tklcaf::tdoc_std::{TDocStdApplication, TDocStdDocument, TDocStdStandardApplication};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;

/// `anApp = new TDocStd_Application(); anApp->NewDocument(format, aDoc)`.
pub fn new_document(format: &str) -> (Handle<TDocStdStandardApplication>, Handle<TDocStdDocument>) {
    let application = TDocStdStandardApplication::new();
    let mut document: Handle<TDocStdDocument> = Handle::default();
    application
        .new_tdoc_std_document(&ExtendedString::from_utf8(format), &mut document)
        .unwrap();
    (application, document)
}

/// `BRepPrimAPI_MakeBox(dx, dy, dz).Shape()`.
pub fn make_box(dx: f64, dy: f64, dz: f64) -> Shape {
    BRepPrimAPIMakeBox::from_sizes(dx, dy, dz)
        .unwrap()
        .solid()
        .unwrap()
        .into_shape()
}
