//! 1:1 translation of the cases of src/ApplicationFramework/TKLCAF/GTests/TDocStd_Transaction_Test.cxx
//! that use TNaming named shapes and TDataXtd attributes, which the TKCAF crate provides (the
//! other cases are in the TKLCAF crate). The fixture's `BinDrivers::DefineFormat` registers
//! the native Bin format, which is not ported (user decision 2026-09); no case reads or writes
//! a file, so the translation omits that call.
#![allow(non_snake_case)]
mod support;

use rustycad_tkcaf::tdata_xtd::{
    TDataXtdAxis, TDataXtdConstraint, TDataXtdConstraintEnum, TDataXtdGeometry,
    TDataXtdGeometryEnum, TDataXtdPatternStd, TDataXtdPlane, TDataXtdPoint, TDataXtdPosition,
};
use rustycad_tkcaf::tnaming::{TNamingBuilder, TNamingNamedShape};
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tklcaf::tdata_std::{TDataStdInteger, TDataStdReal};
use rustycad_tklcaf::tdf::TdfLabel;
use rustycad_tklcaf::tdoc_std::{TDocStdApplication, TDocStdDocument, TDocStdStandardApplication};
use rustycad_tkmath::gp::{Direction3, Line3, Plane3, Point3};

/// `NewApplicationTransactionDocument(anApplication)`.
fn new_application_transaction_document()
-> (Handle<TDocStdStandardApplication>, Handle<TDocStdDocument>) {
    let application = TDocStdStandardApplication::new();
    let mut document: Handle<TDocStdDocument> = Handle::default();
    application
        .new_tdoc_std_document(&ExtendedString::new("BinOcaf"), &mut document)
        .unwrap();
    (application, document)
}

/// `RootChild(theDocument, theTag)`.
fn root_child(document: &Handle<TDocStdDocument>, tag: i32) -> TdfLabel {
    document
        .main()
        .unwrap()
        .root()
        .unwrap()
        .find_child(tag, true)
        .unwrap()
}

/// `DeltaHasAttribute(theDelta, theAttributeName)` on the last undo.
fn last_delta_has(document: &Handle<TDocStdDocument>, attribute_name: &str) -> bool {
    let undos = document.get_undos();
    let delta = undos.last().expect("an undo");
    delta.attribute_deltas().iter().any(|attribute_delta| {
        attribute_delta.attribute().dynamic_type().unwrap().name() == attribute_name
    })
}

/// `ExpectLastDeltaIs(theDocument, theAttributeName)`.
fn expect_last_delta_is(document: &Handle<TDocStdDocument>, attribute_name: &str) {
    let undos = document.get_undos();
    let delta = undos.last().expect("an undo");
    let deltas = delta.attribute_deltas();
    let last = deltas.last().expect("an attribute delta");
    assert_eq!(
        last.attribute().dynamic_type().unwrap().name(),
        attribute_name
    );
}

/// `ExpectLastDeltaIsReal(theDocument)`.
fn expect_last_delta_is_real(document: &Handle<TDocStdDocument>) {
    expect_last_delta_is(document, "TDataStd_Real");
}

/// The common shape of the bug2932 cases: `set` twice with the same values around a real
/// value change (300 then 400) on root child 2, each in its own command.
fn set_twice_around_real(document: &Handle<TDocStdDocument>, set: impl Fn(&TdfLabel)) {
    document.set_undo_limit(100).unwrap();
    let label = root_child(document, 2);
    for value in [300.0, 400.0] {
        document.new_command().unwrap();
        set(&label);
        TDataStdReal::set(&label, value).unwrap();
    }
    document.new_command().unwrap();
}

/// The named shape that `TNaming_Builder(label).Generated(shape)` leaves on `label`.
fn generated_named_shape(
    label: &TdfLabel,
    shape: &rustycad_tkbrep::topods::Shape,
) -> Handle<TNamingNamedShape> {
    TNamingBuilder::new(label)
        .unwrap()
        .generated(shape)
        .unwrap();
    support::find_named_shape(label).expect("a named shape")
}

/// `ExpectLastDeltaHas(theDocument, first, second)`.
fn expect_last_delta_has(document: &Handle<TDocStdDocument>, first: &str, second: &str) {
    assert!(last_delta_has(document, first), "{first}");
    assert!(last_delta_has(document, second), "{second}");
}

// bugs/caf/bug2932_18: an unchanged shape keeps both deltas.
#[test]
fn TDocStd_Transaction_Test__CafBug_2932_18_UnchangedShapeKeepsBothDeltas() {
    let (_application, document) = new_application_transaction_document();
    assert!(!document.is_null());
    document.set_undo_limit(100).unwrap();
    let label = root_child(&document, 2);
    let solid = support::make_box_at([10.0, 20.0, 30.0], 100.0, 200.0, 300.0);
    document.new_command().unwrap();
    TNamingBuilder::new(&label)
        .unwrap()
        .generated(&solid)
        .unwrap();
    TDataStdReal::set(&label, 300.0).unwrap();
    document.new_command().unwrap();
    TNamingBuilder::new(&label)
        .unwrap()
        .generated(&solid)
        .unwrap();
    TDataStdReal::set(&label, 400.0).unwrap();
    document.new_command().unwrap();
    expect_last_delta_has(&document, "TDataStd_Real", "TNaming_NamedShape");
}

// bugs/caf/bug2932_3: repeating an unchanged axis attribute must not replace
// the real attribute modification in the transaction delta.
#[test]
fn TDocStd_Transaction_Test__CafBug_2932_3_UnchangedAxisKeepsRealDelta() {
    let (_application, document) = new_application_transaction_document();
    assert!(!document.is_null());
    let point = Point3::new(10.0, 20.0, 30.0);
    let axis = Line3::new(point, Direction3::new(100.0, 200.0, 300.0).unwrap());
    set_twice_around_real(&document, |label| {
        TDataXtdAxis::set_line(label, &axis).unwrap();
    });
    expect_last_delta_is_real(&document);
}

// bugs/caf/bug2932_24: an unchanged position must not replace the real
// attribute modification in the transaction delta.
#[test]
fn TDocStd_Transaction_Test__CafBug_2932_24_UnchangedPositionKeepsRealDelta() {
    let document = support::new_document();
    let position = Point3::new(123.456, 789.012, 345.678);
    set_twice_around_real(&document, |label| {
        TDataXtdPosition::set_position_of(label, position).unwrap();
    });
    expect_last_delta_is_real(&document);
}

// bugs/caf/bug2932_8: repeating unchanged construction geometry keeps both the
// named-shape and real attribute deltas in the transaction.
#[test]
fn TDocStd_Transaction_Test__CafBug_2932_8_UnchangedGeometryKeepsBothDeltas() {
    let (_application, document) = new_application_transaction_document();
    assert!(!document.is_null());
    let solid = support::make_box_at([10.0, 20.0, 30.0], 100.0, 200.0, 300.0);
    set_twice_around_real(&document, |label| {
        TNamingBuilder::new(label)
            .unwrap()
            .generated(&solid)
            .unwrap();
        let geometry = TDataXtdGeometry::set(label).unwrap();
        geometry.set_type(TDataXtdGeometryEnum::Point).unwrap();
    });
    expect_last_delta_has(&document, "TDataStd_Real", "TNaming_NamedShape");
}

// bugs/caf/bug2932_13: an unchanged plane construction attribute must not
// replace the real attribute modification in the transaction delta.
#[test]
fn TDocStd_Transaction_Test__CafBug_2932_13_UnchangedPlaneKeepsRealDelta() {
    let (_application, document) = new_application_transaction_document();
    assert!(!document.is_null());
    let normal = Direction3::new(-1.0, 0.0, 0.0).unwrap();
    let plane = Plane3::new(Point3::new(10.0, 20.0, 30.0), normal).unwrap();
    set_twice_around_real(&document, |label| {
        TDataXtdPlane::set_plane(label, &plane).unwrap();
    });
    expect_last_delta_is_real(&document);
}

// bugs/caf/bug2932_14: an unchanged point construction attribute must not
// replace the real attribute modification in the transaction delta.
#[test]
fn TDocStd_Transaction_Test__CafBug_2932_14_UnchangedPointKeepsRealDelta() {
    let (_application, document) = new_application_transaction_document();
    assert!(!document.is_null());
    let point = Point3::new(10.0, 20.0, 30.0);
    set_twice_around_real(&document, |label| {
        TDataXtdPoint::set_point(label, point).unwrap();
    });
    expect_last_delta_is_real(&document);
}

/// The constraint metadata of bug2932_5: type, first geometry and plane.
fn set_tangent_constraint(
    label: &TdfLabel,
    named: &Handle<TNamingNamedShape>,
) -> Handle<TDataXtdConstraint> {
    let constraint = TDataXtdConstraint::set(label).unwrap();
    constraint
        .set_type(TDataXtdConstraintEnum::Tangent)
        .unwrap();
    constraint.set_geometry(1, named).unwrap();
    constraint.set_plane(named).unwrap();
    constraint
}

// bugs/caf/bug2932_5: unchanged constraint metadata must not replace the real
// attribute modification in the transaction delta.
#[test]
fn TDocStd_Transaction_Test__CafBug_2932_5_UnchangedConstraintKeepsRealDelta() {
    let (_application, document) = new_application_transaction_document();
    assert!(!document.is_null());
    document.set_undo_limit(100).unwrap();
    let [label, shape_label, integer_label] = [2, 3, 4].map(|tag| root_child(&document, tag));
    let solid = support::make_box_at([10.0, 20.0, 30.0], 100.0, 200.0, 300.0);
    document.new_command().unwrap();
    let named = generated_named_shape(&shape_label, &solid);
    TDataStdInteger::set(&integer_label, 10).unwrap();
    set_tangent_constraint(&label, &named);
    TDataStdReal::set(&label, 300.0).unwrap();
    document.new_command().unwrap();
    let constraint = set_tangent_constraint(&label, &named);
    TDataStdReal::set(&label, 400.0).unwrap();
    document.new_command().unwrap();
    assert_eq!(constraint.get_type(), TDataXtdConstraintEnum::Tangent);
    assert_eq!(constraint.nb_geometries(), 1);
    assert!(!constraint.get_plane().is_null());
    expect_last_delta_is_real(&document);
}

/// The pattern metadata of bug2932_12: signature 1, axis, value and instance count.
fn set_linear_pattern(
    label: &TdfLabel,
    axis: &Handle<TNamingNamedShape>,
    value: &Handle<TDataStdReal>,
    instances: &Handle<TDataStdInteger>,
) -> Handle<TDataXtdPatternStd> {
    let pattern = TDataXtdPatternStd::set(label).unwrap();
    pattern.set_signature(1).unwrap();
    pattern.set_axis1(axis).unwrap();
    pattern.set_value1(value).unwrap();
    pattern.set_nb_instances1(instances).unwrap();
    pattern
}

// bugs/caf/bug2932_12: unchanged pattern metadata must not replace the real
// attribute modification in the transaction delta.
#[test]
fn TDocStd_Transaction_Test__CafBug_2932_12_UnchangedPatternKeepsRealDelta() {
    let (_application, document) = new_application_transaction_document();
    assert!(!document.is_null());
    document.set_undo_limit(100).unwrap();
    let [label, shape_label, real_label, integer_label] =
        [2, 3, 4, 5].map(|tag| root_child(&document, tag));
    let solid = support::make_box_at([10.0, 20.0, 30.0], 100.0, 200.0, 300.0);
    document.new_command().unwrap();
    let named = generated_named_shape(&shape_label, &solid);
    let value = TDataStdReal::set(&real_label, 123.456789).unwrap();
    let instances = TDataStdInteger::set(&integer_label, 123456789).unwrap();
    set_linear_pattern(&label, &named, &value, &instances);
    TDataStdReal::set(&label, 300.0).unwrap();
    document.new_command().unwrap();
    let pattern = set_linear_pattern(&label, &named, &value, &instances);
    TDataStdReal::set(&label, 400.0).unwrap();
    document.new_command().unwrap();

    assert_eq!(pattern.signature(), 1);
    let axis = pattern.axis1();
    assert!(!axis.is_null());
    assert!(TNamingNamedShape::get(&axis).unwrap().is_equal(&solid));
    assert!(!pattern.value1().is_null());
    assert_eq!(pattern.value1().get().unwrap().get(), 123.456789);
    assert!(!pattern.nb_instances1().is_null());
    assert_eq!(pattern.nb_instances1().get().unwrap().get(), 123456789);
    expect_last_delta_is_real(&document);
}
