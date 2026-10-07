//! The OCAF `DumpJson` pieces shared by the attributes: the label entry field and the
//! native `TDF_Attribute::DumpJson` body (also dumped as the base class of every attribute).
use crate::tdf::{TdfAttribute, TdfAttributeState, TdfLabel, TdfTool};
use rustycad_tkcdf::json_dump::{base_class, class_begin, field_guid, field_integer, field_string};
use rustycad_tkernel::{
    ncollection::IndexedMap,
    standard::{Handle, dump::DumpStream},
    tcollection::AsciiString,
};
use std::io;

/// `NCollection_IndexedMap<occ::handle<TDF_Attribute>>`: the attributes of an extended dump
/// by rank.
pub type AttributeIndexMap = IndexedMap<Handle<dyn TdfAttribute>>;

/// `TDF_Tool::Entry(label, s)` then `OCCT_DUMP_FIELD_VALUE_STRING(stream, s)`.
pub fn dump_label_entry(stream: &mut DumpStream, field: &str, label: &TdfLabel) -> io::Result<()> {
    let mut entry = AsciiString::default();
    TdfTool::entry(label, &mut entry);
    field_string(stream, field, &String::from_utf8_lossy(entry.as_bytes()))
}

/// The native `TDF_Attribute::DumpJson`; `attribute_dump_json(&attribute, &mut stream, -1)?`.
pub fn attribute_dump_json<A: TdfAttribute + ?Sized>(
    attribute: &A,
    stream: &mut DumpStream,
    _depth: i32,
) -> io::Result<()> {
    class_begin(stream, "TDF_Attribute")?;
    dump_label_entry(stream, "aLabel", &attribute.label())?;
    field_guid(stream, "ID()", attribute.id())?;
    field_integer(stream, "Transaction()", attribute.transaction())?;
    field_integer(stream, "IsValid()", i32::from(attribute.is_valid()))?;
    field_integer(stream, "IsNew()", i32::from(attribute.is_new()))?;
    field_integer(stream, "IsBackuped()", i32::from(attribute.is_backuped()))?;
    field_integer(stream, "IsForgotten()", i32::from(attribute.is_forgotten()))?;
    field_integer(stream, "myFlags", attribute.attribute_base().flags())
}

/// `OCCT_DUMP_BASE_CLASS(theOStream, theDepth, TDF_Attribute)` in a derived `DumpJson`.
pub fn dump_attribute_json_base<A: TdfAttribute + ?Sized>(
    attribute: &A,
    stream: &mut DumpStream,
    depth: i32,
) -> io::Result<()> {
    base_class(stream, depth, &|s, d| attribute_dump_json(attribute, s, d))
}
