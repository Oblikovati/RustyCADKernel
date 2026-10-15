// Rust port of DataExchange/TKXCAF/XCAFDimTolObjects/XCAFDimTolObjects_GeomToleranceObject.hxx
// and .cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XCAFDimTolObjects_GeomToleranceObject`: the description of a geometric tolerance (type,
//! value, modifiers, zone, axis, affected plane and presentation).
use super::{
    XCAFDimTolObjectsGeomToleranceMatReqModif, XCAFDimTolObjectsGeomToleranceModif,
    XCAFDimTolObjectsGeomToleranceType, XCAFDimTolObjectsGeomToleranceTypeValue,
    XCAFDimTolObjectsGeomToleranceZoneModif, XCAFDimTolObjectsToleranceZoneAffectedPlane,
};
use crate::json::{field_axis2, field_h_string, field_plane, field_point, field_shape};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::HAsciiString;
use rustycad_tkmath::gp::{Axis2, Plane3, Point3};
use rustycad_tkservice::json_dump::{class_begin, field_integer, field_real};
use std::io;

/// The fields of `XCAFDimTolObjects_GeomToleranceObject`. The native default constructor
/// leaves the type, value, modifiers and their values uninitialized; the OCCT allocator
/// clears memory, so they read zero, as here.
#[derive(Clone, Default)]
struct GeomToleranceFields {
    kind: XCAFDimTolObjectsGeomToleranceType,
    type_of_value: XCAFDimTolObjectsGeomToleranceTypeValue,
    value: f64,
    material_requirement: XCAFDimTolObjectsGeomToleranceMatReqModif,
    zone_modifier: XCAFDimTolObjectsGeomToleranceZoneModif,
    value_of_zone_modifier: f64,
    modifiers: Vec<XCAFDimTolObjectsGeomToleranceModif>,
    max_value_modifier: f64,
    axis: Axis2,
    has_axis: bool,
    plane: Axis2,
    point: Point3,
    point_text: Point3,
    has_plane: bool,
    has_point: bool,
    has_point_text: bool,
    presentation: Shape,
    semantic_name: Handle<HAsciiString>,
    presentation_name: Handle<HAsciiString>,
    affected_plane_type: XCAFDimTolObjectsToleranceZoneAffectedPlane,
    affected_plane: Plane3,
}

/// `XCAFDimTolObjects_GeomToleranceObject`.
///
/// ```
/// use rustycad_tkxcaf::xcaf_dim_tol_objects::XCAFDimTolObjectsGeomToleranceObject;
/// let tolerance = XCAFDimTolObjectsGeomToleranceObject::new();
/// tolerance.set_value(0.5);
/// assert_eq!(tolerance.get_value(), 0.5);
/// assert!(!tolerance.has_affected_plane());
/// ```
#[derive(Default)]
pub struct XCAFDimTolObjectsGeomToleranceObject {
    transient: StandardTransient,
    fields: SharedState<GeomToleranceFields>,
}
rustycad_tkernel::impl_transient!(
    XCAFDimTolObjectsGeomToleranceObject,
    "XCAFDimTolObjects_GeomToleranceObject",
    StandardTransient,
    transient
);

impl XCAFDimTolObjectsGeomToleranceObject {
    /// `XCAFDimTolObjects_GeomToleranceObject()`: no axis, plane or points, no affected plane.
    pub fn new() -> Self {
        Self::default()
    }
    /// `XCAFDimTolObjects_GeomToleranceObject(theObj)`: a copy of every field (a null object
    /// gives a default object).
    pub fn from_object(object: &Handle<XCAFDimTolObjectsGeomToleranceObject>) -> Self {
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
    /// `SetType`.
    pub fn set_type(&self, kind: XCAFDimTolObjectsGeomToleranceType) {
        self.fields.write().kind = kind;
    }
    /// `GetType`.
    pub fn get_type(&self) -> XCAFDimTolObjectsGeomToleranceType {
        self.fields.read().kind
    }
    /// `SetTypeOfValue`.
    pub fn set_type_of_value(&self, kind: XCAFDimTolObjectsGeomToleranceTypeValue) {
        self.fields.write().type_of_value = kind;
    }
    /// `GetTypeOfValue`.
    pub fn get_type_of_value(&self) -> XCAFDimTolObjectsGeomToleranceTypeValue {
        self.fields.read().type_of_value
    }
    /// `SetValue`.
    pub fn set_value(&self, value: f64) {
        self.fields.write().value = value;
    }
    /// `GetValue`.
    pub fn get_value(&self) -> f64 {
        self.fields.read().value
    }
    /// `SetMaterialRequirementModifier`.
    pub fn set_material_requirement_modifier(
        &self,
        modifier: XCAFDimTolObjectsGeomToleranceMatReqModif,
    ) {
        self.fields.write().material_requirement = modifier;
    }
    /// `GetMaterialRequirementModifier`.
    pub fn get_material_requirement_modifier(&self) -> XCAFDimTolObjectsGeomToleranceMatReqModif {
        self.fields.read().material_requirement
    }
    /// `SetZoneModifier`.
    pub fn set_zone_modifier(&self, modifier: XCAFDimTolObjectsGeomToleranceZoneModif) {
        self.fields.write().zone_modifier = modifier;
    }
    /// `GetZoneModifier`.
    pub fn get_zone_modifier(&self) -> XCAFDimTolObjectsGeomToleranceZoneModif {
        self.fields.read().zone_modifier
    }
    /// `SetValueOfZoneModifier`.
    pub fn set_value_of_zone_modifier(&self, value: f64) {
        self.fields.write().value_of_zone_modifier = value;
    }
    /// `GetValueOfZoneModifier`.
    pub fn get_value_of_zone_modifier(&self) -> f64 {
        self.fields.read().value_of_zone_modifier
    }
    /// `SetModifiers`.
    pub fn set_modifiers(&self, modifiers: &[XCAFDimTolObjectsGeomToleranceModif]) {
        self.fields.write().modifiers = modifiers.to_vec();
    }
    /// `AddModifier`: appends a modifier.
    pub fn add_modifier(&self, modifier: XCAFDimTolObjectsGeomToleranceModif) {
        self.fields.write().modifiers.push(modifier);
    }
    /// `GetModifiers`: a copy of the modifiers.
    pub fn get_modifiers(&self) -> Vec<XCAFDimTolObjectsGeomToleranceModif> {
        self.fields.read().modifiers.clone()
    }
    /// `SetMaxValueModifier`.
    pub fn set_max_value_modifier(&self, value: f64) {
        self.fields.write().max_value_modifier = value;
    }
    /// `GetMaxValueModifier`.
    pub fn get_max_value_modifier(&self) -> f64 {
        self.fields.read().max_value_modifier
    }
    /// `SetAxis`.
    pub fn set_axis(&self, axis: &Axis2) {
        let mut fields = self.fields.write();
        (fields.axis, fields.has_axis) = (*axis, true);
    }
    /// `GetAxis`.
    pub fn get_axis(&self) -> Axis2 {
        self.fields.read().axis
    }
    /// `HasAxis`.
    pub fn has_axis(&self) -> bool {
        self.fields.read().has_axis
    }
}

/// The presentation members (inline natively).
impl XCAFDimTolObjectsGeomToleranceObject {
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
    /// `HasAffectedPlane`: an affected plane type other than `None`.
    pub fn has_affected_plane(&self) -> bool {
        self.get_affected_plane_type() != XCAFDimTolObjectsToleranceZoneAffectedPlane::None
    }
    /// `GetAffectedPlaneType`.
    pub fn get_affected_plane_type(&self) -> XCAFDimTolObjectsToleranceZoneAffectedPlane {
        self.fields.read().affected_plane_type
    }
    /// `SetAffectedPlaneType`.
    pub fn set_affected_plane_type(&self, kind: XCAFDimTolObjectsToleranceZoneAffectedPlane) {
        self.fields.write().affected_plane_type = kind;
    }
    /// `SetAffectedPlane(thePlane)`.
    pub fn set_affected_plane(&self, plane: &Plane3) {
        self.fields.write().affected_plane = *plane;
    }
    /// `SetAffectedPlane(thePlane, theType)`.
    pub fn set_affected_plane_with_type(
        &self,
        plane: &Plane3,
        kind: XCAFDimTolObjectsToleranceZoneAffectedPlane,
    ) {
        let mut fields = self.fields.write();
        (fields.affected_plane_type, fields.affected_plane) = (kind, *plane);
    }
    /// `GetAffectedPlane`.
    pub fn get_affected_plane(&self) -> Plane3 {
        self.fields.read().affected_plane
    }
    /// `DumpJson`.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let f = self.fields.read().clone();
        class_begin(stream, "XCAFDimTolObjects_GeomToleranceObject")?;
        field_integer(stream, "myType", f.kind as i32)?;
        field_integer(stream, "myTypeOfValue", f.type_of_value as i32)?;
        field_real(stream, "myValue", f.value)?;
        field_integer(stream, "myMatReqModif", f.material_requirement as i32)?;
        field_integer(stream, "myZoneModif", f.zone_modifier as i32)?;
        field_real(stream, "myValueOfZoneModif", f.value_of_zone_modifier)?;
        field_real(stream, "myMaxValueModif", f.max_value_modifier)?;
        dump_geometry_fields(stream, depth, &f)?;
        for modifier in &f.modifiers {
            field_integer(stream, "aModifier", *modifier as i32)?;
        }
        Ok(())
    }
}

/// The axis, plane, points, presentation, names and affected plane of `DumpJson`.
fn dump_geometry_fields(
    stream: &mut DumpStream,
    depth: i32,
    f: &GeomToleranceFields,
) -> io::Result<()> {
    if f.has_axis {
        field_axis2(stream, depth, "&myAxis", &f.axis)?;
    }
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
    field_h_string(stream, "aPresentationName", &f.presentation_name)?;
    field_plane(stream, depth, "&myAffectedPlane", &f.affected_plane)
}
