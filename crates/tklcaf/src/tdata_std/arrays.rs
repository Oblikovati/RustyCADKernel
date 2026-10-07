// Rust port of ApplicationFramework/TKLCAF/TDataStd/TDataStd_IntegerArray, TDataStd_RealArray,
// TDataStd_ExtStringArray and TDataStd_ByteArray (.hxx and .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The array attributes over `NCollection_HArray1` (ocaf-design.md section 7).
use super::array_attribute::array_attribute;
use super::array_core::{
    ArrayAttribute, ArrayState, array, change_array, dump_delta_tail, dump_json, init, items_of,
    length_of, lower_of, paste, restore, set_attr, set_default_id, set_id, set_value, upper_of,
    value,
};
use super::array_delta::{
    TDataStdDeltaOnModificationOfByteArray, TDataStdDeltaOnModificationOfExtStringArray,
    TDataStdDeltaOnModificationOfIntArray, TDataStdDeltaOnModificationOfRealArray,
};
use super::support::address_of;
use crate::guid_from_text;
use crate::tdf::{
    TdfAttribute, TdfAttributeBase, TdfDefaultDeltaOnModification, TdfDeltaOnModification,
    TdfLabel, TdfRelocationTable,
};
use rustycad_tkcdf::json_dump::{field_byte_char, field_integer, field_real, field_string};
use rustycad_tkernel::{
    impl_transient,
    ncollection::HArray1,
    standard::{
        Failure, Guid, Handle, SharedState, StandardTransient, dump::DumpStream, real::real_first,
    },
    tcollection::{AsciiString, ExtendedString},
};
use std::io;

array_attribute!(
    /// `TDataStd_IntegerArray`; `TDataStdIntegerArray::set(&label, 1, 3, false)`.
    ///
    /// ```
    /// use rustycad_tklcaf::{tdata_std::TDataStdIntegerArray, tdf::TdfData};
    /// let data = TdfData::new();
    /// let array = TDataStdIntegerArray::set(&data.root(), 1, 3, false).unwrap();
    /// array.set_value(2, 7).unwrap();
    /// assert_eq!((array.value(2).unwrap(), array.length()), (7, 3));
    /// ```
    TDataStdIntegerArray, "TDataStd_IntegerArray", item: i32,
    guid: "2a96b61d-ec8b-11d0-bee7-080009dc3333", fill: || 0, null_value: || 0, null_upper: 0,
    guards_null: false, id_first: false, delta: TDataStdDeltaOnModificationOfIntArray,
    array: array, dump: dump_integer_array, dump_item: dump_integer_item
);

array_attribute!(
    /// `TDataStd_RealArray`; `TDataStdRealArray::set(&label, 1, 3, false)`.
    ///
    /// ```
    /// use rustycad_tklcaf::{tdata_std::TDataStdRealArray, tdf::TdfData};
    /// let data = TdfData::new();
    /// let array = TDataStdRealArray::set(&data.root(), 0, 1, false).unwrap();
    /// array.set_value(1, 2.5).unwrap();
    /// assert_eq!(array.value(1).unwrap(), 2.5);
    /// ```
    TDataStdRealArray, "TDataStd_RealArray", item: f64,
    guid: "2a96b61e-ec8b-11d0-bee7-080009dc3333", fill: || 0.0, null_value: real_first,
    null_upper: 0, guards_null: true, id_first: false,
    delta: TDataStdDeltaOnModificationOfRealArray, array: array, dump: dump_real_array,
    dump_item: dump_real_item
);

array_attribute!(
    /// `TDataStd_ExtStringArray`; `TDataStdExtStringArray::set(&label, 1, 2, false)`.
    ///
    /// ```
    /// use rustycad_tklcaf::{tdata_std::TDataStdExtStringArray, tdf::TdfData};
    /// use rustycad_tkernel::tcollection::ExtendedString;
    /// let data = TdfData::new();
    /// let array = TDataStdExtStringArray::set(&data.root(), 1, 2, false).unwrap();
    /// array.set_value(1, ExtendedString::new("a")).unwrap();
    /// assert!(array.value(2).unwrap().is_empty());
    /// ```
    TDataStdExtStringArray, "TDataStd_ExtStringArray", item: ExtendedString,
    guid: "2a96b624-ec8b-11d0-bee7-080009dc3333", fill: ExtendedString::default,
    null_value: ExtendedString::default, null_upper: 0, guards_null: false, id_first: true,
    delta: TDataStdDeltaOnModificationOfExtStringArray, array: array,
    dump: dump_ext_string_array, dump_item: dump_ext_string_item
);

array_attribute!(
    /// `TDataStd_ByteArray`; `TDataStdByteArray::set(&label, 1, 4, false)`.
    ///
    /// ```
    /// use rustycad_tklcaf::{tdata_std::TDataStdByteArray, tdf::TdfData};
    /// let data = TdfData::new();
    /// let array = TDataStdByteArray::set(&data.root(), 1, 4, false).unwrap();
    /// array.set_value(4, 255).unwrap();
    /// assert_eq!(array.internal_array().get().unwrap().array1().as_slice(), &[0, 0, 0, 255]);
    /// ```
    TDataStdByteArray, "TDataStd_ByteArray", item: u8,
    guid: "FD9B918F-2980-4c66-85E0-D71965475290", fill: || 0, null_value: || 0,
    null_upper: -1, guards_null: false, id_first: false,
    delta: TDataStdDeltaOnModificationOfByteArray, array: internal_array,
    dump: dump_byte_array, dump_item: dump_byte_item
);

/// `TDataStd_IntegerArray::Dump`: `"\nIntegerArray:: <this> :"`, the items, the delta
/// flag and the ID.
fn dump_integer_array(array: &TDataStdIntegerArray, stream: &mut dyn io::Write) -> io::Result<()> {
    write!(stream, "\nIntegerArray:: {} :", address_of(array))?;
    for item in items_of(array).unwrap_or_default() {
        write!(stream, " {item}")?;
    }
    dump_delta_tail(array, stream)
}

/// `TDataStd_RealArray::Dump`: `"\nRealArray::<this> :"`, the items (`%g`), the delta
/// flag and the ID.
fn dump_real_array(array: &TDataStdRealArray, stream: &mut dyn io::Write) -> io::Result<()> {
    write!(stream, "\nRealArray::{} :", address_of(array))?;
    for item in items_of(array).unwrap_or_default() {
        write!(stream, " ")?;
        stream.write_all(AsciiString::from_real(item).as_bytes())?;
    }
    dump_delta_tail(array, stream)
}

/// `TDataStd_ExtStringArray::Dump`: `"\nExtStringArray :"`, one `"\t<item>\n"` per item,
/// the delta flag and the ID.
fn dump_ext_string_array(
    array: &TDataStdExtStringArray,
    stream: &mut dyn io::Write,
) -> io::Result<()> {
    write!(stream, "\nExtStringArray :")?;
    for item in items_of(array).unwrap_or_default() {
        writeln!(stream, "\t{}", item.to_utf8())?;
    }
    dump_delta_tail(array, stream)
}

/// `TDataStd_ByteArray::Dump`: `"\nByteArray: "` and the ID only.
fn dump_byte_array(array: &TDataStdByteArray, stream: &mut dyn io::Write) -> io::Result<()> {
    writeln!(stream, "\nByteArray: {}", array.id())
}

/// The item field of `TDataStd_IntegerArray::DumpJson`.
fn dump_integer_item(stream: &mut DumpStream, item: &i32) -> io::Result<()> {
    field_integer(stream, "aValue", *item)
}

/// The item field of `TDataStd_RealArray::DumpJson`.
fn dump_real_item(stream: &mut DumpStream, item: &f64) -> io::Result<()> {
    field_real(stream, "aValue", *item)
}

/// The item field of `TDataStd_ExtStringArray::DumpJson`.
fn dump_ext_string_item(stream: &mut DumpStream, item: &ExtendedString) -> io::Result<()> {
    field_string(stream, "aValue", &item.to_utf8())
}

/// The item field of `TDataStd_ByteArray::DumpJson`: a `uint8_t` streams as a character.
fn dump_byte_item(stream: &mut DumpStream, item: &u8) -> io::Result<()> {
    field_byte_char(stream, "aValue", *item)
}
