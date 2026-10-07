// Rust port of ApplicationFramework/TKLCAF/TDataStd/TDataStd_IntegerList, TDataStd_RealList,
// TDataStd_ExtStringList, TDataStd_BooleanList and TDataStd_ReferenceList (.hxx and .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The list attributes (ocaf-design.md section 7).
use super::list_attribute::{list_attribute, list_attribute_base};
use super::list_core::{
    ListAttribute, ListState, dump, dump_json, edit, extent, first, insert_after,
    insert_after_by_index, insert_before, insert_before_by_index, items, last, paste, paste_items,
    remove, remove_by_index, restore, set_attr, set_default_id, set_id,
};
use crate::guid_from_text;
use crate::live;
use crate::tdf::{
    TdfAttribute, TdfAttributeBase, TdfAttributeState, TdfDataSet, TdfLabel, TdfRelocationTable,
    TdfTool,
};
use rustycad_tkcdf::json_dump::{field_byte_char, field_integer, field_real, field_string};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Guid, Handle, SharedState, StandardTransient, dump::DumpStream},
    tcollection::{AsciiString, ExtendedString},
};
use std::io;

list_attribute!(
    /// `TDataStd_IntegerList`; `TDataStdIntegerList::set(&label)`.
    ///
    /// ```
    /// use rustycad_tklcaf::{tdata_std::TDataStdIntegerList, tdf::TdfData};
    /// let data = TdfData::new();
    /// let list = TDataStdIntegerList::set(&data.root()).unwrap();
    /// list.append(2).unwrap();
    /// list.prepend(1).unwrap();
    /// assert_eq!(list.list(), vec![1, 2]);
    /// ```
    TDataStdIntegerList, "TDataStd_IntegerList", "IntegerList", item: i32,
    guid: "E406AA18-FF3F-483b-9A78-1A5EA5D1AA52", dump_item: dump_integer_item,
    paste: paste_values, references: no_references
);

list_attribute!(
    /// `TDataStd_RealList`; `TDataStdRealList::set(&label)`.
    ///
    /// ```
    /// use rustycad_tklcaf::{tdata_std::TDataStdRealList, tdf::TdfData};
    /// let data = TdfData::new();
    /// let list = TDataStdRealList::set(&data.root()).unwrap();
    /// list.append(1.5).unwrap();
    /// assert_eq!(list.last().unwrap(), 1.5);
    /// ```
    TDataStdRealList, "TDataStd_RealList", "RealList", item: f64,
    guid: "349ACE18-7CD6-4525-9938-FBBF22AA54D3", dump_item: dump_real_item,
    paste: paste_values, references: no_references
);

list_attribute!(
    /// `TDataStd_ExtStringList`; `TDataStdExtStringList::set(&label)`.
    ///
    /// ```
    /// use rustycad_tklcaf::{tdata_std::TDataStdExtStringList, tdf::TdfData};
    /// use rustycad_tkernel::tcollection::ExtendedString;
    /// let data = TdfData::new();
    /// let list = TDataStdExtStringList::set(&data.root()).unwrap();
    /// list.append(ExtendedString::new("a")).unwrap();
    /// assert_eq!(list.extent(), 1);
    /// ```
    TDataStdExtStringList, "TDataStd_ExtStringList", "ExtStringList", item: ExtendedString,
    guid: "D13FBE0A-E084-4912-A99D-7713C59C0AC4", dump_item: dump_ext_string_item,
    paste: paste_values, references: no_references
);

list_attribute!(
    /// `TDataStd_ReferenceList`; `TDataStdReferenceList::set(&label)`.
    ///
    /// ```
    /// use rustycad_tklcaf::{tdata_std::TDataStdReferenceList, tdf::TdfData};
    /// let data = TdfData::new();
    /// let list = TDataStdReferenceList::set(&data.root()).unwrap();
    /// list.append(data.root().find_child(1, true).unwrap()).unwrap();
    /// assert_eq!(list.first().unwrap().tag(), 1);
    /// ```
    TDataStdReferenceList, "TDataStd_ReferenceList", "ReferenceList", item: TdfLabel,
    guid: "FCC1A658-59FF-4218-931B-0320A2B469A7", dump_item: dump_label_item,
    paste: paste_references, references: add_references
);

list_attribute_base!(
    /// `TDataStd_BooleanList`; `TDataStdBooleanList::set(&label)`. The items are stored
    /// as the native `uint8_t` 0/1.
    ///
    /// ```
    /// use rustycad_tklcaf::{tdata_std::TDataStdBooleanList, tdf::TdfData};
    /// let data = TdfData::new();
    /// let list = TDataStdBooleanList::set(&data.root()).unwrap();
    /// list.append(true).unwrap();
    /// assert!(list.first().unwrap());
    /// ```
    TDataStdBooleanList, "TDataStd_BooleanList", "BooleanList", stored: u8,
    guid: "23A9D60E-A033-44d8-96EE-015587A41BBC", dump_item: dump_byte_item
);

impl TDataStdBooleanList {
    /// `Prepend`: backs up, then adds the item in front.
    pub fn prepend(&self, value: bool) -> Result<(), Failure> {
        edit(self, |list| list.insert(0, u8::from(value)))
    }
    /// `Append`: backs up, then adds the item at the end.
    pub fn append(&self, value: bool) -> Result<(), Failure> {
        edit(self, |list| list.push(u8::from(value)))
    }
    /// `InsertBefore(index, value)`: before the 1-based index; false outside the list.
    pub fn insert_before_by_index(&self, index: i32, value: bool) -> Result<bool, Failure> {
        insert_before_by_index(self, index, u8::from(value))
    }
    /// `InsertAfter(index, value)`: after the 1-based index; false outside the list.
    pub fn insert_after_by_index(&self, index: i32, value: bool) -> Result<bool, Failure> {
        insert_after_by_index(self, index, u8::from(value))
    }
    /// `First`: the stored item is 1; `Standard_NoSuchObject` for an empty list.
    pub fn first(&self) -> Result<bool, Failure> {
        Ok(first(self)? == 1)
    }
    /// `Last`: the stored item is 1; `Standard_NoSuchObject` for an empty list.
    pub fn last(&self) -> Result<bool, Failure> {
        Ok(last(self)? == 1)
    }
    /// `List`: a copy of the stored 0/1 items.
    pub fn list(&self) -> Vec<u8> {
        items(self)
    }
}

impl TdfAttribute for TDataStdBooleanList {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        self.state.read().id
    }
    fn set_id(&self, guid: &Guid) -> Result<(), Failure> {
        set_id(self, guid)
    }
    fn set_default_id(&self) -> Result<(), Failure> {
        set_default_id(self)
    }
    /// `Restore(With)`: the other list's items as 0/1, and its ID.
    fn restore(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        restore(self, with)?;
        edit_without_backup(self, |item| u8::from(*item != 0));
        Ok(())
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::new()).cast()
    }
    /// `Paste(Into, RT)`: `Into` takes the items (`Append(item != 0)`) and the ID.
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        _table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        let normalized = items(self)
            .iter()
            .map(|item| u8::from(*item != 0))
            .collect();
        paste_items::<Self>(into, normalized, &self.id())
    }
    fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        dump(self, stream)
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        dump_json(self, stream, depth)
    }
}

/// Maps every stored item in place, without backup (inside `Restore`).
fn edit_without_backup(list: &TDataStdBooleanList, map: impl Fn(&u8) -> u8) {
    let mut state = list.state.write();
    state.list = state.list.iter().map(map).collect();
}

/// `Paste` of the value lists: every item is copied.
fn paste_values<A: ListAttribute>(
    attribute: &A,
    into: &Handle<dyn TdfAttribute>,
    _table: &Handle<TdfRelocationTable>,
) -> Result<(), Failure> {
    paste(attribute, into)
}

/// `TDataStd_ReferenceList::Paste`: the non-null labels, relocated when the table knows
/// them; the null labels are dropped, as natively.
fn paste_references(
    attribute: &TDataStdReferenceList,
    into: &Handle<dyn TdfAttribute>,
    table: &Handle<TdfRelocationTable>,
) -> Result<(), Failure> {
    let relocated = items(attribute)
        .into_iter()
        .filter(|label| !label.is_null())
        .map(|label| relocate(table, label))
        .collect::<Result<_, _>>()?;
    paste_items::<TDataStdReferenceList>(into, relocated, &attribute.id())
}

/// `RT->HasRelocation(L, rL) ? rL : L`; `Standard_NullObject` for a null table.
fn relocate(table: &Handle<TdfRelocationTable>, label: TdfLabel) -> Result<TdfLabel, Failure> {
    let mut target = TdfLabel::default();
    let found =
        live(table, "TDataStd_ReferenceList::Paste")?.has_label_relocation(&label, &mut target);
    Ok(if found { target } else { label })
}

/// The lists without references add nothing (`TDF_Attribute::References`).
fn no_references<A: ListAttribute>(_attribute: &A, _data_set: &Handle<TdfDataSet>) {}

/// `TDataStd_ReferenceList::References`: every label of a list on a label that is not
/// imported.
fn add_references(attribute: &TDataStdReferenceList, data_set: &Handle<TdfDataSet>) {
    let (false, Some(data_set)) = (attribute.label().is_imported(), data_set.get()) else {
        return;
    };
    for label in items(attribute) {
        data_set.add_label(&label);
    }
}

/// The item field of `TDataStd_IntegerList::DumpJson`.
fn dump_integer_item(stream: &mut DumpStream, item: &i32) -> io::Result<()> {
    field_integer(stream, "aValue", *item)
}

/// The item field of `TDataStd_RealList::DumpJson`.
fn dump_real_item(stream: &mut DumpStream, item: &f64) -> io::Result<()> {
    field_real(stream, "aValue", *item)
}

/// The item field of `TDataStd_ExtStringList::DumpJson`.
fn dump_ext_string_item(stream: &mut DumpStream, item: &ExtendedString) -> io::Result<()> {
    field_string(stream, "aValue", &item.to_utf8())
}

/// The item field of `TDataStd_BooleanList::DumpJson`: a `uint8_t` streams as a character.
fn dump_byte_item(stream: &mut DumpStream, item: &u8) -> io::Result<()> {
    field_byte_char(stream, "aValue", *item)
}

/// The item field of `TDataStd_ReferenceList::DumpJson`: the label entry.
fn dump_label_item(stream: &mut DumpStream, item: &TdfLabel) -> io::Result<()> {
    let mut entry = AsciiString::default();
    TdfTool::entry(item, &mut entry);
    field_string(stream, "aLabel", &String::from_utf8_lossy(entry.as_bytes()))
}
