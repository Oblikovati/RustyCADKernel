// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc_Datum.hxx and .cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XCAFDoc_Datum`: a datum (name, description, identification) and the
//! `XCAFDimTolObjects_DatumObject` it keeps in the children of its label.
use super::dimension::enum_of;
use super::object_storage::{
    child, forget_children, generate_shape, get_axis_children, get_real_xyz, get_string,
    integer_array_values, integer_value, named_shape, named_shape_get, real_array_values,
    real_value, set_axis_children, set_integer_array, set_real_xyz, set_string,
};
use super::support::{downcast, find_or_add, live};
use crate::json::field_raw_pointer;
use crate::xcaf_dim_tol_objects::{
    XCAFDimTolObjectsDatumModifWithValue, XCAFDimTolObjectsDatumObject,
    XCAFDimTolObjectsDatumSingleModif, XCAFDimTolObjectsDatumTargetType,
};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Guid, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::HAsciiString;
use rustycad_tklcaf::guid_from_text;
use rustycad_tklcaf::json::dump_attribute_json_base;
use rustycad_tklcaf::tdata_std::{TDataStdAsciiString, TDataStdInteger, TDataStdReal};
use rustycad_tklcaf::tdf::{
    TdfAttribute, TdfAttributeBase, TdfAttributeState, TdfLabel, TdfRelocationTable,
};
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::json_dump::{class_begin, field_dumped};
use std::io;

/// `XCAFDoc_Datum::GetID()`.
const DATUM_ID: Guid = guid_from_text("58ed092e-44de-11d8-8776-001083004c77");

/// The child labels of a datum (`ChildLab`), from `Name` = 1 to `End`.
mod child_lab {
    pub const NAME: i32 = 1;
    pub const POSITION: i32 = 2;
    pub const MODIFIERS: i32 = 3;
    pub const MODIFIER_WITH_VALUE: i32 = 4;
    pub const IS_D_TARGET: i32 = 5;
    pub const D_TARGET_TYPE: i32 = 6;
    pub const AXIS: [i32; 3] = [7, 8, 9];
    pub const D_TARGET_LENGTH: i32 = 10;
    pub const D_TARGET_WIDTH: i32 = 11;
    pub const D_TARGET_NUMBER: i32 = 12;
    pub const DATUM_TARGET: i32 = 13;
    pub const PLANE: [i32; 3] = [14, 15, 16];
    pub const PNT: i32 = 17;
    pub const PNT_TEXT: i32 = 18;
    pub const PRESENTATION: i32 = 19;
    pub const END: i32 = 20;
}
use child_lab as lab;

/// The fields of `XCAFDoc_Datum`.
#[derive(Clone, Default)]
struct DatumFields {
    name: Handle<HAsciiString>,
    description: Handle<HAsciiString>,
    identification: Handle<HAsciiString>,
}

/// `XCAFDoc_Datum`.
#[derive(Default)]
pub struct XCAFDocDatum {
    transient: StandardTransient,
    base: TdfAttributeBase,
    fields: SharedState<DatumFields>,
}
rustycad_tkernel::impl_transient!(XCAFDocDatum, "XCAFDoc_Datum", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

impl XCAFDocDatum {
    /// `XCAFDoc_Datum()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `GetID`.
    pub fn get_id() -> Guid {
        DATUM_ID
    }
    /// `Set(theLabel, theName, theDescription, theIdentification)`.
    pub fn set_with_names(
        label: &TdfLabel,
        name: &Handle<HAsciiString>,
        description: &Handle<HAsciiString>,
        identification: &Handle<HAsciiString>,
    ) -> Result<Handle<XCAFDocDatum>, Failure> {
        let datum = Self::set(label)?;
        live(&datum, "XCAFDoc_Datum::Set")?.set_names(name, description, identification)?;
        Ok(datum)
    }
    /// `Set(theLabel)`: the datum of the label, added when absent.
    pub fn set(label: &TdfLabel) -> Result<Handle<XCAFDocDatum>, Failure> {
        find_or_add(label, &DATUM_ID, Self::new)
    }
    /// `Set(theName, theDescription, theIdentification)`: backed up.
    pub fn set_names(
        &self,
        name: &Handle<HAsciiString>,
        description: &Handle<HAsciiString>,
        identification: &Handle<HAsciiString>,
    ) -> Result<(), Failure> {
        self.backup()?;
        *self.fields.write() = DatumFields {
            name: name.clone(),
            description: description.clone(),
            identification: identification.clone(),
        };
        Ok(())
    }
    /// `GetName`: the name, or a new empty string when there is none.
    pub fn get_name(&self) -> Handle<HAsciiString> {
        let name = self.fields.read().name.clone();
        if name.is_null() {
            return Handle::new(HAsciiString::new(""));
        }
        name
    }
    /// `GetDescription`.
    pub fn get_description(&self) -> Handle<HAsciiString> {
        self.fields.read().description.clone()
    }
    /// `GetIdentification`.
    pub fn get_identification(&self) -> Handle<HAsciiString> {
        self.fields.read().identification.clone()
    }
    /// `SetObject(theObject)`: backed up; the semantic name on the label, then the values of
    /// the object on fresh children.
    pub fn set_object(&self, object: &Handle<XCAFDimTolObjectsDatumObject>) -> Result<(), Failure> {
        self.backup()?;
        let object = live(object, "XCAFDoc_Datum::SetObject")?;
        let label = self.label();
        set_string(&label, &object.get_semantic_name())?;
        forget_children(&label, lab::NAME, lab::END)?;
        let name = object.get_name();
        if let Some(name) = name.get().filter(|name| !name.is_empty()) {
            TDataStdAsciiString::set(&child(&label, lab::NAME)?, &name.string())?;
        }
        TDataStdInteger::set(&child(&label, lab::POSITION)?, object.get_position())?;
        store_modifiers(&label, object)?;
        store_target(&label, object)?;
        store_presentation(&label, object)
    }
}

/// The modifiers and the modifier with value of `SetObject`.
fn store_modifiers(label: &TdfLabel, object: &XCAFDimTolObjectsDatumObject) -> Result<(), Failure> {
    let modifiers: Vec<i32> = object.get_modifiers().iter().map(|m| *m as i32).collect();
    if !modifiers.is_empty() {
        set_integer_array(&child(label, lab::MODIFIERS)?, &modifiers)?;
    }
    let (modifier, value) = object.get_modifier_with_value();
    if modifier != XCAFDimTolObjectsDatumModifWithValue::None {
        let with_value = child(label, lab::MODIFIER_WITH_VALUE)?;
        TDataStdInteger::set(&with_value, modifier as i32)?;
        TDataStdReal::set(&with_value, value)?;
    }
    Ok(())
}

/// The datum target of `SetObject`: an area shape, or the axis, length, width; and its
/// number.
fn store_target(label: &TdfLabel, object: &XCAFDimTolObjectsDatumObject) -> Result<(), Failure> {
    TDataStdInteger::set(
        &child(label, lab::IS_D_TARGET)?,
        i32::from(object.is_datum_target()),
    )?;
    if !object.is_datum_target() {
        return Ok(());
    }
    let kind = object.get_datum_target_type();
    TDataStdInteger::set(&child(label, lab::D_TARGET_TYPE)?, kind as i32)?;
    if kind == XCAFDimTolObjectsDatumTargetType::Area {
        let target = object.get_datum_target();
        if !target.is_null() {
            generate_shape(&child(label, lab::DATUM_TARGET)?, &target)?;
        }
    } else if object.has_datum_target_params() {
        set_axis_children(label, lab::AXIS, &object.get_datum_target_axis())?;
        if kind != XCAFDimTolObjectsDatumTargetType::Point {
            TDataStdReal::set(
                &child(label, lab::D_TARGET_LENGTH)?,
                object.get_datum_target_length(),
            )?;
            if kind == XCAFDimTolObjectsDatumTargetType::Rectangle {
                TDataStdReal::set(
                    &child(label, lab::D_TARGET_WIDTH)?,
                    object.get_datum_target_width(),
                )?;
            }
        }
    }
    TDataStdInteger::set(
        &child(label, lab::D_TARGET_NUMBER)?,
        object.get_datum_target_number(),
    )?;
    Ok(())
}

/// The plane, points and presentation of `SetObject`.
fn store_presentation(
    label: &TdfLabel,
    object: &XCAFDimTolObjectsDatumObject,
) -> Result<(), Failure> {
    if object.has_plane() {
        set_axis_children(label, lab::PLANE, &object.get_plane())?;
    }
    if object.has_point() {
        set_real_xyz(&child(label, lab::PNT)?, &object.get_point().xyz())?;
    }
    if object.has_point_text() {
        set_real_xyz(
            &child(label, lab::PNT_TEXT)?,
            &object.get_point_text_attach().xyz(),
        )?;
    }
    let presentation = object.get_presentation();
    if !presentation.is_null() {
        let presentation_label = child(label, lab::PRESENTATION)?;
        generate_shape(&presentation_label, &presentation)?;
        set_string(&presentation_label, &object.get_presentation_name())?;
    }
    Ok(())
}

impl XCAFDocDatum {
    /// `GetObject`: a new object read back from the children (the native `FindChild` calls
    /// create the children that are missing). Natively the X of the point is read from the
    /// plane location array; the port keeps that (`Standard_NullObject` without a plane).
    pub fn get_object(&self) -> Result<Handle<XCAFDimTolObjectsDatumObject>, Failure> {
        let label = self.label();
        let object = XCAFDimTolObjectsDatumObject::new();
        object.set_semantic_name(&get_string(&label)?);
        read_identity(&label, &object)?;
        read_presentation_points(&label, &object)?;
        let Some(is_target) = integer_value(&child(&label, lab::IS_D_TARGET)?)? else {
            return Ok(Handle::new(object));
        };
        object.set_is_datum_target(is_target != 0);
        if is_target != 0 {
            read_target(&label, &object)?;
        }
        let presentation_label = child(&label, lab::PRESENTATION)?;
        if let Some(presentation) = named_shape(&presentation_label)?.filter(|s| !s.is_null()) {
            object.set_presentation(&presentation, &get_string(&presentation_label)?);
        }
        Ok(Handle::new(object))
    }
}

/// The name, modifiers, modifier with value and position of `GetObject`.
fn read_identity(label: &TdfLabel, object: &XCAFDimTolObjectsDatumObject) -> Result<(), Failure> {
    let mut name: Handle<TDataStdAsciiString> = Handle::default();
    if child(label, lab::NAME)?.find_attribute_as(&TDataStdAsciiString::get_id(), &mut name)? {
        let text = live(&name, "XCAFDoc_Datum::GetObject")?.get();
        object.set_name(&Handle::new(HAsciiString::from_string(&text)));
    }
    if let Some(values) = integer_array_values(&child(label, lab::MODIFIERS)?)? {
        let modifiers = values
            .into_iter()
            .map(|v| enum_of::<XCAFDimTolObjectsDatumSingleModif>(v, "datum modifier"))
            .collect::<Result<Vec<_>, _>>()?;
        object.set_modifiers(&modifiers);
    }
    let with_value = child(label, lab::MODIFIER_WITH_VALUE)?;
    if let (Some(modifier), Some(value)) = (integer_value(&with_value)?, real_value(&with_value)?) {
        let modifier =
            enum_of::<XCAFDimTolObjectsDatumModifWithValue>(modifier, "modifier with value")?;
        object.set_modifier_with_value(modifier, value);
    }
    if let Some(position) = integer_value(&child(label, lab::POSITION)?)? {
        object.set_position(position);
    }
    Ok(())
}

/// The plane, the point (its X from the plane location, as natively) and the text point of
/// `GetObject`.
fn read_presentation_points(
    label: &TdfLabel,
    object: &XCAFDimTolObjectsDatumObject,
) -> Result<(), Failure> {
    if let Some(plane) = get_axis_children(label, lab::PLANE)? {
        object.set_plane(&plane);
    }
    if let Some(point) = real_array_values(&child(label, lab::PNT)?)?.filter(|v| v.len() == 3) {
        let plane_location = real_array_values(&child(label, lab::PLANE[0])?)?.ok_or_else(|| {
            rustycad_tkernel::standard::exceptions::null_object(
                "XCAFDoc_Datum::GetObject: the point reads the plane location array, which is null",
            )
        })?;
        let x = plane_location.first().copied().unwrap_or_default();
        object.set_point(&Point3::new(x, point[1], point[2]));
    }
    if let Some(point) = get_real_xyz(&child(label, lab::PNT_TEXT)?)? {
        object.set_point_text_attach(&Point3::new(point.x(), point.y(), point.z()));
    }
    Ok(())
}

/// The datum target of `GetObject`: type, area shape or axis, length and width, and the
/// number (0 without one).
fn read_target(label: &TdfLabel, object: &XCAFDimTolObjectsDatumObject) -> Result<(), Failure> {
    if let Some(kind) = integer_value(&child(label, lab::D_TARGET_TYPE)?)? {
        let kind = enum_of::<XCAFDimTolObjectsDatumTargetType>(kind, "datum target type")?;
        object.set_datum_target_type(kind);
        if kind == XCAFDimTolObjectsDatumTargetType::Area {
            if let Some(target) = named_shape_get(&child(label, lab::DATUM_TARGET)?)? {
                object.set_datum_target(&target);
            }
        } else {
            read_target_parameters(label, object, kind)?;
        }
    }
    let number = integer_value(&child(label, lab::D_TARGET_NUMBER)?)?.unwrap_or(0);
    object.set_datum_target_number(number);
    Ok(())
}

/// The axis, length and width of a non-area datum target.
fn read_target_parameters(
    label: &TdfLabel,
    object: &XCAFDimTolObjectsDatumObject,
    kind: XCAFDimTolObjectsDatumTargetType,
) -> Result<(), Failure> {
    if let Some(axis) = get_axis_children(label, lab::AXIS)? {
        object.set_datum_target_axis(&axis);
    }
    if kind == XCAFDimTolObjectsDatumTargetType::Point {
        return Ok(());
    }
    if let Some(length) = real_value(&child(label, lab::D_TARGET_LENGTH)?)? {
        object.set_datum_target_length(length);
    }
    if kind == XCAFDimTolObjectsDatumTargetType::Rectangle
        && let Some(width) = real_value(&child(label, lab::D_TARGET_WIDTH)?)?
    {
        object.set_datum_target_width(width);
    }
    Ok(())
}

impl TdfAttribute for XCAFDocDatum {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        DATUM_ID
    }
    /// `Restore`: the name (`GetName`, so never null), description and identification.
    fn restore(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        let other = downcast::<XCAFDocDatum>(with, "XCAFDoc_Datum::Restore")?;
        let other = live(&other, "XCAFDoc_Datum::Restore")?;
        *self.fields.write() = DatumFields {
            name: other.get_name(),
            description: other.get_description(),
            identification: other.get_identification(),
        };
        Ok(())
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::new()).cast()
    }
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        _table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        let target = downcast::<XCAFDocDatum>(into, "XCAFDoc_Datum::Paste")?;
        let f = self.fields.read().clone();
        live(&target, "XCAFDoc_Datum::Paste")?.set_names(&f.name, &f.description, &f.identification)
    }
    /// `DumpJson`: the three strings as pointers (`myName.get()`, as natively), then the
    /// dump of the object.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "XCAFDoc_Datum")?;
        dump_attribute_json_base(self, stream, depth)?;
        let f = self.fields.read().clone();
        field_raw_pointer(stream, "myName.get()", f.name.object_address())?;
        field_raw_pointer(
            stream,
            "myDescription.get()",
            f.description.object_address(),
        )?;
        field_raw_pointer(
            stream,
            "myIdentification.get()",
            f.identification.object_address(),
        )?;
        let object = self.get_object().map_err(io::Error::other)?;
        let Some(object) = object.get() else {
            return Ok(());
        };
        field_dumped(stream, depth, "anObject.get()", &|s, d| {
            object.dump_json(s, d)
        })
    }
}
