// Rust port of ApplicationFramework/TKCAF/TNaming/TNaming_Naming.hxx and the attribute
// members of TNaming_Naming.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TNaming_Naming`: the attribute that stores a `TNaming_Name` on the label of a
//! selection. Building a name (`Name`) and solving it (`Solve`, `Regenerate`) are in
//! `naming_build.rs` and `naming_solve.rs`.
use super::TNamingName;
use rustycad_tkcdf::json_dump::{class_begin, field_dumped};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Guid, Handle, SharedState, StandardTransient, dump::DumpStream},
};
use rustycad_tklcaf::guid_from_text;
use rustycad_tklcaf::json::{AttributeIndexMap, dump_attribute_json_base};
use rustycad_tklcaf::live;
use rustycad_tklcaf::tdata_std::support::cast_attribute;
use rustycad_tklcaf::tdf::{
    TdfAttribute, TdfAttributeBase, TdfDataSet, TdfIdFilter, TdfLabel, TdfRelocationTable,
    TdfTagSource,
};
use std::io;
use std::sync::RwLockWriteGuard;

/// `TNaming_Naming::GetID()`.
const NAMING_ID: Guid = guid_from_text("c0a19201-5b78-11d1-8940-080009dc3333");

/// `TNaming_Naming`.
///
/// ```
/// use rustycad_tkcaf::tnaming::{TNamingNameType, TNamingNaming};
/// let naming = TNamingNaming::new();
/// assert!(!naming.is_defined());
/// naming.change_name().set_type(TNamingNameType::Identity);
/// assert!(naming.is_defined());
/// ```
pub struct TNamingNaming {
    transient: StandardTransient,
    base: TdfAttributeBase,
    name: SharedState<TNamingName>,
}
impl_transient!(TNamingNaming, "TNaming_Naming", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

impl Default for TNamingNaming {
    fn default() -> Self {
        Self::new()
    }
}

impl TNamingNaming {
    /// `GetID`.
    pub fn get_id() -> Guid {
        NAMING_ID
    }
    /// `Insert(under)`: a new naming on a new child of `under` (`TDF_TagSource::NewChild`).
    pub fn insert(under: &TdfLabel) -> Result<Handle<TNamingNaming>, Failure> {
        let child = TdfTagSource::new_child_of(under)?;
        let naming = Handle::new(TNamingNaming::new());
        child.add_attribute(&naming.cast(), true)?;
        Ok(naming)
    }
    /// `TNaming_Naming()`.
    pub fn new() -> Self {
        Self {
            transient: StandardTransient::default(),
            base: TdfAttributeBase::default(),
            name: SharedState::new(TNamingName::new()),
        }
    }
    /// `IsDefined`: the name's type is not `UNKNOWN`.
    pub fn is_defined(&self) -> bool {
        self.name.read().name_type() != super::TNamingNameType::Unknown
    }
    /// `GetName`: a copy of the name.
    pub fn get_name(&self) -> TNamingName {
        self.name.read().clone()
    }
    /// `ChangeName`: the name itself; drop the guard before calling back into the
    /// framework.
    pub fn change_name(&self) -> RwLockWriteGuard<'_, TNamingName> {
        self.name.write()
    }
}

impl TdfAttribute for TNamingNaming {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        NAMING_ID
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(TNamingNaming::new()).cast()
    }
    /// `Restore`: the other naming's name.
    fn restore(&self, other: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        let other = cast_attribute::<TNamingNaming>(other, "TNaming_Naming::Restore")?;
        let name = live(&other, "TNaming_Naming::Restore")?.get_name();
        *self.name.write() = name;
        Ok(())
    }
    /// `Paste`: `TNaming_Name::Paste` into the target's name.
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        relocation_table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        let target = cast_attribute::<TNamingNaming>(into, "TNaming_Naming::Paste")?;
        let table = live(relocation_table, "TNaming_Naming::Paste")?;
        let name = self.get_name();
        name.paste(
            &mut live(&target, "TNaming_Naming::Paste")?.change_name(),
            table,
        );
        Ok(())
    }
    /// `References`: the arguments and the stop named shape of the name.
    fn references(&self, data_set: &Handle<TdfDataSet>) {
        let Some(data_set) = data_set.get() else {
            return;
        };
        let name = self.get_name();
        for argument in name.arguments() {
            data_set.add_attribute(&argument.cast());
        }
        let stop = name.stop_named_shape();
        if stop.is_some() {
            data_set.add_attribute(&stop.cast());
        }
    }
    /// `Dump`: `TNaming_Naming`.
    fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        write!(stream, "TNaming_Naming")
    }
    /// `ExtendedDump`: `TNaming_Naming ExtendedDump  `.
    fn extended_dump(
        &self,
        stream: &mut dyn io::Write,
        _filter: &TdfIdFilter,
        _map: &mut AttributeIndexMap,
    ) -> io::Result<()> {
        write!(stream, "TNaming_Naming ExtendedDump  ")
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "TNaming_Naming")?;
        dump_attribute_json_base(self, stream, depth)?;
        let name = self.get_name();
        let child = |s: &mut DumpStream, d: i32| name.dump_json(s, d);
        field_dumped(stream, depth, "&myName", Some(&child))
    }
}
