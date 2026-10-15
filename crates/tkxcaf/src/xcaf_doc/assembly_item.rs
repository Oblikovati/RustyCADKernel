// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc_AssemblyItemId and XCAFDoc_AssemblyItemRef
// (.hxx and .cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The items of an assembly that notes annotate: `XCAFDoc_AssemblyItemId` (the path of
//! label entries from the root assembly to the item) and `XCAFDoc_AssemblyItemRef` (the
//! attribute referring to an item, with an optional attribute GUID or subshape index).
use super::support::find;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkcaf::tnaming::TNamingNamedShape;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Guid, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tklcaf::guid_from_text;
use rustycad_tklcaf::json::dump_attribute_json_base;
use rustycad_tklcaf::tdf::{
    TdfAttribute, TdfAttributeBase, TdfAttributeState, TdfLabel, TdfRelocationTable, TdfTool,
};
use rustycad_tklcaf::tdoc_std::TDocStdOwner;
use rustycad_tkservice::json_dump::{class_begin, field_dumped, field_integer, field_string};
use std::io;

/// `XCAFDoc_AssemblyItemId`: the path of entries of an assembly item.
///
/// ```
/// use rustycad_tkxcaf::xcaf_doc::XCAFDocAssemblyItemId;
/// let child = XCAFDocAssemblyItemId::from_string(&"0:1:1:1/0:1:1:2".into());
/// let parent = XCAFDocAssemblyItemId::from_string(&"0:1:1:1".into());
/// assert!(child.is_child(&parent) && !child.is_direct_child(&parent));
/// assert_eq!(child.to_string_path()?, "0:1:1:1/0:1:1:2");
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct XCAFDocAssemblyItemId {
    path: Vec<AsciiString>,
}

impl XCAFDocAssemblyItemId {
    /// `XCAFDoc_AssemblyItemId()`: a null item.
    pub fn new() -> Self {
        Self::default()
    }
    /// `XCAFDoc_AssemblyItemId(thePath)`.
    pub fn from_path(path: &[AsciiString]) -> Self {
        Self {
            path: path.to_vec(),
        }
    }
    /// `XCAFDoc_AssemblyItemId(theString)`: the path of a `/`-separated string.
    pub fn from_string(text: &AsciiString) -> Self {
        let mut id = Self::new();
        id.init_from_string(text);
        id
    }
    /// `Init(thePath)`.
    pub fn init(&mut self, path: &[AsciiString]) {
        self.path = path.to_vec();
    }
    /// `Init(theString)`: the tokens of the string up to the first empty one.
    pub fn init_from_string(&mut self, text: &AsciiString) {
        self.path.clear();
        for index in 1.. {
            let entry = text.token("/", index);
            if entry.is_empty() {
                break;
            }
            self.path.push(entry);
        }
    }
    /// `IsNull`: an empty path.
    pub fn is_null(&self) -> bool {
        self.path.is_empty()
    }
    /// `Nullify`.
    pub fn nullify(&mut self) {
        self.path.clear();
    }
    /// `IsChild(theOther)`: a longer path starting with the other one.
    pub fn is_child(&self, other: &Self) -> bool {
        self.path.len() > other.path.len() && self.path.starts_with(&other.path)
    }
    /// `IsDirectChild(theOther)`: the native compares with the other path length minus one,
    /// so it never holds for a child (kept).
    pub fn is_direct_child(&self, other: &Self) -> bool {
        self.path.len() as i64 == other.path.len() as i64 - 1 && self.is_child(other)
    }
    /// `IsEqual(theOther)`: the same entries.
    pub fn is_equal(&self, other: &Self) -> bool {
        self.path == other.path
    }
    /// `GetPath`.
    pub fn get_path(&self) -> &[AsciiString] {
        &self.path
    }
    /// `ToString`: the entries joined by `/` (`Standard_OutOfRange` for a null item, whose
    /// empty string cannot lose its first character).
    pub fn to_string_path(&self) -> Result<AsciiString, Failure> {
        let mut text = AsciiString::default();
        for entry in &self.path {
            text.assign_cat("/");
            text.assign_cat(entry.as_bytes());
        }
        text.remove(1, 1)?;
        Ok(text)
    }
    /// `DumpJson`.
    pub fn dump_json(&self, stream: &mut DumpStream, _depth: i32) -> io::Result<()> {
        class_begin(stream, "XCAFDoc_AssemblyItemId")?;
        for entry in &self.path {
            field_string(stream, "aPath", entry.as_bytes())?;
        }
        Ok(())
    }
}

/// `XCAFDoc_AssemblyItemRef::GetID()`.
const ITEM_REF_ID: Guid = guid_from_text("3F2E4CD6-169B-4747-A321-5670E4291F5D");
/// The kinds of extra reference (`ExtraRef_None`, `ExtraRef_AttrGUID`, `ExtraRef_SubshapeIndex`).
const EXTRA_NONE: i32 = 0;
const EXTRA_GUID: i32 = 1;
const EXTRA_SUBSHAPE: i32 = 2;

/// The values of `XCAFDoc_AssemblyItemRef`.
#[derive(Clone, Default)]
struct ItemRefFields {
    item: XCAFDocAssemblyItemId,
    extra_ref: i32,
    extra_id: AsciiString,
}

/// `XCAFDoc_AssemblyItemRef`: a reference to an assembly item.
#[derive(Default)]
pub struct XCAFDocAssemblyItemRef {
    transient: StandardTransient,
    base: TdfAttributeBase,
    fields: SharedState<ItemRefFields>,
}
rustycad_tkernel::impl_transient!(XCAFDocAssemblyItemRef, "XCAFDoc_AssemblyItemRef", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

impl XCAFDocAssemblyItemRef {
    /// `XCAFDoc_AssemblyItemRef()`: no item, no extra reference.
    pub fn new() -> Self {
        Self::default()
    }
    /// `GetID`.
    pub fn get_id() -> Guid {
        ITEM_REF_ID
    }
    /// `Get(theLabel)`: the reference of the label, or null.
    pub fn get(label: &TdfLabel) -> Result<Handle<XCAFDocAssemblyItemRef>, Failure> {
        find::<XCAFDocAssemblyItemRef>(label, &ITEM_REF_ID)
    }
    /// The common body of the `Set` overloads: a new reference prepared by `prepare` on a
    /// label without one; null for a null label or one that has a reference.
    fn set_with(
        label: &TdfLabel,
        prepare: impl FnOnce(&Self) -> Result<(), Failure>,
    ) -> Result<Handle<XCAFDocAssemblyItemRef>, Failure> {
        if label.is_null() || label.is_attribute(&ITEM_REF_ID)? {
            return Ok(Handle::default());
        }
        let reference = Handle::new(Self::new());
        prepare(rustycad_tklcaf::live(
            &reference,
            "XCAFDoc_AssemblyItemRef::Set",
        )?)?;
        label.add_attribute(&reference.clone().cast(), false)?;
        Ok(reference)
    }
    /// `Set(theLabel, theItemId)`.
    pub fn set(
        label: &TdfLabel,
        item: &XCAFDocAssemblyItemId,
    ) -> Result<Handle<XCAFDocAssemblyItemRef>, Failure> {
        Self::set_with(label, |reference| reference.set_item(item))
    }
    /// `Set(theLabel, theItemId, theAttrGUID)`.
    pub fn set_attribute(
        label: &TdfLabel,
        item: &XCAFDocAssemblyItemId,
        guid: &Guid,
    ) -> Result<Handle<XCAFDocAssemblyItemRef>, Failure> {
        Self::set_with(label, |reference| {
            reference.set_item(item)?;
            reference.set_guid(guid)
        })
    }
    /// `Set(theLabel, theItemId, theShapeIndex)`.
    pub fn set_subshape(
        label: &TdfLabel,
        item: &XCAFDocAssemblyItemId,
        index: i32,
    ) -> Result<Handle<XCAFDocAssemblyItemRef>, Failure> {
        Self::set_with(label, |reference| {
            reference.set_item(item)?;
            reference.set_subshape_index(index)
        })
    }
    /// Backs up, then edits the values.
    fn edit(&self, change: impl FnOnce(&mut ItemRefFields)) -> Result<(), Failure> {
        self.backup()?;
        change(&mut self.fields.write());
        Ok(())
    }
}

/// The item and its extra reference.
impl XCAFDocAssemblyItemRef {
    /// `GetItem`.
    pub fn get_item(&self) -> XCAFDocAssemblyItemId {
        self.fields.read().item.clone()
    }
    /// `SetItem(theItemId)`: backed up; clears the extra reference.
    pub fn set_item(&self, item: &XCAFDocAssemblyItemId) -> Result<(), Failure> {
        self.edit(|fields| fields.item = item.clone())?;
        self.clear_extra_ref()
    }
    /// `SetItem(thePath)`.
    pub fn set_item_path(&self, path: &[AsciiString]) -> Result<(), Failure> {
        self.set_item(&XCAFDocAssemblyItemId::from_path(path))
    }
    /// `SetItem(theString)`.
    pub fn set_item_string(&self, text: &AsciiString) -> Result<(), Failure> {
        self.set_item(&XCAFDocAssemblyItemId::from_string(text))
    }
    /// `HasExtraRef`.
    pub fn has_extra_ref(&self) -> bool {
        self.fields.read().extra_ref != EXTRA_NONE
    }
    /// `IsGUID`: an attribute reference with a well-formed GUID.
    pub fn is_guid(&self) -> bool {
        let fields = self.fields.read();
        fields.extra_ref == EXTRA_GUID && Guid::check_format(Some(fields.extra_id.as_bytes()))
    }
    /// `IsSubshapeIndex`: a subshape reference with an integer.
    pub fn is_subshape_index(&self) -> bool {
        let fields = self.fields.read();
        fields.extra_ref == EXTRA_SUBSHAPE && fields.extra_id.is_integer_value()
    }
    /// `GetGUID`: the attribute GUID, else the null GUID.
    pub fn get_guid(&self) -> Guid {
        if !self.is_guid() {
            return Guid::new();
        }
        Guid::from_ascii(self.fields.read().extra_id.as_bytes()).unwrap_or_default()
    }
    /// `GetSubshapeIndex`: the subshape index, else 0.
    pub fn get_subshape_index(&self) -> i32 {
        if !self.is_subshape_index() {
            return 0;
        }
        self.fields.read().extra_id.integer_value().unwrap_or(0)
    }
    /// `SetGUID(theAttrGUID)`: backed up.
    pub fn set_guid(&self, guid: &Guid) -> Result<(), Failure> {
        let text = guid.to_c_string();
        let length = text
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(text.len());
        self.edit(|fields| {
            fields.extra_ref = EXTRA_GUID;
            fields.extra_id = AsciiString::from(&text[..length]);
        })
    }
    /// `SetSubshapeIndex(theShapeIndex)`: backed up.
    pub fn set_subshape_index(&self, index: i32) -> Result<(), Failure> {
        self.edit(|fields| {
            fields.extra_ref = EXTRA_SUBSHAPE;
            fields.extra_id = AsciiString::from_integer(index);
        })
    }
    /// `ClearExtraRef`: backed up.
    pub fn clear_extra_ref(&self) -> Result<(), Failure> {
        self.edit(|fields| {
            fields.extra_ref = EXTRA_NONE;
            fields.extra_id = AsciiString::default();
        })
    }
}

/// The orphan test.
impl XCAFDocAssemblyItemRef {
    /// `IsOrphan`: no item, no owning document, no label for the last entry of the item, or
    /// a missing referred attribute or subshape.
    pub fn is_orphan(&self) -> Result<bool, Failure> {
        let item = self.get_item();
        let Some(entry) = item.get_path().last() else {
            return Ok(true);
        };
        let Some(label) = self.item_label(entry)? else {
            return Ok(true);
        };
        if self.is_guid() {
            return Ok(!label.is_attribute(&self.get_guid())?);
        }
        if self.is_subshape_index() {
            return Ok(!self.has_subshape(&label)?);
        }
        Ok(false)
    }
    /// The label of the entry in the owning document, `None` without one.
    fn item_label(&self, entry: &AsciiString) -> Result<Option<TdfLabel>, Failure> {
        let root = self.label().root()?;
        let owner = find::<TDocStdOwner>(&root, &TDocStdOwner::get_id())?;
        let document = owner
            .get()
            .map(TDocStdOwner::get_document)
            .unwrap_or_default();
        let Some(document) = document.get() else {
            return Ok(None);
        };
        let data = document.get_data();
        if data.is_null() {
            return Ok(None);
        }
        let mut label = TdfLabel::default();
        TdfTool::label_from_entry(&data, entry, &mut label, false)?;
        Ok((!label.is_null()).then_some(label))
    }
    /// The named shape of the label has the referred subshape index.
    fn has_subshape(&self, label: &TdfLabel) -> Result<bool, Failure> {
        let named = find::<TNamingNamedShape>(label, &TNamingNamedShape::get_id())?;
        let Some(named) = named.get() else {
            return Ok(false);
        };
        let mut map = IndexedMapOfShape::default();
        topexp::map_shapes(&named.get()?, &mut map, true, true);
        let index = self.get_subshape_index();
        Ok(index >= 1 && map.extent() >= index as usize)
    }
}

impl TdfAttribute for XCAFDocAssemblyItemRef {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        ITEM_REF_ID
    }
    /// `Restore`: the values of another reference.
    fn restore(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        if let Some(other) = with.cast::<XCAFDocAssemblyItemRef>().get() {
            *self.fields.write() = other.fields.read().clone();
        }
        Ok(())
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::new()).cast()
    }
    /// `Paste`: the values into another reference (not backed up, as natively).
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        _table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        if let Some(other) = into.cast::<XCAFDocAssemblyItemRef>().get() {
            *other.fields.write() = self.fields.read().clone();
        }
        Ok(())
    }
    /// `Dump`: `Path: <path>`, then the GUID or the subshape index (`Standard_OutOfRange`,
    /// reported as an I/O error, for a null item).
    fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        let path = self.get_item().to_string_path().map_err(io::Error::other)?;
        stream.write_all(b"Path: ")?;
        stream.write_all(path.as_bytes())?;
        let extra = self.fields.read().extra_id.clone();
        if self.is_guid() {
            stream.write_all(b"/GUID:")?;
            stream.write_all(extra.as_bytes())?;
        } else if self.is_subshape_index() {
            stream.write_all(b"/Subshape: ")?;
            stream.write_all(extra.as_bytes())?;
        }
        Ok(())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "XCAFDoc_AssemblyItemRef")?;
        dump_attribute_json_base(self, stream, depth)?;
        let fields = self.fields.read().clone();
        field_dumped(stream, depth, "&myItemId", &|s, d| {
            fields.item.dump_json(s, d)
        })?;
        field_integer(stream, "myExtraRef", fields.extra_ref)?;
        field_string(stream, "myExtraId", fields.extra_id.as_bytes())
    }
}
