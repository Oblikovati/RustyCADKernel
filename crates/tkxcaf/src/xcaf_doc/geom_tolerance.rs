// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc_GeomTolerance.hxx and .cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XCAFDoc_GeomTolerance`: the attribute that keeps an
//! `XCAFDimTolObjects_GeomToleranceObject` in the children of its label.
use super::dimension::enum_of;
use super::generic_empty_tool::generic_empty_attribute;
use super::object_storage::{
    child, forget_children, generate_shape, get_axis_children, get_real_xyz, get_string,
    integer_array_values, integer_value, named_shape, point_of, real_value, set_axis_children,
    set_integer_array, set_real_xyz, set_string,
};
use super::support::{find, find_or_add, live};
use crate::xcaf_dim_tol_objects::{
    XCAFDimTolObjectsGeomToleranceMatReqModif, XCAFDimTolObjectsGeomToleranceModif,
    XCAFDimTolObjectsGeomToleranceObject, XCAFDimTolObjectsGeomToleranceType,
    XCAFDimTolObjectsGeomToleranceTypeValue, XCAFDimTolObjectsGeomToleranceZoneModif,
    XCAFDimTolObjectsToleranceZoneAffectedPlane,
};
use rustycad_tkcaf::tdata_xtd::{TDataXtdGeometry, TDataXtdPlane};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tklcaf::tdata_std::{TDataStdInteger, TDataStdReal};
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfLabel};
use rustycad_tkmath::gp::Plane3;
use rustycad_tkservice::json_dump::field_dumped;
use std::io;

/// The child labels of a geometric tolerance (`ChildLab`), from `Type` = 1 to `End`.
mod child_lab {
    pub const TYPE: i32 = 1;
    pub const TYPE_OF_VALUE: i32 = 2;
    pub const VALUE: i32 = 3;
    pub const MAT_REQ_MODIF: i32 = 4;
    pub const ZONE_MODIF: i32 = 5;
    pub const VALUE_OF_ZONE_MODIF: i32 = 6;
    pub const MODIFIERS: i32 = 7;
    pub const MAX_VALUE_MODIF: i32 = 8;
    pub const AXIS: [i32; 3] = [9, 10, 11];
    pub const PLANE: [i32; 3] = [12, 13, 14];
    pub const PNT: i32 = 15;
    pub const PNT_TEXT: i32 = 16;
    pub const PRESENTATION: i32 = 17;
    pub const AFFECTED_PLANE: i32 = 18;
    pub const END: i32 = 19;
}
use child_lab as lab;

generic_empty_attribute!(
    /// `XCAFDoc_GeomTolerance`.
    XCAFDocGeomTolerance, "XCAFDoc_GeomTolerance", "58ed092f-44de-11d8-8776-001083004c77", state: (),
    dump: |attribute: &XCAFDocGeomTolerance, stream: &mut DumpStream, depth: i32| -> io::Result<()> {
        rustycad_tkservice::json_dump::class_begin(stream, "XCAFDoc_GeomTolerance")?;
        super::generic_empty_tool::dump_generic_empty_base(attribute, stream, depth)?;
        let object = attribute.get_object().map_err(io::Error::other)?;
        let Some(object) = object.get() else {
            return Ok(());
        };
        field_dumped(stream, depth, "anObject.get()", &|s, d| object.dump_json(s, d))
    }
);

impl XCAFDocGeomTolerance {
    /// `Set(theLabel)`: the tolerance of the label, added when absent.
    pub fn set(label: &TdfLabel) -> Result<Handle<XCAFDocGeomTolerance>, Failure> {
        find_or_add(label, &Self::get_id(), Self::new)
    }
    /// `SetObject(theObject)`: backed up; the semantic name on the label, then the values of
    /// the object on fresh children (the optional ones only when set).
    pub fn set_object(
        &self,
        object: &Handle<XCAFDimTolObjectsGeomToleranceObject>,
    ) -> Result<(), Failure> {
        self.backup()?;
        let object = live(object, "XCAFDoc_GeomTolerance::SetObject")?;
        let label = self.label();
        set_string(&label, &object.get_semantic_name())?;
        forget_children(&label, lab::TYPE, lab::END)?;
        TDataStdInteger::set(&child(&label, lab::TYPE)?, object.get_type() as i32)?;
        if object.get_type_of_value() != XCAFDimTolObjectsGeomToleranceTypeValue::None {
            let value = object.get_type_of_value() as i32;
            TDataStdInteger::set(&child(&label, lab::TYPE_OF_VALUE)?, value)?;
        }
        TDataStdReal::set(&child(&label, lab::VALUE)?, object.get_value())?;
        store_modifiers(&label, object)?;
        store_geometry(&label, object)
    }
}

/// The material requirement, zone and other modifiers of `SetObject`.
fn store_modifiers(
    label: &TdfLabel,
    object: &XCAFDimTolObjectsGeomToleranceObject,
) -> Result<(), Failure> {
    let requirement = object.get_material_requirement_modifier();
    if requirement != XCAFDimTolObjectsGeomToleranceMatReqModif::None {
        TDataStdInteger::set(&child(label, lab::MAT_REQ_MODIF)?, requirement as i32)?;
    }
    if object.get_zone_modifier() != XCAFDimTolObjectsGeomToleranceZoneModif::None {
        TDataStdInteger::set(
            &child(label, lab::ZONE_MODIF)?,
            object.get_zone_modifier() as i32,
        )?;
    }
    if object.get_value_of_zone_modifier() > 0.0 {
        TDataStdReal::set(
            &child(label, lab::VALUE_OF_ZONE_MODIF)?,
            object.get_value_of_zone_modifier(),
        )?;
    }
    let modifiers: Vec<i32> = object.get_modifiers().iter().map(|m| *m as i32).collect();
    if !modifiers.is_empty() {
        set_integer_array(&child(label, lab::MODIFIERS)?, &modifiers)?;
    }
    if object.get_max_value_modifier() > 0.0 {
        TDataStdReal::set(
            &child(label, lab::MAX_VALUE_MODIF)?,
            object.get_max_value_modifier(),
        )?;
    }
    Ok(())
}

/// The axis, plane, points, presentation and affected plane of `SetObject`.
fn store_geometry(
    label: &TdfLabel,
    object: &XCAFDimTolObjectsGeomToleranceObject,
) -> Result<(), Failure> {
    if object.has_axis() {
        set_axis_children(label, lab::AXIS, &object.get_axis())?;
    }
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
    if object.has_affected_plane() {
        let affected = child(label, lab::AFFECTED_PLANE)?;
        TDataStdInteger::set(&affected, object.get_affected_plane_type() as i32)?;
        TDataXtdPlane::set_plane(&affected, &object.get_affected_plane())?;
    }
    Ok(())
}

impl XCAFDocGeomTolerance {
    /// `GetObject`: a new object read back from the children (the native `FindChild` calls
    /// create the children that are missing).
    pub fn get_object(&self) -> Result<Handle<XCAFDimTolObjectsGeomToleranceObject>, Failure> {
        let label = self.label();
        let object = XCAFDimTolObjectsGeomToleranceObject::new();
        object.set_semantic_name(&get_string(&label)?);
        read_values(&label, &object)?;
        read_geometry(&label, &object)?;
        Ok(Handle::new(object))
    }
}

/// The type, value and modifiers of `GetObject`.
fn read_values(
    label: &TdfLabel,
    object: &XCAFDimTolObjectsGeomToleranceObject,
) -> Result<(), Failure> {
    if let Some(value) = integer_value(&child(label, lab::TYPE)?)? {
        object.set_type(enum_of::<XCAFDimTolObjectsGeomToleranceType>(
            value,
            "tolerance type",
        )?);
    }
    if let Some(value) = integer_value(&child(label, lab::TYPE_OF_VALUE)?)? {
        object.set_type_of_value(enum_of::<XCAFDimTolObjectsGeomToleranceTypeValue>(
            value,
            "type of value",
        )?);
    }
    if let Some(value) = real_value(&child(label, lab::VALUE)?)? {
        object.set_value(value);
    }
    if let Some(value) = integer_value(&child(label, lab::MAT_REQ_MODIF)?)? {
        object.set_material_requirement_modifier(enum_of::<
            XCAFDimTolObjectsGeomToleranceMatReqModif,
        >(value, "material modifier")?);
    }
    if let Some(value) = integer_value(&child(label, lab::ZONE_MODIF)?)? {
        object.set_zone_modifier(enum_of::<XCAFDimTolObjectsGeomToleranceZoneModif>(
            value,
            "zone modifier",
        )?);
    }
    if let Some(value) = real_value(&child(label, lab::VALUE_OF_ZONE_MODIF)?)? {
        object.set_value_of_zone_modifier(value);
    }
    if let Some(values) = integer_array_values(&child(label, lab::MODIFIERS)?)? {
        let modifiers = values
            .into_iter()
            .map(|v| enum_of::<XCAFDimTolObjectsGeomToleranceModif>(v, "modifier"))
            .collect::<Result<Vec<_>, _>>()?;
        object.set_modifiers(&modifiers);
    }
    if let Some(value) = real_value(&child(label, lab::MAX_VALUE_MODIF)?)? {
        object.set_max_value_modifier(value);
    }
    Ok(())
}

/// The axis, plane, points, presentation and affected plane of `GetObject`.
fn read_geometry(
    label: &TdfLabel,
    object: &XCAFDimTolObjectsGeomToleranceObject,
) -> Result<(), Failure> {
    if let Some(axis) = get_axis_children(label, lab::AXIS)? {
        object.set_axis(&axis);
    }
    if let Some(plane) = get_axis_children(label, lab::PLANE)? {
        object.set_plane(&plane);
    }
    if let Some(point) = get_real_xyz(&child(label, lab::PNT)?)? {
        object.set_point(&point_of(&point));
    }
    if let Some(point) = get_real_xyz(&child(label, lab::PNT_TEXT)?)? {
        object.set_point_text_attach(&point_of(&point));
    }
    let presentation_label = child(label, lab::PRESENTATION)?;
    if let Some(presentation) = named_shape(&presentation_label)?.filter(|s| !s.is_null()) {
        object.set_presentation(&presentation, &get_string(&presentation_label)?);
    }
    read_affected_plane(&child(label, lab::AFFECTED_PLANE)?, object)
}

/// The affected plane: the plane of the child's `TDataXtd_Plane` and its integer type
/// (natively a missing integer is a null dereference).
fn read_affected_plane(
    affected: &TdfLabel,
    object: &XCAFDimTolObjectsGeomToleranceObject,
) -> Result<(), Failure> {
    let plane_attribute = find::<TDataXtdPlane>(affected, &TDataXtdPlane::get_id())?;
    if plane_attribute.is_null() {
        return Ok(());
    }
    let mut plane = Plane3::default();
    TDataXtdGeometry::plane_of_label(affected, &mut plane)?;
    let kind = integer_value(affected)?.ok_or_else(|| {
        rustycad_tkernel::standard::exceptions::null_object(
            "XCAFDoc_GeomTolerance::GetObject: affected plane without its TDataStd_Integer type",
        )
    })?;
    let kind = enum_of::<XCAFDimTolObjectsToleranceZoneAffectedPlane>(kind, "affected plane type")?;
    object.set_affected_plane_with_type(&plane, kind);
    Ok(())
}
