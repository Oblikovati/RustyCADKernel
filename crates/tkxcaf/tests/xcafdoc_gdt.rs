//! 1:1 translation of src/DataExchange/TKXCAF/GTests/XCAFDoc_GDT_Test.cxx.
#![allow(non_snake_case)]
// The native test compares with the literal 0.7071067811865476.
#![allow(clippy::approx_constant)]

mod support;

use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::HAsciiString;
use rustycad_tklcaf::tdf::TdfLabel;
use rustycad_tklcaf::tdoc_std::{TDocStdDocument, TDocStdStandardApplication};
use rustycad_tkmath::gp::{Direction3, Point3};
use rustycad_tkxcaf::xcaf_dim_tol_objects::{
    XCAFDimTolObjectsDatumObject, XCAFDimTolObjectsDatumSingleModif,
    XCAFDimTolObjectsDimensionFormVariance, XCAFDimTolObjectsDimensionGrade,
    XCAFDimTolObjectsDimensionModif, XCAFDimTolObjectsDimensionObject,
    XCAFDimTolObjectsDimensionQualifier, XCAFDimTolObjectsDimensionType,
    XCAFDimTolObjectsGeomToleranceMatReqModif, XCAFDimTolObjectsGeomToleranceModif,
    XCAFDimTolObjectsGeomToleranceObject, XCAFDimTolObjectsGeomToleranceType,
    XCAFDimTolObjectsGeomToleranceTypeValue, XCAFDimTolObjectsGeomToleranceZoneModif,
};
use rustycad_tkxcaf::xcaf_doc::{
    XCAFDocDatum, XCAFDocDimTolTool, XCAFDocDimension, XCAFDocDocumentTool, XCAFDocGeomTolerance,
};

struct GdtContext {
    _application: Handle<TDocStdStandardApplication>,
    _document: Handle<TDocStdDocument>,
    dim_tol_tool: Handle<XCAFDocDimTolTool>,
    shape1: TdfLabel,
    shape2: TdfLabel,
}

impl GdtContext {
    fn tool(&self) -> &XCAFDocDimTolTool {
        self.dim_tol_tool.get().unwrap()
    }
}

fn new_context() -> GdtContext {
    let (application, document) = support::new_document("BinXCAF");
    let main = document.main().unwrap();
    let shape_tool = XCAFDocDocumentTool::shape_tool(&main).unwrap();
    let dim_tol_tool = XCAFDocDocumentTool::dim_tol_tool(&main).unwrap();
    let shapes = shape_tool.get().unwrap();
    let shape1 = shapes
        .add_shape(&support::make_box(10.0, 10.0, 10.0), true, true)
        .unwrap();
    let shape2 = shapes
        .add_shape(&support::make_box(20.0, 20.0, 20.0), true, true)
        .unwrap();
    GdtContext {
        _application: application,
        _document: document,
        dim_tol_tool,
        shape1,
        shape2,
    }
}

fn add_dimension(context: &GdtContext, has_second_shape: bool) -> TdfLabel {
    let tool = context.tool();
    let dimension = tool.add_dimension().unwrap();
    if has_second_shape {
        tool.set_dimension_pair(&context.shape1, &context.shape2, &dimension)
            .unwrap();
    } else {
        tool.set_dimension(&context.shape1, &dimension).unwrap();
    }
    assert!(tool.is_dimension(&dimension).unwrap());
    dimension
}

fn dimension_object(label: &TdfLabel) -> Handle<XCAFDimTolObjectsDimensionObject> {
    let mut attribute: Handle<XCAFDocDimension> = Handle::default();
    assert!(
        label
            .find_attribute_as(&XCAFDocDimension::get_id(), &mut attribute)
            .unwrap()
    );
    attribute.get().unwrap().get_object().unwrap()
}

fn store_dimension(label: &TdfLabel, object: &Handle<XCAFDimTolObjectsDimensionObject>) {
    XCAFDocDimension::set(label)
        .unwrap()
        .get()
        .unwrap()
        .set_object(object)
        .unwrap();
}

fn add_geom_tolerance(context: &GdtContext) -> TdfLabel {
    let tool = context.tool();
    let tolerance = tool.add_geom_tolerance().unwrap();
    tool.set_geom_tolerance(&context.shape1, &tolerance)
        .unwrap();
    assert!(tool.is_geom_tolerance(&tolerance).unwrap());
    tolerance
}

fn geom_tolerance_object(label: &TdfLabel) -> Handle<XCAFDimTolObjectsGeomToleranceObject> {
    let mut attribute: Handle<XCAFDocGeomTolerance> = Handle::default();
    assert!(
        label
            .find_attribute_as(&XCAFDocGeomTolerance::get_id(), &mut attribute)
            .unwrap()
    );
    attribute.get().unwrap().get_object().unwrap()
}

fn store_tolerance(label: &TdfLabel, object: &Handle<XCAFDimTolObjectsGeomToleranceObject>) {
    XCAFDocGeomTolerance::set(label)
        .unwrap()
        .get()
        .unwrap()
        .set_object(object)
        .unwrap();
}

fn add_datum(
    context: &GdtContext,
    tolerance: &TdfLabel,
    name: &str,
    position: i32,
    distance_modifier: bool,
) -> TdfLabel {
    let tool = context.tool();
    let datum = tool.add_datum().unwrap();
    tool.set_datum_shapes(std::slice::from_ref(&context.shape1), &datum)
        .unwrap();
    let mut attribute: Handle<XCAFDocDatum> = Handle::default();
    assert!(
        datum
            .find_attribute_as(&XCAFDocDatum::get_id(), &mut attribute)
            .unwrap()
    );
    let attribute = attribute.get().unwrap();
    let object = attribute.get_object().unwrap();
    let o = object.get().unwrap();
    o.set_name(&Handle::new(HAsciiString::new(name)));
    o.set_position(position);
    if distance_modifier {
        o.add_modifier(XCAFDimTolObjectsDatumSingleModif::DistanceVariable);
    }
    attribute.set_object(&object).unwrap();
    tool.set_datum_to_geom_tol(&datum, tolerance).unwrap();
    datum
}

fn datum_object(label: &TdfLabel) -> Handle<XCAFDimTolObjectsDatumObject> {
    let mut attribute: Handle<XCAFDocDatum> = Handle::default();
    assert!(
        label
            .find_attribute_as(&XCAFDocDatum::get_id(), &mut attribute)
            .unwrap()
    );
    attribute.get().unwrap().get_object().unwrap()
}

fn datum_references(tolerance: &TdfLabel) -> Vec<TdfLabel> {
    let mut datums = Vec::new();
    assert!(XCAFDocDimTolTool::get_datum_of_toler_labels(tolerance, &mut datums).unwrap());
    datums
}

fn name_of(object: &XCAFDimTolObjectsDatumObject) -> String {
    String::from_utf8_lossy(object.get_name().get().unwrap().string().as_bytes()).into_owned()
}

#[test]
fn XCAFDoc_GDT_Test__GdtDimensions_A1_CurveLength() {
    let context = new_context();
    let dimension = add_dimension(&context, false);
    let object = dimension_object(&dimension);
    object
        .get()
        .unwrap()
        .set_type(XCAFDimTolObjectsDimensionType::SizeCurveLength);
    object.get().unwrap().set_value(1.5);
    store_dimension(&dimension, &object);
    let object = dimension_object(&dimension);
    let o = object.get().unwrap();
    assert_eq!(
        o.get_type(),
        XCAFDimTolObjectsDimensionType::SizeCurveLength
    );
    assert_eq!(o.get_value(), 1.5);
}

#[test]
fn XCAFDoc_GDT_Test__GdtDimensions_A2_DiameterQualifier() {
    let context = new_context();
    let dimension = add_dimension(&context, false);
    let object = dimension_object(&dimension);
    let o = object.get().unwrap();
    o.set_type(XCAFDimTolObjectsDimensionType::SizeDiameter);
    o.set_value(2.5);
    o.set_qualifier(XCAFDimTolObjectsDimensionQualifier::Max);
    store_dimension(&dimension, &object);
    let object = dimension_object(&dimension);
    let o = object.get().unwrap();
    assert_eq!(o.get_type(), XCAFDimTolObjectsDimensionType::SizeDiameter);
    assert_eq!(o.get_value(), 2.5);
    assert!(o.has_qualifier());
    assert_eq!(o.get_qualifier(), XCAFDimTolObjectsDimensionQualifier::Max);
}

#[test]
fn XCAFDoc_GDT_Test__GdtDimensions_A3_Range() {
    let context = new_context();
    let dimension = add_dimension(&context, false);
    let object = dimension_object(&dimension);
    let o = object.get().unwrap();
    o.set_type(XCAFDimTolObjectsDimensionType::SizeCurveLength);
    o.set_lower_bound(1.5);
    o.set_upper_bound(2.5);
    store_dimension(&dimension, &object);
    let object = dimension_object(&dimension);
    let o = object.get().unwrap();
    assert_eq!(
        o.get_type(),
        XCAFDimTolObjectsDimensionType::SizeCurveLength
    );
    assert!(o.is_dim_with_range());
    assert_eq!(o.get_lower_bound(), 1.5);
    assert_eq!(o.get_upper_bound(), 2.5);
}

#[test]
fn XCAFDoc_GDT_Test__GdtDimensions_A4_ClassAndModifiers() {
    let context = new_context();
    let dimension = add_dimension(&context, false);
    let object = dimension_object(&dimension);
    let o = object.get().unwrap();
    o.set_type(XCAFDimTolObjectsDimensionType::SizeDiameter);
    o.set_value(2.5);
    o.set_class_of_tolerance(
        true,
        XCAFDimTolObjectsDimensionFormVariance::F,
        XCAFDimTolObjectsDimensionGrade::IT6,
    );
    o.set_nb_of_decimal_places(3, 3);
    o.add_modifier(XCAFDimTolObjectsDimensionModif::AverageSize);
    o.add_modifier(XCAFDimTolObjectsDimensionModif::ControlledRadius);
    store_dimension(&dimension, &object);
    let object = dimension_object(&dimension);
    let o = object.get().unwrap();
    let mut hole = false;
    let mut form = XCAFDimTolObjectsDimensionFormVariance::None;
    let mut grade = XCAFDimTolObjectsDimensionGrade::IT01;
    assert!(o.get_class_of_tolerance(&mut hole, &mut form, &mut grade));
    assert!(hole);
    assert_eq!(form, XCAFDimTolObjectsDimensionFormVariance::F);
    assert_eq!(grade, XCAFDimTolObjectsDimensionGrade::IT6);
    assert_eq!(o.get_nb_of_decimal_places(), (3, 3));
    let modifiers = o.get_modifiers();
    assert_eq!(modifiers.len(), 2);
    assert_eq!(modifiers[0], XCAFDimTolObjectsDimensionModif::AverageSize);
    assert_eq!(
        modifiers[1],
        XCAFDimTolObjectsDimensionModif::ControlledRadius
    );
}

#[test]
fn XCAFDoc_GDT_Test__GdtDimensions_A5_LinearDistance() {
    let context = new_context();
    let dimension = add_dimension(&context, true);
    let object = dimension_object(&dimension);
    let o = object.get().unwrap();
    o.set_type(XCAFDimTolObjectsDimensionType::LocationLinearDistance);
    o.set_value(6.0);
    assert!(o.set_lower_tol_value(-0.001));
    assert!(o.set_upper_tol_value(0.002));
    o.set_point(&Point3::new(10.0, 10.0, 10.0));
    o.set_point2(&Point3::new(20.0, 20.0, 20.0));
    store_dimension(&dimension, &object);
    let object = dimension_object(&dimension);
    let o = object.get().unwrap();
    assert_eq!(
        o.get_type(),
        XCAFDimTolObjectsDimensionType::LocationLinearDistance
    );
    assert_eq!(o.get_value(), 6.0);
    assert!(o.is_dim_with_plus_minus_tolerance());
    assert_eq!(o.get_lower_tol_value(), -0.001);
    assert_eq!(o.get_upper_tol_value(), 0.002);
    assert!(o.has_point());
    assert!(o.has_point2());
    assert!(o.get_point().is_equal(&Point3::new(10.0, 10.0, 10.0), 0.0));
    assert!(o.get_point2().is_equal(&Point3::new(20.0, 20.0, 20.0), 0.0));
}

#[test]
fn XCAFDoc_GDT_Test__GdtDimensions_A6_Oriented() {
    let context = new_context();
    let dimension = add_dimension(&context, true);
    let object = dimension_object(&dimension);
    let o = object.get().unwrap();
    o.set_type(XCAFDimTolObjectsDimensionType::LocationOriented);
    o.set_value(6.0);
    assert!(o.set_lower_tol_value(-0.001));
    assert!(o.set_upper_tol_value(0.002));
    assert!(o.set_direction(&Direction3::new(1.0, 1.0, 0.0).unwrap()));
    store_dimension(&dimension, &object);
    let object = dimension_object(&dimension);
    let o = object.get().unwrap();
    let mut direction = Direction3::default();
    assert_eq!(
        o.get_type(),
        XCAFDimTolObjectsDimensionType::LocationOriented
    );
    assert_eq!(o.get_value(), 6.0);
    assert!(o.is_dim_with_plus_minus_tolerance());
    assert_eq!(o.get_lower_tol_value(), -0.001);
    assert_eq!(o.get_upper_tol_value(), 0.002);
    assert!(o.get_direction(&mut direction));
    assert!((direction.x() - 0.7071067811865476).abs() <= 1.0e-15);
    assert!((direction.y() - 0.7071067811865476).abs() <= 1.0e-15);
    assert_eq!(direction.z(), 0.0);
}

#[test]
fn XCAFDoc_GDT_Test__GdtTolerances_A1_FullDefinition() {
    let context = new_context();
    let tolerance = add_geom_tolerance(&context);
    let datum = add_datum(&context, &tolerance, "A", 1, true);
    let object = geom_tolerance_object(&tolerance);
    let o = object.get().unwrap();
    o.set_value(0.5);
    o.set_type(XCAFDimTolObjectsGeomToleranceType::Angularity);
    o.set_type_of_value(XCAFDimTolObjectsGeomToleranceTypeValue::Diameter);
    o.set_material_requirement_modifier(XCAFDimTolObjectsGeomToleranceMatReqModif::M);
    o.set_zone_modifier(XCAFDimTolObjectsGeomToleranceZoneModif::Projected);
    o.set_value_of_zone_modifier(1.0);
    o.add_modifier(XCAFDimTolObjectsGeomToleranceModif::MajorDiameter);
    o.add_modifier(XCAFDimTolObjectsGeomToleranceModif::CommonZone);
    o.set_max_value_modifier(1.5);
    store_tolerance(&tolerance, &object);
    let object = geom_tolerance_object(&tolerance);
    let o = object.get().unwrap();
    assert_eq!(o.get_value(), 0.5);
    assert_eq!(o.get_type(), XCAFDimTolObjectsGeomToleranceType::Angularity);
    assert_eq!(
        o.get_type_of_value(),
        XCAFDimTolObjectsGeomToleranceTypeValue::Diameter
    );
    assert_eq!(
        o.get_material_requirement_modifier(),
        XCAFDimTolObjectsGeomToleranceMatReqModif::M
    );
    assert_eq!(
        o.get_zone_modifier(),
        XCAFDimTolObjectsGeomToleranceZoneModif::Projected
    );
    assert_eq!(o.get_value_of_zone_modifier(), 1.0);
    assert_eq!(o.get_max_value_modifier(), 1.5);
    let modifiers = o.get_modifiers();
    assert_eq!(modifiers.len(), 2);
    assert_eq!(
        modifiers[0],
        XCAFDimTolObjectsGeomToleranceModif::MajorDiameter
    );
    assert_eq!(
        modifiers[1],
        XCAFDimTolObjectsGeomToleranceModif::CommonZone
    );
    let datums = datum_references(&tolerance);
    assert_eq!(datums.len(), 1);
    assert!(datums[0] == datum);
    let datum_object = datum_object(&datum);
    let d = datum_object.get().unwrap();
    assert_eq!(name_of(d), "A");
    assert_eq!(d.get_position(), 1);
    let datum_modifiers = d.get_modifiers();
    assert_eq!(datum_modifiers.len(), 1);
    assert_eq!(
        datum_modifiers[0],
        XCAFDimTolObjectsDatumSingleModif::DistanceVariable
    );
}

#[test]
fn XCAFDoc_GDT_Test__GdtTolerances_A2_TwoDatums() {
    let context = new_context();
    let tolerance = add_geom_tolerance(&context);
    let datum_a = add_datum(&context, &tolerance, "A", 1, true);
    let datum_b = add_datum(&context, &tolerance, "B", 2, false);
    let object = geom_tolerance_object(&tolerance);
    object.get().unwrap().set_value(0.5);
    store_tolerance(&tolerance, &object);
    let object = geom_tolerance_object(&tolerance);
    assert_eq!(object.get().unwrap().get_value(), 0.5);
    let datums = datum_references(&tolerance);
    assert_eq!(datums.len(), 2);
    assert!(datums[0] == datum_a);
    assert!(datums[1] == datum_b);
    let object_a = datum_object(&datum_a);
    let object_b = datum_object(&datum_b);
    let (a, b) = (object_a.get().unwrap(), object_b.get().unwrap());
    assert_eq!(name_of(a), "A");
    assert_eq!(a.get_position(), 1);
    assert_eq!(name_of(b), "B");
    assert_eq!(b.get_position(), 2);
    assert_eq!(a.get_modifiers().len(), 1);
    assert_eq!(b.get_modifiers().len(), 0);
}
