// Rust port of DataExchange/TKXCAF/XCAFDimTolObjects/XCAFDimTolObjects_DatumObject.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XCAFDimTolObjects_DatumObject`: the description of a datum (name, modifiers, datum
//! target, position and presentation).
use super::{
    XCAFDimTolObjectsDatumModifWithValue, XCAFDimTolObjectsDatumSingleModif,
    XCAFDimTolObjectsDatumTargetType,
};
use crate::json::{field_axis2, field_h_string, field_point, field_shape};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::HAsciiString;
use rustycad_tkmath::gp::{Axis2, Point3};
use rustycad_tkservice::json_dump::{class_begin, field_integer, field_real};
use std::io;

/// The fields of `XCAFDimTolObjects_DatumObject`. The native default constructor leaves the
/// modifier with value, its value, the position, the target type, length, width and number
/// uninitialized; the OCCT allocator clears memory, so they read zero, as here.
#[derive(Clone, Default)]
struct DatumFields {
    name: Handle<HAsciiString>,
    modifiers: Vec<XCAFDimTolObjectsDatumSingleModif>,
    modifier_with_value: XCAFDimTolObjectsDatumModifWithValue,
    value_of_modifier: f64,
    datum_target: Shape,
    position: i32,
    is_datum_target: bool,
    is_valid_datum_target: bool,
    datum_target_type: XCAFDimTolObjectsDatumTargetType,
    length: f64,
    width: f64,
    datum_target_number: i32,
    axis: Axis2,
    plane: Axis2,
    point: Point3,
    point_text: Point3,
    has_plane: bool,
    has_point: bool,
    has_point_text: bool,
    presentation: Shape,
    semantic_name: Handle<HAsciiString>,
    presentation_name: Handle<HAsciiString>,
}

/// `XCAFDimTolObjects_DatumObject`.
///
/// ```
/// use rustycad_tkxcaf::xcaf_dim_tol_objects::XCAFDimTolObjectsDatumObject;
/// let datum = XCAFDimTolObjectsDatumObject::new();
/// datum.set_position(2);
/// assert_eq!(datum.get_position(), 2);
/// assert!(datum.get_name().get().unwrap().is_empty());
/// ```
#[derive(Default)]
pub struct XCAFDimTolObjectsDatumObject {
    transient: StandardTransient,
    fields: SharedState<DatumFields>,
}
rustycad_tkernel::impl_transient!(
    XCAFDimTolObjectsDatumObject,
    "XCAFDimTolObjects_DatumObject",
    StandardTransient,
    transient
);

impl XCAFDimTolObjectsDatumObject {
    /// `XCAFDimTolObjects_DatumObject()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `XCAFDimTolObjects_DatumObject(theObj)`: a copy of every field (a null object is a
    /// null dereference natively; here it gives a default object).
    pub fn from_object(object: &Handle<XCAFDimTolObjectsDatumObject>) -> Self {
        let fields = object
            .get()
            .map(|o| o.fields.read().clone())
            .unwrap_or_default();
        Self {
            transient: StandardTransient::default(),
            fields: SharedState::new(fields),
        }
    }
    /// `GetSemanticName`.
    pub fn get_semantic_name(&self) -> Handle<HAsciiString> {
        self.fields.read().semantic_name.clone()
    }
    /// `SetSemanticName`.
    pub fn set_semantic_name(&self, name: &Handle<HAsciiString>) {
        self.fields.write().semantic_name = name.clone();
    }
    /// `GetName`: the name, or a new empty string when there is none.
    pub fn get_name(&self) -> Handle<HAsciiString> {
        let name = self.fields.read().name.clone();
        if name.is_null() {
            return Handle::new(HAsciiString::new(""));
        }
        name
    }
    /// `SetName`.
    pub fn set_name(&self, name: &Handle<HAsciiString>) {
        self.fields.write().name = name.clone();
    }
    /// `GetModifiers`: a copy of the modifiers.
    pub fn get_modifiers(&self) -> Vec<XCAFDimTolObjectsDatumSingleModif> {
        self.fields.read().modifiers.clone()
    }
    /// `SetModifiers`.
    pub fn set_modifiers(&self, modifiers: &[XCAFDimTolObjectsDatumSingleModif]) {
        self.fields.write().modifiers = modifiers.to_vec();
    }
    /// `GetModifierWithValue(theModifier, theValue)`.
    pub fn get_modifier_with_value(&self) -> (XCAFDimTolObjectsDatumModifWithValue, f64) {
        let fields = self.fields.read();
        (fields.modifier_with_value, fields.value_of_modifier)
    }
    /// `SetModifierWithValue(theModifier, theValue)`.
    pub fn set_modifier_with_value(
        &self,
        modifier: XCAFDimTolObjectsDatumModifWithValue,
        value: f64,
    ) {
        let mut fields = self.fields.write();
        (fields.modifier_with_value, fields.value_of_modifier) = (modifier, value);
    }
    /// `AddModifier`: appends a modifier.
    pub fn add_modifier(&self, modifier: XCAFDimTolObjectsDatumSingleModif) {
        self.fields.write().modifiers.push(modifier);
    }
    /// `GetDatumTarget`.
    pub fn get_datum_target(&self) -> Shape {
        self.fields.read().datum_target.clone()
    }
    /// `SetDatumTarget`.
    pub fn set_datum_target(&self, shape: &Shape) {
        self.fields.write().datum_target = shape.clone();
    }
    /// `GetPosition`.
    pub fn get_position(&self) -> i32 {
        self.fields.read().position
    }
    /// `SetPosition`.
    pub fn set_position(&self, position: i32) {
        self.fields.write().position = position;
    }
    /// `IsDatumTarget()`.
    pub fn is_datum_target(&self) -> bool {
        self.fields.read().is_datum_target
    }
    /// `IsDatumTarget(theIsDT)`.
    pub fn set_is_datum_target(&self, is_datum_target: bool) {
        self.fields.write().is_datum_target = is_datum_target;
    }
}

/// The datum target parameters.
impl XCAFDimTolObjectsDatumObject {
    /// `GetDatumTargetType`.
    pub fn get_datum_target_type(&self) -> XCAFDimTolObjectsDatumTargetType {
        self.fields.read().datum_target_type
    }
    /// `SetDatumTargetType`.
    pub fn set_datum_target_type(&self, kind: XCAFDimTolObjectsDatumTargetType) {
        self.fields.write().datum_target_type = kind;
    }
    /// `GetDatumTargetAxis`.
    pub fn get_datum_target_axis(&self) -> Axis2 {
        self.fields.read().axis
    }
    /// `SetDatumTargetAxis`: the parameters become valid.
    pub fn set_datum_target_axis(&self, axis: &Axis2) {
        let mut fields = self.fields.write();
        (fields.axis, fields.is_valid_datum_target) = (*axis, true);
    }
    /// `GetDatumTargetLength`.
    pub fn get_datum_target_length(&self) -> f64 {
        self.fields.read().length
    }
    /// `SetDatumTargetLength`: the parameters become valid.
    pub fn set_datum_target_length(&self, length: f64) {
        let mut fields = self.fields.write();
        (fields.length, fields.is_valid_datum_target) = (length, true);
    }
    /// `GetDatumTargetWidth`.
    pub fn get_datum_target_width(&self) -> f64 {
        self.fields.read().width
    }
    /// `SetDatumTargetWidth`: the parameters become valid.
    pub fn set_datum_target_width(&self, width: f64) {
        let mut fields = self.fields.write();
        (fields.width, fields.is_valid_datum_target) = (width, true);
    }
    /// `GetDatumTargetNumber`.
    pub fn get_datum_target_number(&self) -> i32 {
        self.fields.read().datum_target_number
    }
    /// `SetDatumTargetNumber`.
    pub fn set_datum_target_number(&self, number: i32) {
        self.fields.write().datum_target_number = number;
    }
    /// `HasDatumTargetParams`.
    pub fn has_datum_target_params(&self) -> bool {
        self.fields.read().is_valid_datum_target
    }
}

/// The presentation members (inline natively).
impl XCAFDimTolObjectsDatumObject {
    /// `SetPlane`.
    pub fn set_plane(&self, plane: &Axis2) {
        let mut fields = self.fields.write();
        (fields.plane, fields.has_plane) = (*plane, true);
    }
    /// `GetPlane`.
    pub fn get_plane(&self) -> Axis2 {
        self.fields.read().plane
    }
    /// `SetPoint`.
    pub fn set_point(&self, point: &Point3) {
        let mut fields = self.fields.write();
        (fields.point, fields.has_point) = (*point, true);
    }
    /// `GetPoint`.
    pub fn get_point(&self) -> Point3 {
        self.fields.read().point
    }
    /// `SetPointTextAttach`.
    pub fn set_point_text_attach(&self, point: &Point3) {
        let mut fields = self.fields.write();
        (fields.point_text, fields.has_point_text) = (*point, true);
    }
    /// `GetPointTextAttach`.
    pub fn get_point_text_attach(&self) -> Point3 {
        self.fields.read().point_text
    }
    /// `HasPlane`.
    pub fn has_plane(&self) -> bool {
        self.fields.read().has_plane
    }
    /// `HasPoint`.
    pub fn has_point(&self) -> bool {
        self.fields.read().has_point
    }
    /// `HasPointText`.
    pub fn has_point_text(&self) -> bool {
        self.fields.read().has_point_text
    }
    /// `SetPresentation(thePresentation, thePresentationName)`.
    pub fn set_presentation(&self, presentation: &Shape, name: &Handle<HAsciiString>) {
        let mut fields = self.fields.write();
        (fields.presentation, fields.presentation_name) = (presentation.clone(), name.clone());
    }
    /// `GetPresentation`.
    pub fn get_presentation(&self) -> Shape {
        self.fields.read().presentation.clone()
    }
    /// `GetPresentationName`.
    pub fn get_presentation_name(&self) -> Handle<HAsciiString> {
        self.fields.read().presentation_name.clone()
    }
    /// `DumpJson`.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let f = self.fields.read().clone();
        class_begin(stream, "XCAFDimTolObjects_DatumObject")?;
        field_h_string(stream, "aDatumName", &f.name)?;
        field_integer(stream, "myModifierWithValue", f.modifier_with_value as i32)?;
        field_real(stream, "myValueOfModifier", f.value_of_modifier)?;
        if !f.datum_target.is_null() {
            field_shape(stream, depth, "&myDatumTarget", &f.datum_target)?;
        }
        field_integer(stream, "myPosition", f.position)?;
        field_integer(stream, "myIsDTarget", i32::from(f.is_datum_target))?;
        field_integer(stream, "myIsValidDT", i32::from(f.is_valid_datum_target))?;
        field_integer(stream, "myDTargetType", f.datum_target_type as i32)?;
        field_axis2(stream, depth, "&myAxis", &f.axis)?;
        field_real(stream, "myLength", f.length)?;
        field_real(stream, "myWidth", f.width)?;
        field_integer(stream, "myDatumTargetNumber", f.datum_target_number)?;
        dump_presentation_fields(stream, depth, &f)?;
        for modifier in &f.modifiers {
            field_integer(stream, "aModifier", *modifier as i32)?;
        }
        Ok(())
    }
}

/// The plane, points, presentation and names of `DumpJson`, each when it is set.
fn dump_presentation_fields(
    stream: &mut DumpStream,
    depth: i32,
    f: &DatumFields,
) -> io::Result<()> {
    if f.has_plane {
        field_axis2(stream, depth, "&myPlane", &f.plane)?;
    }
    if f.has_point {
        field_point(stream, depth, "&myPnt", &f.point)?;
    }
    if f.has_point_text {
        field_point(stream, depth, "&myPntText", &f.point_text)?;
    }
    if !f.presentation.is_null() {
        field_shape(stream, depth, "&myPresentation", &f.presentation)?;
    }
    field_h_string(stream, "aSemanticName", &f.semantic_name)?;
    field_h_string(stream, "aPresentationName", &f.presentation_name)
}
