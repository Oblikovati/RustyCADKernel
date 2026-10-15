// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc_Dimension.hxx and .cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XCAFDoc_Dimension`: the attribute that keeps an `XCAFDimTolObjects_DimensionObject` in the
//! children of its label (one value per child: type, values, qualifiers, class, decimal
//! places, modifiers, path, direction, connections, plane, text point, presentation and
//! descriptions).
use super::generic_empty_tool::generic_empty_attribute;
use super::object_storage::{
    child, forget_children, generate_shape, get_real_xyz, get_string, integer_array_values,
    integer_value, named_shape, named_shape_get, point_of, real_array_values, set_integer_array,
    set_real_array, set_real_xyz, set_string,
};
use super::support::{find_or_add, live};
use crate::NativeEnum;
use crate::xcaf_dim_tol_objects::{
    XCAFDimTolObjectsAngularQualifier, XCAFDimTolObjectsDimensionFormVariance,
    XCAFDimTolObjectsDimensionGrade, XCAFDimTolObjectsDimensionModif,
    XCAFDimTolObjectsDimensionObject, XCAFDimTolObjectsDimensionQualifier,
    XCAFDimTolObjectsDimensionType, real_array,
};
use rustycad_tkbrep::topods;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString, HAsciiString};
use rustycad_tklcaf::tdata_std::{TDataStdExtStringArray, TDataStdInteger};
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfLabel};
use rustycad_tkmath::gp::{Axis2, Direction3};
use rustycad_tkservice::json_dump::field_dumped;
use std::io;

/// The child labels of a dimension (`ChildLab`), from `Type` = 1 to `End`.
mod child_lab {
    pub const TYPE: i32 = 1;
    pub const VALUE: i32 = 2;
    pub const QUALIFIER: i32 = 3;
    pub const CLASS: i32 = 4;
    pub const DEC: i32 = 5;
    pub const MODIFIERS: i32 = 6;
    pub const PATH: i32 = 7;
    pub const DIR: i32 = 8;
    pub const PNT1: i32 = 9;
    pub const PNT2: i32 = 10;
    pub const PLANE_LOC: i32 = 11;
    pub const PLANE_N: i32 = 12;
    pub const PLANE_REF: i32 = 13;
    pub const PNT_TEXT: i32 = 14;
    pub const PRESENTATION: i32 = 15;
    pub const DESCRIPTIONS: i32 = 16;
    pub const DESCRIPTION_NAMES: i32 = 17;
    pub const ANGULAR_QUALIFIER: i32 = 18;
    pub const CONNECTION_AXIS1: i32 = 19;
    pub const CONNECTION_AXIS2: i32 = 20;
    pub const END: i32 = 21;
}
use child_lab as lab;

generic_empty_attribute!(
    /// `XCAFDoc_Dimension`.
    XCAFDocDimension, "XCAFDoc_Dimension", "58ed092c-44de-11d8-8776-001083004c77", state: (),
    dump: |attribute: &XCAFDocDimension, stream: &mut DumpStream, depth: i32| -> io::Result<()> {
        rustycad_tkservice::json_dump::class_begin(stream, "XCAFDoc_Dimension")?;
        super::generic_empty_tool::dump_generic_empty_base(attribute, stream, depth)?;
        let object = attribute.get_object().map_err(io::Error::other)?;
        let Some(object) = object.get() else {
            return Ok(());
        };
        field_dumped(stream, depth, "anObject.get()", &|s, d| object.dump_json(s, d))
    }
);

/// `(Enum)value` of a stored integer; `Standard_OutOfRange` for a value outside the
/// enumeration (an invalid enumerator natively).
pub(crate) fn enum_of<E: NativeEnum>(value: i32, what: &str) -> Result<E, Failure> {
    E::from_i32(value).ok_or_else(|| {
        rustycad_tkernel::standard::exceptions::out_of_range(&format!(
            "XCAFDoc: stored {what} {value}; expected 0..{}",
            E::ALL.len()
        ))
    })
}

impl XCAFDocDimension {
    /// `Set(theLabel)`: the dimension of the label, added when absent.
    pub fn set(label: &TdfLabel) -> Result<Handle<XCAFDocDimension>, Failure> {
        find_or_add(label, &Self::get_id(), Self::new)
    }
    /// `SetObject(theObject)`: backed up; the semantic name on the label, then every value
    /// of the object on fresh children.
    pub fn set_object(
        &self,
        object: &Handle<XCAFDimTolObjectsDimensionObject>,
    ) -> Result<(), Failure> {
        self.backup()?;
        let object = live(object, "XCAFDoc_Dimension::SetObject")?;
        let label = self.label();
        set_string(&label, &object.get_semantic_name())?;
        forget_children(&label, lab::TYPE, lab::END)?;
        TDataStdInteger::set(&child(&label, lab::TYPE)?, object.get_type() as i32)?;
        set_real_array(&child(&label, lab::VALUE)?, &object.get_values())?;
        TDataStdInteger::set(
            &child(&label, lab::QUALIFIER)?,
            object.get_qualifier() as i32,
        )?;
        let angular = object.get_angular_qualifier() as i32;
        TDataStdInteger::set(&child(&label, lab::ANGULAR_QUALIFIER)?, angular)?;
        store_classification(&label, object)?;
        store_geometry(&label, object)?;
        store_descriptions(&label, object)
    }
}

/// The class of tolerance, the decimal places, the modifiers and the path of `SetObject`.
fn store_classification(
    label: &TdfLabel,
    object: &XCAFDimTolObjectsDimensionObject,
) -> Result<(), Failure> {
    let (mut hole, mut form, mut grade) = Default::default();
    object.get_class_of_tolerance(&mut hole, &mut form, &mut grade);
    if form != XCAFDimTolObjectsDimensionFormVariance::None {
        set_integer_array(
            &child(label, lab::CLASS)?,
            &[i32::from(hole), form as i32, grade as i32],
        )?;
    }
    let (left, right) = object.get_nb_of_decimal_places();
    if left > 0 || right > 0 {
        set_integer_array(&child(label, lab::DEC)?, &[left, right])?;
    }
    let modifiers: Vec<i32> = object.get_modifiers().iter().map(|m| *m as i32).collect();
    if !modifiers.is_empty() {
        set_integer_array(&child(label, lab::MODIFIERS)?, &modifiers)?;
    }
    let path = object.get_path();
    if !path.is_null() {
        generate_shape(&child(label, lab::PATH)?, &path)?;
    }
    Ok(())
}

/// The direction, connections, plane, text point and presentation of `SetObject`.
fn store_geometry(
    label: &TdfLabel,
    object: &XCAFDimTolObjectsDimensionObject,
) -> Result<(), Failure> {
    if object.get_type() == XCAFDimTolObjectsDimensionType::LocationOriented {
        let mut direction = Direction3::default();
        object.get_direction(&mut direction);
        set_real_xyz(&child(label, lab::DIR)?, &direction.xyz())?;
    }
    if object.has_point() {
        let connection = (
            object.get_point(),
            object.get_connection_name(),
            object.is_point_connection(),
        );
        store_connection(
            label,
            [lab::PNT1, lab::CONNECTION_AXIS1],
            connection,
            &object.get_connection_axis(),
        )?;
    }
    if object.has_point2() {
        let connection = (
            object.get_point2(),
            object.get_connection_name2(),
            object.is_point_connection2(),
        );
        store_connection(
            label,
            [lab::PNT2, lab::CONNECTION_AXIS2],
            connection,
            &object.get_connection_axis2(),
        )?;
    }
    if object.has_plane() {
        let plane = object.get_plane();
        set_real_xyz(&child(label, lab::PLANE_LOC)?, &plane.location().xyz())?;
        set_real_xyz(&child(label, lab::PLANE_N)?, &plane.direction().xyz())?;
        set_real_xyz(&child(label, lab::PLANE_REF)?, &plane.x_direction().xyz())?;
    }
    if object.has_text_point() {
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

/// One connection: its point and name, and for an axis connection the direction and the X
/// direction in a six-value array.
fn store_connection(
    label: &TdfLabel,
    [point_tag, axis_tag]: [i32; 2],
    (point, name, is_point): (rustycad_tkmath::gp::Point3, Handle<HAsciiString>, bool),
    axis: &Axis2,
) -> Result<(), Failure> {
    let point_label = child(label, point_tag)?;
    set_real_xyz(&point_label, &point.xyz())?;
    set_string(&point_label, &name)?;
    if is_point {
        return Ok(());
    }
    let (d, x) = (axis.direction(), axis.x_direction());
    let [dx, dy, dz] = *d.coordinates();
    let [xx, xy, xz] = *x.coordinates();
    set_real_array(
        &child(label, axis_tag)?,
        &real_array(&[dx, dy, dz, xx, xy, xz]),
    )
}

/// The descriptions and their names of `SetObject`, in two extended string arrays.
fn store_descriptions(
    label: &TdfLabel,
    object: &XCAFDimTolObjectsDimensionObject,
) -> Result<(), Failure> {
    if !object.has_descriptions() {
        return Ok(());
    }
    let count = object.nb_descriptions();
    let text = |value: Handle<HAsciiString>| {
        value
            .get()
            .map(|v| ExtendedString::from_ascii(&v.string()))
            .unwrap_or_default()
    };
    let descriptions: Vec<ExtendedString> = (0..count)
        .map(|i| text(object.get_description(i)))
        .collect();
    let names: Vec<ExtendedString> = (0..count)
        .map(|i| text(object.get_description_name(i)))
        .collect();
    for (tag, values) in [
        (lab::DESCRIPTIONS, descriptions),
        (lab::DESCRIPTION_NAMES, names),
    ] {
        let attribute = TDataStdExtStringArray::set(&child(label, tag)?, 1, count, false)?;
        let array =
            rustycad_tkernel::ncollection::HArray1::with_bounds(1, count).unwrap_or_default();
        array
            .change_array1()
            .as_mut_slice()
            .clone_from_slice(&values);
        live(&attribute, "TDataStd_ExtStringArray::ChangeArray")?
            .change_array(&Handle::new(array), true)?;
    }
    Ok(())
}

impl XCAFDocDimension {
    /// `GetObject`: a new object read back from the children (the native `FindChild` calls
    /// create the children that are missing).
    pub fn get_object(&self) -> Result<Handle<XCAFDimTolObjectsDimensionObject>, Failure> {
        let label = self.label();
        let object = XCAFDimTolObjectsDimensionObject::new();
        object.set_semantic_name(&get_string(&label)?);
        read_values(&label, &object)?;
        read_classification(&label, &object)?;
        read_geometry(&label, &object)?;
        read_descriptions(&label, &object)?;
        Ok(Handle::new(object))
    }
}

/// The type, values and qualifiers of `GetObject`.
fn read_values(label: &TdfLabel, object: &XCAFDimTolObjectsDimensionObject) -> Result<(), Failure> {
    if let Some(kind) = integer_value(&child(label, lab::TYPE)?)? {
        object.set_type(enum_of::<XCAFDimTolObjectsDimensionType>(
            kind,
            "dimension type",
        )?);
    }
    let mut values = Handle::default();
    if child(label, lab::VALUE)?.find_attribute_as(
        &rustycad_tklcaf::tdata_std::TDataStdRealArray::get_id(),
        &mut values,
    )? {
        let values: Handle<rustycad_tklcaf::tdata_std::TDataStdRealArray> = values;
        let array = live(&values, "XCAFDoc_Dimension::GetObject")?.array();
        if !array.is_null() {
            object.set_values(&array);
        }
    }
    if let Some(value) = integer_value(&child(label, lab::QUALIFIER)?)? {
        object.set_qualifier(enum_of::<XCAFDimTolObjectsDimensionQualifier>(
            value,
            "qualifier",
        )?);
    }
    if let Some(value) = integer_value(&child(label, lab::ANGULAR_QUALIFIER)?)? {
        object.set_angular_qualifier(enum_of::<XCAFDimTolObjectsAngularQualifier>(
            value,
            "angular qualifier",
        )?);
    }
    Ok(())
}

/// The class of tolerance, decimal places, modifiers and path of `GetObject`.
fn read_classification(
    label: &TdfLabel,
    object: &XCAFDimTolObjectsDimensionObject,
) -> Result<(), Failure> {
    if let Some(class) = integer_array_values(&child(label, lab::CLASS)?)?.filter(|v| !v.is_empty())
    {
        let value = |index: usize| class.get(index).copied().unwrap_or_default();
        object.set_class_of_tolerance(
            value(0) != 0,
            enum_of::<XCAFDimTolObjectsDimensionFormVariance>(value(1), "form variance")?,
            enum_of::<XCAFDimTolObjectsDimensionGrade>(value(2), "grade")?,
        );
    }
    if let Some(dec) = integer_array_values(&child(label, lab::DEC)?)?.filter(|v| !v.is_empty()) {
        object.set_nb_of_decimal_places(dec[0], dec.get(1).copied().unwrap_or_default());
    }
    if let Some(values) = integer_array_values(&child(label, lab::MODIFIERS)?)? {
        let modifiers = values
            .into_iter()
            .map(|v| enum_of::<XCAFDimTolObjectsDimensionModif>(v, "modifier"))
            .collect::<Result<Vec<_>, _>>()?;
        object.set_modifiers(&modifiers);
    }
    if let Some(path) = named_shape_get(&child(label, lab::PATH)?)? {
        object.set_path(&topods::edge(&path)?);
    }
    Ok(())
}

/// The direction, connections, plane, text point and presentation of `GetObject`.
fn read_geometry(
    label: &TdfLabel,
    object: &XCAFDimTolObjectsDimensionObject,
) -> Result<(), Failure> {
    if let Some(direction) = get_real_xyz(&child(label, lab::DIR)?)? {
        object.set_direction(&Direction3::from_xyz(direction)?);
    }
    read_connection(label, [lab::PNT1, lab::CONNECTION_AXIS1], object, true)?;
    read_connection(label, [lab::PNT2, lab::CONNECTION_AXIS2], object, false)?;
    let plane = (
        get_real_xyz(&child(label, lab::PLANE_LOC)?)?,
        get_real_xyz(&child(label, lab::PLANE_N)?)?,
        get_real_xyz(&child(label, lab::PLANE_REF)?)?,
    );
    if let (Some(location), Some(normal), Some(x)) = plane {
        let axis = Axis2::with_x_direction(
            point_of(&location),
            Direction3::from_xyz(normal)?,
            Direction3::from_xyz(x)?,
        )?;
        object.set_plane(&axis);
    }
    if let Some(point) = get_real_xyz(&child(label, lab::PNT_TEXT)?)? {
        object.set_point_text_attach(&point_of(&point));
    }
    let presentation_label = child(label, lab::PRESENTATION)?;
    if let Some(presentation) = named_shape(&presentation_label)?.filter(|s| !s.is_null()) {
        object.set_presentation(&presentation, &get_string(&presentation_label)?);
    }
    Ok(())
}

/// One connection of `GetObject`: an axis when the six-value array is there, else a point.
fn read_connection(
    label: &TdfLabel,
    [point_tag, axis_tag]: [i32; 2],
    object: &XCAFDimTolObjectsDimensionObject,
    first: bool,
) -> Result<(), Failure> {
    let point_label = child(label, point_tag)?;
    let Some(point) = get_real_xyz(&point_label)? else {
        return Ok(());
    };
    let point = point_of(&point);
    let name = get_string(&point_label)?;
    if first {
        object.set_connection_name(&name);
    } else {
        object.set_connection_name2(&name);
    }
    let axis = real_array_values(&child(label, axis_tag)?)?.filter(|v| v.len() == 6);
    let Some(axis) = axis else {
        if first {
            object.set_point(&point)
        } else {
            object.set_point2(&point)
        }
        return Ok(());
    };
    let direction = Direction3::new(axis[0], axis[1], axis[2])?;
    let x = Direction3::new(axis[3], axis[4], axis[5])?;
    let axis = Axis2::with_x_direction(point, direction, x)?;
    if first {
        object.set_connection_axis(&axis);
    } else {
        object.set_connection_axis2(&axis);
    }
    Ok(())
}

/// The descriptions of `GetObject` (both arrays present).
fn read_descriptions(
    label: &TdfLabel,
    object: &XCAFDimTolObjectsDimensionObject,
) -> Result<(), Failure> {
    let mut descriptions: Handle<TDataStdExtStringArray> = Handle::default();
    let mut names: Handle<TDataStdExtStringArray> = Handle::default();
    let id = TDataStdExtStringArray::get_id();
    if !child(label, lab::DESCRIPTIONS)?.find_attribute_as(&id, &mut descriptions)?
        || !child(label, lab::DESCRIPTION_NAMES)?.find_attribute_as(&id, &mut names)?
    {
        return Ok(());
    }
    let (descriptions, names) = (
        live(&descriptions, "XCAFDoc_Dimension::GetObject")?,
        live(&names, "XCAFDoc_Dimension::GetObject")?,
    );
    let ascii = |text: ExtendedString| {
        Handle::new(HAsciiString::from_string(&AsciiString::from_extended(
            &text, None,
        )))
    };
    for index in 1..=descriptions.length() {
        object.add_description(
            &ascii(descriptions.value(index)?),
            &ascii(names.value(index)?),
        );
    }
    Ok(())
}
