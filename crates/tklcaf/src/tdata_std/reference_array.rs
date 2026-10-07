// Rust port of ApplicationFramework/TKLCAF/TDataStd/TDataStd_ReferenceArray.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDataStd_ReferenceArray`: an array of labels, relocated on paste.
use super::array_core::{
    ArrayAttribute, ArrayState, array, init, length_of, lower_of, new_default_array, restore,
    set_attr, set_default_id, set_id, set_value, upper_of,
};
use crate::guid_from_text;
use crate::json::dump_attribute_json_base;
use crate::live;
use crate::tdf::{
    TdfAttribute, TdfAttributeBase, TdfAttributeState, TdfDataSet, TdfLabel, TdfRelocationTable,
    TdfTool,
};
use rustycad_tkcdf::json_dump::{class_begin, field_integer, field_string};
use rustycad_tkernel::{
    impl_transient,
    ncollection::HArray1,
    standard::{Failure, Guid, Handle, SharedState, StandardTransient, dump::DumpStream},
    tcollection::AsciiString,
};
use std::io;

/// `TDataStd_ReferenceArray::GetID()`.
const REFERENCE_ARRAY_ID: Guid = guid_from_text("7EE745A6-BB50-446c-BB0B-C195B23AB5CA");

/// `TDataStd_ReferenceArray`; `TDataStdReferenceArray::set(&label, 1, 2)`.
///
/// ```
/// use rustycad_tklcaf::{tdata_std::TDataStdReferenceArray, tdf::TdfData};
/// let data = TdfData::new();
/// let array = TDataStdReferenceArray::set(&data.root(), 1, 2).unwrap();
/// array.set_value(1, data.root().find_child(3, true).unwrap()).unwrap();
/// assert_eq!(array.value(1).unwrap().tag(), 3);
/// ```
pub struct TDataStdReferenceArray {
    transient: StandardTransient,
    base: TdfAttributeBase,
    state: SharedState<ArrayState<TdfLabel>>,
}
impl_transient!(TDataStdReferenceArray, "TDataStd_ReferenceArray", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

impl Default for TDataStdReferenceArray {
    /// `TDataStd_ReferenceArray()`: no array and the default ID.
    fn default() -> Self {
        Self {
            transient: StandardTransient::default(),
            base: TdfAttributeBase::default(),
            state: SharedState::new(ArrayState::new(REFERENCE_ARRAY_ID)),
        }
    }
}

impl ArrayAttribute for TDataStdReferenceArray {
    type Item = TdfLabel;
    const FILL: fn() -> TdfLabel = TdfLabel::default;
    const NULL_VALUE: fn() -> TdfLabel = TdfLabel::default;
    const NULL_UPPER: i32 = -1;
    const CHANGE_GUARDS_NULL: bool = false;
    const SET_ID_FIRST: bool = false;
    const DEFAULT_ID: Guid = REFERENCE_ARRAY_ID;
    fn make() -> Self {
        Self::default()
    }
    fn array_state(&self) -> &SharedState<ArrayState<TdfLabel>> {
        &self.state
    }
    fn dump_item(stream: &mut DumpStream, item: &TdfLabel) -> io::Result<()> {
        let mut entry = AsciiString::default();
        TdfTool::entry(item, &mut entry);
        field_string(stream, "aLabel", &String::from_utf8_lossy(entry.as_bytes()))
    }
}

impl TDataStdReferenceArray {
    /// `GetID`: the default ID.
    pub fn get_id() -> Guid {
        REFERENCE_ARRAY_ID
    }
    /// `TDataStd_ReferenceArray()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Set(label, lower, upper)`: the attribute of the default ID.
    pub fn set(label: &TdfLabel, lower: i32, upper: i32) -> Result<Handle<Self>, Failure> {
        set_attr(label, &REFERENCE_ARRAY_ID, (lower, upper), false)
    }
    /// `Set(label, guid, lower, upper)`: the attribute of a user ID.
    pub fn set_with_id(
        label: &TdfLabel,
        guid: &Guid,
        lower: i32,
        upper: i32,
    ) -> Result<Handle<Self>, Failure> {
        set_attr(label, guid, (lower, upper), false)
    }
    /// `Init(lower, upper)`: backs up, then an array of null labels.
    pub fn init(&self, lower: i32, upper: i32) -> Result<(), Failure> {
        init(self, lower, upper)
    }
    /// `SetValue(index, value)`: backed up when the label changes.
    pub fn set_value(&self, index: i32, value: TdfLabel) -> Result<(), Failure> {
        set_value(self, index, value)
    }
    /// `Value(index)` and `operator()`; `Standard_NullObject` without array (a null
    /// dereference natively), `Standard_OutOfRange` outside the bounds.
    pub fn value(&self, index: i32) -> Result<TdfLabel, Failure> {
        let shared = array(self);
        let items = live(&shared, "TDataStd_ReferenceArray::Value")?.array1();
        items
            .value(index)
            .cloned()
            .map_err(|e| e.into_failure("TDataStd_ReferenceArray::Value"))
    }
    /// `Lower`: 0 without array.
    pub fn lower(&self) -> i32 {
        lower_of(self)
    }
    /// `Upper`: -1 without array.
    pub fn upper(&self) -> i32 {
        upper_of(self)
    }
    /// `Length`: 0 without array.
    pub fn length(&self) -> i32 {
        length_of(self)
    }
    /// `InternalArray`: the labels, shared with the attribute.
    pub fn internal_array(&self) -> Handle<HArray1<TdfLabel>> {
        array(self)
    }
    /// `SetInternalArray(values, isCheckItems)`: backs up, then a new array with the
    /// labels of `values`. The item check is compiled out natively (`OCC2932` is not
    /// defined), so `check_items` is ignored and the copy always happens.
    pub fn set_internal_array(
        &self,
        values: &Handle<HArray1<TdfLabel>>,
        _check_items: bool,
    ) -> Result<(), Failure> {
        let labels: Vec<TdfLabel> = live(values, "TDataStd_ReferenceArray::SetInternalArray")?
            .array1()
            .iter()
            .cloned()
            .collect();
        let lower = live(values, "TDataStd_ReferenceArray::SetInternalArray")?
            .array1()
            .lower();
        self.backup()?;
        let target = new_default_array::<Self>(lower, lower + labels.len() as i32 - 1)?;
        write_labels(&target, lower, labels)?;
        self.state.write().value = target;
        Ok(())
    }
}

/// Writes `labels` into `target` from index `lower` on.
fn write_labels(
    target: &Handle<HArray1<TdfLabel>>,
    lower: i32,
    labels: Vec<TdfLabel>,
) -> Result<(), Failure> {
    let mut items = live(target, "TDataStd_ReferenceArray")?.change_array1();
    for (index, label) in (lower..).zip(labels) {
        items
            .set_value(index, label)
            .map_err(|e| e.into_failure("TDataStd_ReferenceArray"))?;
    }
    Ok(())
}

impl TdfAttribute for TDataStdReferenceArray {
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
    fn restore(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        restore(self, with)
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::new()).cast()
    }
    /// `Paste(Into, RT)`: without array, `Into` loses its array; else `Into` takes the
    /// bounds, the non-null labels (relocated when the table knows them) and the ID.
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        let target = into.cast::<Self>();
        let target = live(&target, "TDataStd_ReferenceArray::Paste")?;
        let shared = array(self);
        let Some(items) = shared.get() else {
            target.state.write().value = Handle::default();
            return Ok(());
        };
        let (lower, labels): (i32, Vec<TdfLabel>) = {
            let items = items.array1();
            (items.lower(), items.iter().cloned().collect())
        };
        if lower != target.lower() || lower + labels.len() as i32 - 1 != target.upper() {
            target.init(lower, lower + labels.len() as i32 - 1)?;
        }
        paste_labels(&array(target), lower, labels, table)?;
        target.set_id(&self.id())
    }
    /// `References`: every non-null label, unless the label is imported.
    fn references(&self, data_set: &Handle<TdfDataSet>) {
        let (false, Some(data_set)) = (self.label().is_imported(), data_set.get()) else {
            return;
        };
        let shared = array(self);
        let Some(items) = shared.get() else {
            return;
        };
        for label in items.array1().iter().filter(|label| !label.is_null()) {
            data_set.add_label(label);
        }
    }
    /// `Dump`: `"\nReferenceArray: <ID>\n"`.
    fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        writeln!(stream, "\nReferenceArray: {}", self.id())
    }
    /// `DumpJson`: the bounds and the label entries.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "TDataStd_ReferenceArray")?;
        dump_attribute_json_base(self, stream, depth)?;
        let shared = array(self);
        let Some(items) = shared.get() else {
            return Ok(());
        };
        let items = items.array1();
        field_integer(stream, "myArray->Lower()", items.lower())?;
        field_integer(stream, "myArray->Upper()", items.upper())?;
        for label in items.iter() {
            Self::dump_item(stream, label)?;
        }
        Ok(())
    }
}

/// The relocation loop of `Paste`: a null label leaves the target item unchanged.
fn paste_labels(
    target: &Handle<HArray1<TdfLabel>>,
    lower: i32,
    labels: Vec<TdfLabel>,
    table: &Handle<TdfRelocationTable>,
) -> Result<(), Failure> {
    for (index, label) in (lower..).zip(labels).filter(|(_, label)| !label.is_null()) {
        let mut relocated = TdfLabel::default();
        let table = live(table, "TDataStd_ReferenceArray::Paste")?;
        let value = if table.has_label_relocation(&label, &mut relocated) {
            relocated
        } else {
            label
        };
        let mut items = live(target, "TDataStd_ReferenceArray::Paste")?.change_array1();
        items
            .set_value(index, value)
            .map_err(|e| e.into_failure("TDataStd_ReferenceArray::Paste"))?;
    }
    Ok(())
}
