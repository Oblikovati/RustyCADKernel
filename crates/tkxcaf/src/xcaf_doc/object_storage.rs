//! How `XCAFDoc_Datum`, `XCAFDoc_Dimension`, `XCAFDoc_GeomTolerance` and `XCAFDoc_View` keep
//! their description object: one value per child label, in standard attributes (the
//! file-static `setString`, `getString`, `setRealArrayXYZ` and `getRealArrayXYZ` and the
//! repeated array, integer and real blocks of the native files).
use crate::xcaf_dim_tol_objects::real_array;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkcaf::tnaming::{TNamingBuilder, TNamingNamedShape, TNamingTool};
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString, HAsciiString};
use rustycad_tklcaf::tdata_std::{
    TDataStdGenericExtString, TDataStdInteger, TDataStdIntegerArray, TDataStdName, TDataStdReal,
    TDataStdRealArray,
};
use rustycad_tklcaf::tdf::TdfLabel;
use rustycad_tkmath::gp::{Axis2, Direction3, Point3, Xyz};

/// `Label().FindChild(tag)` (created when absent, as the native default).
pub(crate) fn child(label: &TdfLabel, tag: i32) -> Result<TdfLabel, Failure> {
    label.find_child(tag, true)
}

/// `ForgetAllAttributes` of the children `first..end`.
pub(crate) fn forget_children(label: &TdfLabel, first: i32, end: i32) -> Result<(), Failure> {
    for tag in first..end {
        child(label, tag)?.forget_all_attributes(true)?;
    }
    Ok(())
}

/// `setString`: a `TDataStd_Name` with the string (nothing for a null string).
pub(crate) fn set_string(label: &TdfLabel, text: &Handle<HAsciiString>) -> Result<(), Failure> {
    let Some(text) = text.get() else {
        return Ok(());
    };
    TDataStdName::set(label, &ExtendedString::from_ascii(&text.string()))?;
    Ok(())
}

/// `getString`: the label's non-empty `TDataStd_Name` as an ASCII string, else null.
pub(crate) fn get_string(label: &TdfLabel) -> Result<Handle<HAsciiString>, Failure> {
    let mut name: Handle<TDataStdName> = Handle::default();
    if !label.find_attribute_as(&TDataStdName::get_id(), &mut name)? {
        return Ok(Handle::default());
    }
    let text = rustycad_tklcaf::live(&name, "XCAFDoc getString")?.get();
    if text.is_empty() {
        return Ok(Handle::default());
    }
    Ok(Handle::new(HAsciiString::from_string(
        &AsciiString::from_extended(&text, None),
    )))
}

/// `TDataStd_RealArray::Set(label, lower, upper)` then `ChangeArray(values)`.
pub(crate) fn set_real_array(
    label: &TdfLabel,
    values: &Handle<HArray1<f64>>,
) -> Result<(), Failure> {
    let Some(array) = values.get() else {
        return Ok(());
    };
    let (lower, length) = {
        let values = array.array1();
        (values.lower(), values.length())
    };
    let attribute = TDataStdRealArray::set(label, lower, lower + length - 1, false)?;
    rustycad_tklcaf::live(&attribute, "TDataStd_RealArray::ChangeArray")?.change_array(values, true)
}

/// `setRealArrayXYZ` (and the three-value blocks): a `TDataStd_RealArray` `1..=3`.
pub(crate) fn set_real_xyz(label: &TdfLabel, xyz: &Xyz) -> Result<(), Failure> {
    set_real_array(label, &real_array(&[xyz.x(), xyz.y(), xyz.z()]))
}

/// `TDataStd_IntegerArray::Set(label, 1, n)` then `ChangeArray(values)`.
pub(crate) fn set_integer_array(label: &TdfLabel, values: &[i32]) -> Result<(), Failure> {
    let array = HArray1::with_bounds(1, values.len() as i32).unwrap_or_default();
    array.change_array1().as_mut_slice().copy_from_slice(values);
    let attribute = TDataStdIntegerArray::set(label, 1, values.len() as i32, false)?;
    rustycad_tklcaf::live(&attribute, "TDataStd_IntegerArray::ChangeArray")?
        .change_array(&Handle::new(array), true)
}

/// The values of the label's `TDataStd_RealArray`, `None` without one.
pub(crate) fn real_array_values(label: &TdfLabel) -> Result<Option<Vec<f64>>, Failure> {
    let mut found: Handle<TDataStdRealArray> = Handle::default();
    if !label.find_attribute_as(&TDataStdRealArray::get_id(), &mut found)? {
        return Ok(None);
    }
    let array = rustycad_tklcaf::live(&found, "TDataStd_RealArray")?.array();
    Ok(Some(
        crate::xcaf_dim_tol_objects::array_values(&array).unwrap_or_default(),
    ))
}

/// `getRealArrayXYZ`: the label's three-value `TDataStd_RealArray`, `None` otherwise.
pub(crate) fn get_real_xyz(label: &TdfLabel) -> Result<Option<Xyz>, Failure> {
    Ok(
        real_array_values(label)?.and_then(|values| match values.as_slice() {
            [x, y, z] => Some(Xyz::new(*x, *y, *z)),
            _ => None,
        }),
    )
}

/// The values of the label's `TDataStd_IntegerArray` with a non-null array, `None` otherwise.
pub(crate) fn integer_array_values(label: &TdfLabel) -> Result<Option<Vec<i32>>, Failure> {
    let mut found: Handle<TDataStdIntegerArray> = Handle::default();
    if !label.find_attribute_as(&TDataStdIntegerArray::get_id(), &mut found)? {
        return Ok(None);
    }
    let array = rustycad_tklcaf::live(&found, "TDataStd_IntegerArray")?.array();
    Ok(array.get().map(|array| array.array1().as_slice().to_vec()))
}

/// The value of the label's `TDataStd_Integer`, `None` without one.
pub(crate) fn integer_value(label: &TdfLabel) -> Result<Option<i32>, Failure> {
    let mut found: Handle<TDataStdInteger> = Handle::default();
    if !label.find_attribute_as(&TDataStdInteger::get_id(), &mut found)? {
        return Ok(None);
    }
    Ok(Some(
        rustycad_tklcaf::live(&found, "TDataStd_Integer")?.get(),
    ))
}

/// The value of the label's `TDataStd_Real`, `None` without one.
pub(crate) fn real_value(label: &TdfLabel) -> Result<Option<f64>, Failure> {
    let mut found: Handle<TDataStdReal> = Handle::default();
    if !label.find_attribute_as(&TDataStdReal::get_id(), &mut found)? {
        return Ok(None);
    }
    Ok(Some(rustycad_tklcaf::live(&found, "TDataStd_Real")?.get()))
}

/// A `TNaming_Builder` on the label with the shape generated.
pub(crate) fn generate_shape(label: &TdfLabel, shape: &Shape) -> Result<(), Failure> {
    TNamingBuilder::new(label)?.generated(shape)
}

/// `TNaming_Tool::GetShape` of the label's named shape, `None` without one.
pub(crate) fn named_shape(label: &TdfLabel) -> Result<Option<Shape>, Failure> {
    let mut found: Handle<TNamingNamedShape> = Handle::default();
    if !label.find_attribute_as(&TNamingNamedShape::get_id(), &mut found)? {
        return Ok(None);
    }
    Ok(Some(TNamingTool::get_shape(&found)?))
}

/// `TNaming_NamedShape::Get` of the label's named shape, `None` without one.
pub(crate) fn named_shape_get(label: &TdfLabel) -> Result<Option<Shape>, Failure> {
    let mut found: Handle<TNamingNamedShape> = Handle::default();
    if !label.find_attribute_as(&TNamingNamedShape::get_id(), &mut found)? {
        return Ok(None);
    }
    Ok(Some(TNamingNamedShape::get(&found)?))
}

/// The three `gp_Ax2` blocks (location, direction, X direction) on three child labels.
pub(crate) fn set_axis_children(
    label: &TdfLabel,
    tags: [i32; 3],
    axis: &Axis2,
) -> Result<(), Failure> {
    set_real_xyz(&child(label, tags[0])?, &axis.location().xyz())?;
    set_real_xyz(&child(label, tags[1])?, &axis.direction().xyz())?;
    set_real_xyz(&child(label, tags[2])?, &axis.x_direction().xyz())
}

/// The `gp_Ax2(aL, aD, aDR)` of three child labels holding three-value real arrays, `None`
/// when one is missing.
pub(crate) fn get_axis_children(
    label: &TdfLabel,
    tags: [i32; 3],
) -> Result<Option<Axis2>, Failure> {
    let (Some(location), Some(direction), Some(x)) = (
        get_real_xyz(&child(label, tags[0])?)?,
        get_real_xyz(&child(label, tags[1])?)?,
        get_real_xyz(&child(label, tags[2])?)?,
    ) else {
        return Ok(None);
    };
    let point = Point3::new(location.x(), location.y(), location.z());
    let axis = Axis2::with_x_direction(
        point,
        Direction3::from_xyz(direction)?,
        Direction3::from_xyz(x)?,
    )?;
    Ok(Some(axis))
}

/// A point of a three-value array.
pub(crate) fn point_of(xyz: &Xyz) -> Point3 {
    Point3::new(xyz.x(), xyz.y(), xyz.z())
}
