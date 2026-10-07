// Rust port of ApplicationFramework/TKLCAF/TDataStd/TDataStd_BooleanArray.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDataStd_BooleanArray`: booleans packed eight per byte in a `uint8_t` array.
use super::support::cast_attribute;
use crate::guid_from_text;
use crate::json::dump_attribute_json_base;
use crate::live;
use crate::tdf::{TdfAttribute, TdfAttributeBase, TdfAttributeState, TdfLabel, TdfRelocationTable};
use rustycad_tkcdf::json_dump::{class_begin, field_byte_char, field_integer};
use rustycad_tkernel::{
    impl_transient,
    ncollection::HArray1,
    standard::{
        Failure, Guid, Handle, SharedState, StandardTransient, dump::DumpStream, exceptions,
    },
};
use std::io;

/// `TDataStd_BooleanArray::GetID()`.
const BOOLEAN_ARRAY_ID: Guid = guid_from_text("C7E98E54-B5EA-4aa9-AC99-9164EBD07F10");

/// The native fields `myValues`, `myLower`, `myUpper` and `myID`.
struct BooleanArrayState {
    values: Handle<HArray1<u8>>,
    lower: i32,
    upper: i32,
    id: Guid,
}

/// `TDataStd_BooleanArray`; `TDataStdBooleanArray::set(&label, 1, 10)`.
///
/// ```
/// use rustycad_tklcaf::{tdata_std::TDataStdBooleanArray, tdf::TdfData};
/// let data = TdfData::new();
/// let array = TDataStdBooleanArray::set(&data.root(), 1, 10).unwrap();
/// array.set_value(9, true).unwrap();
/// assert!(array.value(9).unwrap() && !array.value(8).unwrap());
/// ```
pub struct TDataStdBooleanArray {
    transient: StandardTransient,
    base: TdfAttributeBase,
    state: SharedState<BooleanArrayState>,
}
impl_transient!(TDataStdBooleanArray, "TDataStd_BooleanArray", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

impl Default for TDataStdBooleanArray {
    /// `TDataStd_BooleanArray()`: the default ID and no array. The native constructor
    /// leaves `myLower` and `myUpper` uninitialised; the port starts with the empty
    /// range 0..-1.
    fn default() -> Self {
        Self {
            transient: StandardTransient::default(),
            base: TdfAttributeBase::default(),
            state: SharedState::new(BooleanArrayState {
                values: Handle::default(),
                lower: 0,
                upper: -1,
                id: BOOLEAN_ARRAY_ID,
            }),
        }
    }
}

/// The byte index and the bit mask of an index (the native `DegreeOf2`).
fn bit_of(index: i32, lower: i32) -> (i32, u8) {
    let byte_index = (index - lower) >> 3;
    let degree = index - (byte_index << 3) - lower;
    (byte_index, 1u8 << degree)
}

impl TDataStdBooleanArray {
    /// `GetID`: the default ID.
    pub fn get_id() -> Guid {
        BOOLEAN_ARRAY_ID
    }
    /// `TDataStd_BooleanArray()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Set(label, lower, upper)`: the attribute of the default ID.
    pub fn set(label: &TdfLabel, lower: i32, upper: i32) -> Result<Handle<Self>, Failure> {
        Self::set_with_id(label, &BOOLEAN_ARRAY_ID, lower, upper)
    }
    /// `Set(label, guid, lower, upper)`: the file-static `SetAttr`; a new attribute is
    /// given the ID, then initialized; an existing one is re-initialized when its bounds
    /// differ.
    pub fn set_with_id(
        label: &TdfLabel,
        guid: &Guid,
        lower: i32,
        upper: i32,
    ) -> Result<Handle<Self>, Failure> {
        let mut found: Handle<Self> = Handle::default();
        if label.find_attribute_as(guid, &mut found)? {
            let current = live(&found, "TDataStd_BooleanArray::Set")?;
            if lower != current.lower() || upper != current.upper() {
                current.init(lower, upper)?;
            }
            return Ok(found);
        }
        let created = Handle::new(Self::new());
        let attribute = live(&created, "TDataStd_BooleanArray::Set")?;
        attribute.set_id(guid)?;
        attribute.init(lower, upper)?;
        label.add_attribute(&created.cast(), false)?;
        Ok(created)
    }
    /// `Init(lower, upper)`: backs up, then `Length() >> 3` + 1 zero bytes;
    /// `Standard_RangeError` when `upper < lower`.
    pub fn init(&self, lower: i32, upper: i32) -> Result<(), Failure> {
        if upper < lower {
            let message =
                format!("TDataStd_BooleanArray::Init: upper {upper} is below lower {lower}");
            return Err(exceptions::range_error(&message));
        }
        self.backup()?;
        let bytes = HArray1::with_value(0, (upper - lower + 1) >> 3, 0u8)
            .map_err(|e| e.into_failure("TDataStd_BooleanArray::Init"))?;
        let mut state = self.state.write();
        (state.lower, state.upper, state.values) = (lower, upper, Handle::new(bytes));
        Ok(())
    }
    /// `SetValue(index, value)`: nothing without array or when the bit already holds the
    /// value; else backs up and sets or toggles the bit. The index is not checked against
    /// the bounds, as natively; a byte outside the array is `Standard_OutOfRange`.
    pub fn set_value(&self, index: i32, value: bool) -> Result<(), Failure> {
        let (values, lower) = {
            let s = self.state.read();
            (s.values.clone(), s.lower)
        };
        let Some(bytes) = values.get() else {
            return Ok(());
        };
        let (byte_index, mask) = bit_of(index, lower);
        let byte = *bytes
            .array1()
            .value(byte_index)
            .map_err(|e| e.into_failure("TDataStd_BooleanArray::SetValue"))?;
        if value == (byte & mask != 0) {
            return Ok(());
        }
        self.backup()?;
        let updated = if value { byte | mask } else { byte ^ mask };
        let mut bytes = bytes.change_array1();
        bytes
            .set_value(byte_index, updated)
            .map_err(|e| e.into_failure("TDataStd_BooleanArray::SetValue"))
    }
    /// `Value(index)` and `operator()`: false without array or outside the bounds.
    pub fn value(&self, index: i32) -> Result<bool, Failure> {
        let state = self.state.read();
        let Some(bytes) = state
            .values
            .get()
            .filter(|_| (state.lower..=state.upper).contains(&index))
        else {
            return Ok(false);
        };
        let (byte_index, mask) = bit_of(index, state.lower);
        let bytes = bytes.array1();
        let byte = bytes
            .value(byte_index)
            .map_err(|e| e.into_failure("TDataStd_BooleanArray::Value"))?;
        Ok(byte & mask != 0)
    }
    /// `Lower`.
    pub fn lower(&self) -> i32 {
        self.state.read().lower
    }
    /// `Upper`.
    pub fn upper(&self) -> i32 {
        self.state.read().upper
    }
    /// `Length`: `Upper - Lower + 1`.
    pub fn length(&self) -> i32 {
        let state = self.state.read();
        state.upper - state.lower + 1
    }
    /// `InternalArray`: the packed bytes, shared with the attribute.
    pub fn internal_array(&self) -> Handle<HArray1<u8>> {
        self.state.read().values.clone()
    }
    /// `SetInternalArray`: shares `values` as the packed bytes (no backup, as natively).
    pub fn set_internal_array(&self, values: &Handle<HArray1<u8>>) {
        self.state.write().values = values.clone();
    }
    /// The packed bytes in order, for `DumpJson`.
    fn bytes(&self) -> Option<(i32, i32, Vec<u8>)> {
        let values = self.internal_array();
        let bytes = values.get()?.array1();
        Some((
            bytes.lower(),
            bytes.upper(),
            bytes.iter().copied().collect(),
        ))
    }
}

impl TdfAttribute for TDataStdBooleanArray {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        self.state.read().id
    }
    fn set_id(&self, guid: &Guid) -> Result<(), Failure> {
        if self.state.read().id == *guid {
            return Ok(());
        }
        self.backup()?;
        self.state.write().id = *guid;
        Ok(())
    }
    fn set_default_id(&self) -> Result<(), Failure> {
        self.backup()?;
        self.state.write().id = BOOLEAN_ARRAY_ID;
        Ok(())
    }
    /// `Restore(With)`: a copy of the bounds, bytes and ID; without bytes, only the array
    /// is dropped (the bounds and ID stay), as natively.
    fn restore(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        let other = cast_attribute::<Self>(with, "TDataStd_BooleanArray::Restore")?;
        let other = live(&other, "TDataStd_BooleanArray::Restore")?;
        let Some((_, _, bytes)) = other.bytes() else {
            self.state.write().values = Handle::default();
            return Ok(());
        };
        let (lower, upper, id) = {
            let s = other.state.read();
            (s.lower, s.upper, s.id)
        };
        let copy = restored_bytes(&bytes, (upper - lower + 1) >> 3)?;
        let mut state = self.state.write();
        (state.lower, state.upper, state.values, state.id) = (lower, upper, copy, id);
        Ok(())
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::new()).cast()
    }
    /// `Paste(Into, RT)`: with bytes, `Into` is initialized to the bounds and takes every
    /// value and the ID; `Standard_NullObject` when `Into` is not a boolean array.
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        _table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        if self.internal_array().is_null() {
            return Ok(());
        }
        let target = into.cast::<Self>();
        if let Some(target) = target.get() {
            target.init(self.lower(), self.upper())?;
            for index in self.lower()..=self.upper() {
                target.set_value(index, self.value(index)?)?;
            }
        }
        live(&target, "TDataStd_BooleanArray::Paste")?.set_id(&self.id())
    }
    /// `Dump`: `"\nBooleanArray: <ID>\n"`.
    fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        writeln!(stream, "\nBooleanArray: {}", self.id())
    }
    /// `DumpJson`: the bytes (each streamed as a character), then the bounds.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "TDataStd_BooleanArray")?;
        dump_attribute_json_base(self, stream, depth)?;
        if let Some((lower, upper, bytes)) = self.bytes() {
            field_integer(stream, "myValues->Lower()", lower)?;
            field_integer(stream, "myValues->Upper()", upper)?;
            for byte in bytes {
                field_byte_char(stream, "aValue", byte)?;
            }
        }
        field_integer(stream, "myLower", self.lower())?;
        field_integer(stream, "myUpper", self.upper())
    }
}

/// The bytes `0..=byte_upper` of `Restore`, copied from the other array.
fn restored_bytes(bytes: &[u8], byte_upper: i32) -> Result<Handle<HArray1<u8>>, Failure> {
    let failure = |e: rustycad_tkernel::ncollection::ArrayError| {
        e.into_failure("TDataStd_BooleanArray::Restore")
    };
    let copy = HArray1::with_value(0, byte_upper, 0u8).map_err(failure)?;
    for (index, byte) in (0..=byte_upper).zip(bytes) {
        copy.change_array1()
            .set_value(index, *byte)
            .map_err(failure)?;
    }
    Ok(Handle::new(copy))
}
