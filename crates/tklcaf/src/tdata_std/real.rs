// Rust port of ApplicationFramework/TKLCAF/TDataStd/TDataStd_Real.hxx, .cxx and
// TDataStd_RealEnum.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDataStd_Real`: a real value attribute with a user ID and a (deprecated) dimension.
use super::TDataStd;
use super::support::{
    backup_as_dynamic, cast_attribute, find_or_add, reset_user_id, set_backed_up, set_user_id,
};
use crate::guid_from_text;
use crate::json::dump_attribute_json_base;
use crate::tdf::{
    TdfAttribute, TdfAttributeBase, TdfAttributeState, TdfLabel, TdfReference, TdfRelocationTable,
};
use rustycad_tkcdf::json_dump::{class_begin, field_integer, field_real};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Guid, Handle, SharedState, StandardTransient, dump::DumpStream},
    tcollection::AsciiString,
};
use std::io;

/// `TDataStd_Real::GetID()`.
const REAL_ID: Guid = guid_from_text("2a96b60f-ec8b-11d0-bee7-080009dc3333");

/// `TDataStd_RealEnum`: the dimension of a real, in native order.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TDataStdRealEnum {
    /// `TDataStd_SCALAR`.
    #[default]
    Scalar,
    /// `TDataStd_LENGTH`.
    Length,
    /// `TDataStd_ANGULAR`.
    Angular,
}

#[derive(Clone, Copy, PartialEq)]
struct RealState {
    value: f64,
    dimension: TDataStdRealEnum,
}

/// `TDataStd_Real`; `TDataStdReal::set(&label, 1.5)`.
///
/// ```
/// use rustycad_tklcaf::{tdata_std::TDataStdReal, tdf::TdfData};
/// let data = TdfData::new();
/// let attribute = TDataStdReal::set(&data.root(), 1.5).unwrap();
/// assert_eq!(TDataStdReal::get(&attribute), 1.5);
/// ```
pub struct TDataStdReal {
    transient: StandardTransient,
    base: TdfAttributeBase,
    value: SharedState<f64>,
    dimension: SharedState<TDataStdRealEnum>,
    id: SharedState<Guid>,
}
impl_transient!(TDataStdReal, "TDataStd_Real", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

impl Default for TDataStdReal {
    /// `TDataStd_Real()`: `RealFirst()` (the most negative double), scalar, default ID.
    fn default() -> Self {
        Self {
            transient: StandardTransient::default(),
            base: TdfAttributeBase::default(),
            value: SharedState::new(f64::MIN),
            dimension: SharedState::new(TDataStdRealEnum::Scalar),
            id: SharedState::new(REAL_ID),
        }
    }
}

impl TDataStdReal {
    /// `GetID`: the default ID.
    pub fn get_id() -> Guid {
        REAL_ID
    }
    /// `TDataStd_Real()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Set(label, value)`: finds or adds the attribute of the default ID and sets it.
    pub fn set(label: &TdfLabel, value: f64) -> Result<Handle<TDataStdReal>, Failure> {
        Self::set_with_id(label, &REAL_ID, value)
    }
    /// `Set(label, guid, value)`: the same with a user ID.
    pub fn set_with_id(
        label: &TdfLabel,
        guid: &Guid,
        value: f64,
    ) -> Result<Handle<TDataStdReal>, Failure> {
        let attribute = find_or_add(label, guid, true, Self::new)?;
        crate::live(&attribute, "TDataStd_Real::Set")?.set_value(value)?;
        Ok(attribute)
    }
    /// `Set(V)`: backed up when it changes (OCC2932; `==` on doubles, as natively). The
    /// backup goes through the attribute's own handle, so that a derived attribute
    /// (`XCAFDoc_Area`, `XCAFDoc_Volume`) backs up as itself (the virtual `BackupCopy`).
    pub fn set_value(&self, value: f64) -> Result<(), Failure> {
        if *self.value.read() == value {
            return Ok(());
        }
        backup_as_dynamic(self)?;
        *self.value.write() = value;
        Ok(())
    }
    /// `Get`; `TDataStdReal::get(&attribute)`.
    pub fn get(&self) -> f64 {
        *self.value.read()
    }
    /// `SetDimension` (deprecated natively): backed up when it changes.
    pub fn set_dimension(&self, dimension: TDataStdRealEnum) -> Result<(), Failure> {
        set_backed_up(self, &self.dimension, &dimension)
    }
    /// `GetDimension` (deprecated natively).
    pub fn get_dimension(&self) -> TDataStdRealEnum {
        *self.dimension.read()
    }
    /// `IsCaptured`: the label holds a `TDF_Reference` whose label has an attribute of this ID.
    pub fn is_captured(&self) -> Result<bool, Failure> {
        let mut reference: Handle<TdfReference> = Handle::default();
        if !self
            .label()
            .find_attribute_as(&TdfReference::get_id(), &mut reference)?
        {
            return Ok(false);
        }
        let origin = crate::live(&reference, "TDataStd_Real::IsCaptured")?.get();
        origin.is_attribute(&self.id())
    }
    fn state(&self) -> RealState {
        RealState {
            value: self.get(),
            dimension: self.get_dimension(),
        }
    }
}

impl TdfAttribute for TDataStdReal {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        *self.id.read()
    }
    fn set_id(&self, guid: &Guid) -> Result<(), Failure> {
        set_user_id(self, &self.id, guid)
    }
    fn set_default_id(&self) -> Result<(), Failure> {
        reset_user_id(self, &self.id, REAL_ID)
    }
    fn restore(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        let other = cast_attribute::<TDataStdReal>(with, "TDataStd_Real::Restore")?;
        let other = crate::live(&other, "TDataStd_Real::Restore")?;
        let state = other.state();
        let id = other.id();
        *self.value.write() = state.value;
        *self.dimension.write() = state.dimension;
        *self.id.write() = id;
        Ok(())
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::new()).cast()
    }
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        _relocation_table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        let target = cast_attribute::<TDataStdReal>(into, "TDataStd_Real::Paste")?;
        let target = crate::live(&target, "TDataStd_Real::Paste")?;
        target.set_value(self.get())?;
        target.set_dimension(self.get_dimension())?;
        target.set_id(&self.id())
    }
    /// `Dump`: `Real <dimension><value><guid>` with the default stream precision.
    fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        write!(stream, "Real ")?;
        TDataStd::print(self.get_dimension(), stream)?;
        let value = AsciiString::from_real(self.get());
        stream.write_all(value.as_bytes())?;
        write!(stream, "{}", self.id())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "TDataStd_Real")?;
        dump_attribute_json_base(self, stream, depth)?;
        field_real(stream, "myValue", self.get())?;
        field_integer(stream, "myDimension", self.get_dimension() as i32)
    }
}
