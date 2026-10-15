// Rust port of DataExchange/TKXCAF/XCAFDimTolObjects/XCAFDimTolObjects_DimensionObject.hxx and
// .cxx (the inline connection, plane, presentation and description members and DumpJson) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The presentation members of `XCAFDimTolObjects_DimensionObject`.
use super::XCAFDimTolObjectsDimensionObject;
use super::dimension_object::{DimensionFields, array_values};
use crate::json::{field_axis2, field_direction, field_h_string, field_point, field_shape};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::tcollection::HAsciiString;
use rustycad_tkmath::gp::{Axis2, Point3};
use rustycad_tkservice::json_dump::{class_begin, field_integer, field_real};
use std::io;

/// The text point, plane and the two connections.
impl XCAFDimTolObjectsDimensionObject {
    /// `SetPointTextAttach`.
    pub fn set_point_text_attach(&self, point: &Point3) {
        let mut fields = self.fields.write();
        (fields.point_text, fields.has_point_text) = (*point, true);
    }
    /// `GetPointTextAttach`.
    pub fn get_point_text_attach(&self) -> Point3 {
        self.fields.read().point_text
    }
    /// `HasTextPoint`.
    pub fn has_text_point(&self) -> bool {
        self.fields.read().has_point_text
    }
    /// `SetPlane`.
    pub fn set_plane(&self, plane: &Axis2) {
        let mut fields = self.fields.write();
        (fields.plane, fields.has_plane) = (*plane, true);
    }
    /// `GetPlane`.
    pub fn get_plane(&self) -> Axis2 {
        self.fields.read().plane
    }
    /// `HasPlane`.
    pub fn has_plane(&self) -> bool {
        self.fields.read().has_plane
    }
    /// `HasPoint`: the first connection is set.
    pub fn has_point(&self) -> bool {
        self.fields.read().has_connection1
    }
    /// `HasPoint2`: the second connection is set.
    pub fn has_point2(&self) -> bool {
        self.fields.read().has_connection2
    }
    /// `IsPointConnection`.
    pub fn is_point_connection(&self) -> bool {
        self.fields.read().connection_is_point1
    }
    /// `IsPointConnection2`.
    pub fn is_point_connection2(&self) -> bool {
        self.fields.read().connection_is_point2
    }
    /// `SetPoint`: the location of the first connection, a point connection.
    pub fn set_point(&self, point: &Point3) {
        let mut fields = self.fields.write();
        fields.connection1.set_location(*point);
        (fields.has_connection1, fields.connection_is_point1) = (true, true);
    }
    /// `SetPoint2`: the location of the second connection, a point connection.
    pub fn set_point2(&self, point: &Point3) {
        let mut fields = self.fields.write();
        fields.connection2.set_location(*point);
        (fields.has_connection2, fields.connection_is_point2) = (true, true);
    }
    /// `SetConnectionAxis`: the first connection, an axis connection.
    pub fn set_connection_axis(&self, axis: &Axis2) {
        let mut fields = self.fields.write();
        fields.connection1 = *axis;
        (fields.has_connection1, fields.connection_is_point1) = (true, false);
    }
    /// `SetConnectionAxis2`: the second connection, an axis connection.
    pub fn set_connection_axis2(&self, axis: &Axis2) {
        let mut fields = self.fields.write();
        fields.connection2 = *axis;
        (fields.has_connection2, fields.connection_is_point2) = (true, false);
    }
    /// `GetPoint`: the location of the first connection.
    pub fn get_point(&self) -> Point3 {
        self.fields.read().connection1.location()
    }
    /// `GetPoint2`: the location of the second connection.
    pub fn get_point2(&self) -> Point3 {
        self.fields.read().connection2.location()
    }
    /// `GetConnectionAxis`.
    pub fn get_connection_axis(&self) -> Axis2 {
        self.fields.read().connection1
    }
    /// `GetConnectionAxis2`.
    pub fn get_connection_axis2(&self) -> Axis2 {
        self.fields.read().connection2
    }
    /// `GetConnectionName`.
    pub fn get_connection_name(&self) -> Handle<HAsciiString> {
        self.fields.read().connection_name1.clone()
    }
    /// `GetConnectionName2`.
    pub fn get_connection_name2(&self) -> Handle<HAsciiString> {
        self.fields.read().connection_name2.clone()
    }
    /// `SetConnectionName`.
    pub fn set_connection_name(&self, name: &Handle<HAsciiString>) {
        self.fields.write().connection_name1 = name.clone();
    }
    /// `SetConnectionName2`.
    pub fn set_connection_name2(&self, name: &Handle<HAsciiString>) {
        self.fields.write().connection_name2 = name.clone();
    }
}

/// The presentation and the descriptions.
impl XCAFDimTolObjectsDimensionObject {
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
    /// `HasDescriptions`.
    pub fn has_descriptions(&self) -> bool {
        !self.fields.read().descriptions.is_empty()
    }
    /// `NbDescriptions`.
    pub fn nb_descriptions(&self) -> i32 {
        self.fields.read().descriptions.len() as i32
    }
    /// The 0-based position of a description number, `None` outside the descriptions.
    fn description_index(&self, number: i32) -> Option<usize> {
        usize::try_from(number)
            .ok()
            .filter(|index| *index < self.fields.read().descriptions.len())
    }
    /// `GetDescription(theNumber)` (0-based): a new empty string outside the descriptions.
    pub fn get_description(&self, number: i32) -> Handle<HAsciiString> {
        match self.description_index(number) {
            Some(index) => self.fields.read().descriptions[index].clone(),
            None => Handle::new(HAsciiString::new("")),
        }
    }
    /// `GetDescriptionName(theNumber)` (0-based): a new empty string outside the
    /// descriptions.
    pub fn get_description_name(&self, number: i32) -> Handle<HAsciiString> {
        match self.description_index(number) {
            Some(index) => self.fields.read().description_names[index].clone(),
            None => Handle::new(HAsciiString::new("")),
        }
    }
    /// `RemoveDescription(theNumber)`. Natively the copy loop reads the bounds of the new,
    /// empty array, so the descriptions from `theNumber` on are all dropped; the port keeps
    /// that.
    pub fn remove_description(&self, number: i32) {
        let Some(index) = self.description_index(number) else {
            return;
        };
        let mut fields = self.fields.write();
        fields.descriptions.truncate(index);
        fields.description_names.truncate(index);
    }
    /// `AddDescription(theDescription, theName)`.
    pub fn add_description(&self, description: &Handle<HAsciiString>, name: &Handle<HAsciiString>) {
        let mut fields = self.fields.write();
        fields.descriptions.push(description.clone());
        fields.description_names.push(name.clone());
    }
    /// `DumpJson`.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let f = self.fields.read().clone();
        class_begin(stream, "XCAFDimTolObjects_DimensionObject")?;
        field_integer(stream, "myType", f.kind as i32)?;
        for value in array_values(&f.values).unwrap_or_default() {
            field_real(stream, "aValue", value)?;
        }
        field_integer(stream, "myQualifier", f.qualifier as i32)?;
        field_integer(stream, "myIsHole", i32::from(f.is_hole))?;
        field_integer(stream, "myFormVariance", f.form_variance as i32)?;
        field_integer(stream, "myGrade", f.grade as i32)?;
        field_integer(stream, "myL", f.left)?;
        field_integer(stream, "myR", f.right)?;
        if !f.path.is_null() {
            field_shape(stream, depth, "&myPath", &f.path)?;
        }
        field_direction(stream, depth, "&myDir", &f.direction)?;
        dump_connections(stream, depth, &f)?;
        for modifier in &f.modifiers {
            field_integer(stream, "aModifier", *modifier as i32)?;
        }
        Ok(())
    }
}

/// The connections, plane, text point, presentation, names and descriptions of `DumpJson`.
fn dump_connections(stream: &mut DumpStream, depth: i32, f: &DimensionFields) -> io::Result<()> {
    if f.has_connection1 {
        field_axis2(stream, depth, "&myConnection1", &f.connection1)?;
    }
    if f.has_connection2 {
        field_axis2(stream, depth, "&myConnection2", &f.connection2)?;
    }
    if f.has_plane {
        field_axis2(stream, depth, "&myPlane", &f.plane)?;
    }
    if f.has_point_text {
        field_point(stream, depth, "&myPntText", &f.point_text)?;
    }
    if !f.presentation.is_null() {
        field_shape(stream, depth, "&myPresentation", &f.presentation)?;
    }
    field_h_string(stream, "aSemanticName", &f.semantic_name)?;
    field_h_string(stream, "aPresentationName", &f.presentation_name)?;
    for description in &f.descriptions {
        field_h_string(stream, "aDescription", description)?;
    }
    for name in &f.description_names {
        field_h_string(stream, "aDescriptionName", name)?;
    }
    Ok(())
}
