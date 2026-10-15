// Rust port of ApplicationFramework/TKTObj/TObj/TObj_TNameContainer.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TObj_TNameContainer`: the OCAF attribute with the unique names of a model's objects.
use rustycad_tkernel::{
    impl_transient,
    ncollection::DataMap,
    standard::{Failure, Guid, Handle, SharedState, StandardTransient},
    tcollection::{HExtendedString, HExtendedStringHasher},
};
use rustycad_tklcaf::{
    guid_from_text,
    tdata_std::support::{cast_attribute, find_or_add},
    tdf::{TdfAttribute, TdfAttributeBase, TdfAttributeState, TdfLabel, TdfRelocationTable},
};

/// `TObj_TNameContainer::GetID()`.
const NAME_CONTAINER_ID: Guid = guid_from_text("3bbefb47-e618-11d4-ba38-0060b0ee18ea");

/// The map of names: `NCollection_DataMap<handle<TCollection_HExtendedString>, TDF_Label>`,
/// hashed and compared by the string (the native handle specialization).
pub type TObjNameMap = DataMap<Handle<HExtendedString>, TdfLabel, HExtendedStringHasher>;

/// `TObj_TNameContainer`; `TObjTNameContainer::set(&label)`.
pub struct TObjTNameContainer {
    transient: StandardTransient,
    base: TdfAttributeBase,
    map: SharedState<TObjNameMap>,
}
impl_transient!(TObjTNameContainer, "TObj_TNameContainer", dyn TdfAttribute, transient,
    interfaces: [dyn TdfAttribute]);

impl Default for TObjTNameContainer {
    fn default() -> Self {
        Self {
            transient: StandardTransient::default(),
            base: TdfAttributeBase::default(),
            map: SharedState::new(TObjNameMap::with_hasher(HExtendedStringHasher, 1)),
        }
    }
}

impl TObjTNameContainer {
    /// `GetID`.
    pub fn get_id() -> Guid {
        NAME_CONTAINER_ID
    }
    /// `TObj_TNameContainer()`: no name.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Set(theLabel)` (static): the container of the label, added when absent.
    pub fn set(label: &TdfLabel) -> Result<Handle<TObjTNameContainer>, Failure> {
        find_or_add(label, &NAME_CONTAINER_ID, false, Self::new)
    }
    /// `RecordName(theName, theLabel)`: records a copy of the name unless registered.
    pub fn record_name(
        &self,
        name: &Handle<HExtendedString>,
        label: &TdfLabel,
    ) -> Result<(), Failure> {
        if self.is_registered(name) {
            return Ok(());
        }
        let text = rustycad_tkcdf::live(name, "TObj_TNameContainer::RecordName")?;
        let copy = Handle::new(HExtendedString::from(text.string().clone()));
        self.backup()?;
        self.map.write().bind(copy, label.clone());
        Ok(())
    }
    /// `RemoveName(theName)`: backed up when the name is registered.
    pub fn remove_name(&self, name: &Handle<HExtendedString>) -> Result<(), Failure> {
        if !self.is_registered(name) {
            return Ok(());
        }
        self.backup()?;
        self.map.write().unbind(name);
        Ok(())
    }
    /// `IsRegistered(theName)`.
    pub fn is_registered(&self, name: &Handle<HExtendedString>) -> bool {
        self.map.read().is_bound(name)
    }
    /// `Clear`: backed up, then empty.
    pub fn clear(&self) -> Result<(), Failure> {
        self.backup()?;
        self.map.write().clear(true);
        Ok(())
    }
    /// `Set(theElem)`: backed up, then a copy of the map.
    pub fn set_map(&self, map: &TObjNameMap) -> Result<(), Failure> {
        self.backup()?;
        self.map.write().assign(map);
        Ok(())
    }
    /// `Get`: a copy of the map (natively a reference to it).
    pub fn get_map(&self) -> TObjNameMap {
        let mut copy = TObjNameMap::with_hasher(HExtendedStringHasher, 1);
        copy.assign(&self.map.read());
        copy
    }
    /// `Get().Find(theName)`: the label of a registered name; `Standard_NoSuchObject`
    /// when the name is not registered.
    pub fn find_label(&self, name: &Handle<HExtendedString>) -> Result<TdfLabel, Failure> {
        self.map.read().find(name).cloned()
    }
}

impl TdfAttribute for TObjTNameContainer {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        NAME_CONTAINER_ID
    }
    fn restore(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        let other = cast_attribute::<TObjTNameContainer>(with, "TObj_TNameContainer::Restore")?;
        let map = other.get_map();
        self.map.write().assign(&map);
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
        let target = cast_attribute::<TObjTNameContainer>(into, "TObj_TNameContainer::Paste")?;
        target.set_map(&self.get_map())
    }
}
