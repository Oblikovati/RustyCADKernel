// Rust port of ApplicationFramework/TKLCAF/TDataStd/TDataStd_NamedData.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDataStd_NamedData`: named integers, reals, strings, bytes and arrays on one label.
use super::h_data_maps::{
    TDataStdHDataMapOfStringByte, TDataStdHDataMapOfStringHArray1OfInteger,
    TDataStdHDataMapOfStringHArray1OfReal, TDataStdHDataMapOfStringInteger,
    TDataStdHDataMapOfStringReal, TDataStdHDataMapOfStringString,
};
use super::named_data_category::{
    NamedDataState, bind, change_all, deep_copy, ensure, extent_if_any, get, has, has_any,
    paste_category, restore_category, set, size_if_set,
};
use crate::guid_from_text;
use crate::json::dump_attribute_json_base;
use crate::tdf::{TdfAttribute, TdfAttributeBase, TdfAttributeState, TdfLabel, TdfRelocationTable};
use rustycad_tkcdf::json_dump::{class_begin, field_integer};
use rustycad_tkernel::{
    impl_transient,
    ncollection::{DataMap, HArray1},
    standard::{Failure, Guid, Handle, SharedState, StandardTransient, dump::DumpStream},
    tcollection::ExtendedString,
};
use std::io;

/// `TDataStd_NamedData::GetID()`.
const NAMED_DATA_ID: Guid = guid_from_text("F170FD21-CBAE-4e7d-A4B4-0560A4DA2D16");

/// `TDataStd_NamedData`; `TDataStdNamedData::set(&label)`. The `Get*Container` members
/// return the category's shared map (natively a reference to it): a caller must drop the
/// guards of `map()` before it modifies the attribute.
///
/// ```
/// use rustycad_tklcaf::{tdata_std::TDataStdNamedData, tdf::TdfData};
/// use rustycad_tkernel::tcollection::ExtendedString;
/// let data = TdfData::new();
/// let named = TDataStdNamedData::set(&data.root()).unwrap();
/// named.set_integer(&ExtendedString::new("n"), 3).unwrap();
/// assert_eq!(named.get_integer(&ExtendedString::new("n")).unwrap(), 3);
/// assert!(named.get_real(&ExtendedString::new("x")).is_err());
/// ```
pub struct TDataStdNamedData {
    transient: StandardTransient,
    base: TdfAttributeBase,
    pub(crate) state: SharedState<NamedDataState>,
}
impl_transient!(TDataStdNamedData, "TDataStd_NamedData", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

impl Default for TDataStdNamedData {
    /// `TDataStd_NamedData()`: every category without map.
    fn default() -> Self {
        Self {
            transient: StandardTransient::default(),
            base: TdfAttributeBase::default(),
            state: SharedState::new(NamedDataState::default()),
        }
    }
}

/// Declares the members of one category.
macro_rules! category_members {
    (
        $map:ty, $value:ty, has_any: $has_any:ident, has: $has:ident, get: $get:ident,
        container: $container:ident, change: $change:ident
    ) => {
        /// `Has<Category>s`: the category has bindings.
        pub fn $has_any(&self) -> bool {
            has_any::<$map>(self)
        }
        /// `Has<Category>(name)`: the name is bound in the category.
        pub fn $has(&self, name: &ExtendedString) -> bool {
            has::<$map>(self, name)
        }
        /// `Get<Category>(name)`: the bound value; `Standard_NoSuchObject` for an unknown
        /// name (the category's map is created empty first, as natively).
        pub fn $get(&self, name: &ExtendedString) -> Result<$value, Failure> {
            get::<$map>(self, name)
        }
        /// `Get<Category>sContainer`: the category's map, created empty when missing.
        pub fn $container(&self) -> Handle<$map> {
            ensure::<$map>(self)
        }
        /// `Change<Category>s(map)`: nothing for the category's own map; else backs up and
        /// copies the bindings.
        pub fn $change(&self, map: &DataMap<ExtendedString, $value>) -> Result<(), Failure> {
            change_all::<$map>(self, map)
        }
    };
}

/// Declares the setters of one value category.
macro_rules! value_setters {
    ($map:ty, $value:ty, $set:ident, $set_raw:ident, backup_new: $backup_new:expr) => {
        /// `Set<Category>(name, value)`: nothing when unchanged; else backs up and binds.
        pub fn $set(&self, name: &ExtendedString, value: $value) -> Result<(), Failure> {
            set::<$map>(self, name, value, $backup_new)
        }
        /// `set<Category>(name, value)`: binds without backup.
        pub fn $set_raw(&self, name: &ExtendedString, value: $value) {
            bind::<$map>(self, name, value);
        }
    };
}

/// Declares the setters of one array category.
macro_rules! array_setters {
    ($map:ty, $item:ty, $set:ident, $set_raw:ident) => {
        /// `SetArrayOf<Category>(name, array)`: backs up, then binds a deep copy.
        pub fn $set(
            &self,
            name: &ExtendedString,
            array: &Handle<HArray1<$item>>,
        ) -> Result<(), Failure> {
            self.backup()?;
            self.$set_raw(name, array);
            Ok(())
        }
        /// `setArrayOf<Category>(name, array)`: binds a deep copy (null stays null) without
        /// backup.
        pub fn $set_raw(&self, name: &ExtendedString, array: &Handle<HArray1<$item>>) {
            bind::<$map>(self, name, deep_copy(array));
        }
    };
}

impl TDataStdNamedData {
    /// `GetID`.
    pub fn get_id() -> Guid {
        NAMED_DATA_ID
    }
    /// `TDataStd_NamedData()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Set(label)`: the attribute of the label, added when absent.
    pub fn set(label: &TdfLabel) -> Result<Handle<Self>, Failure> {
        let mut found: Handle<Self> = Handle::default();
        if label.find_attribute_as(&NAMED_DATA_ID, &mut found)? {
            return Ok(found);
        }
        let created = Handle::new(Self::new());
        label.add_attribute(&created.cast(), false)?;
        Ok(created)
    }
    category_members!(TDataStdHDataMapOfStringInteger, i32, has_any: has_integers, has: has_integer,
        get: get_integer, container: get_integers_container, change: change_integers);
    value_setters!(TDataStdHDataMapOfStringInteger, i32, set_integer, set_integer_without_backup, backup_new: true);
    category_members!(TDataStdHDataMapOfStringReal, f64, has_any: has_reals, has: has_real,
        get: get_real, container: get_reals_container, change: change_reals);
    value_setters!(TDataStdHDataMapOfStringReal, f64, set_real, set_real_without_backup, backup_new: false);
    category_members!(TDataStdHDataMapOfStringString, ExtendedString, has_any: has_strings,
        has: has_string, get: get_string, container: get_strings_container, change: change_strings);
    value_setters!(TDataStdHDataMapOfStringString, ExtendedString, set_string, set_string_without_backup, backup_new: true);
    category_members!(TDataStdHDataMapOfStringByte, u8, has_any: has_bytes, has: has_byte,
        get: get_byte, container: get_bytes_container, change: change_bytes);
    value_setters!(TDataStdHDataMapOfStringByte, u8, set_byte, set_byte_without_backup, backup_new: true);
    category_members!(TDataStdHDataMapOfStringHArray1OfInteger, Handle<HArray1<i32>>,
        has_any: has_arrays_of_integers, has: has_array_of_integers, get: get_array_of_integers,
        container: get_arrays_of_integers_container, change: change_arrays_of_integers);
    array_setters!(
        TDataStdHDataMapOfStringHArray1OfInteger,
        i32,
        set_array_of_integers,
        set_array_of_integers_without_backup
    );
    category_members!(TDataStdHDataMapOfStringHArray1OfReal, Handle<HArray1<f64>>,
        has_any: has_arrays_of_reals, has: has_array_of_reals, get: get_array_of_reals,
        container: get_arrays_of_reals_container, change: change_arrays_of_reals);
    array_setters!(
        TDataStdHDataMapOfStringHArray1OfReal,
        f64,
        set_array_of_reals,
        set_array_of_reals_without_backup
    );

    /// `Clear`: backs up, then drops every category's map.
    pub fn clear(&self) -> Result<(), Failure> {
        self.backup()?;
        self.clear_without_backup();
        Ok(())
    }
    /// `clear()`: drops every category's map without backup.
    pub fn clear_without_backup(&self) {
        *self.state.write() = NamedDataState::default();
    }
    /// `HasDeferredData` (virtual): false; the deferred storage belongs to the native
    /// binary format, which is not ported (user decision 2026-09).
    pub fn has_deferred_data(&self) -> bool {
        false
    }
    /// `LoadDeferredData(theToKeepDeferred)` (virtual): false, no deferred storage.
    pub fn load_deferred_data(&self, _to_keep_deferred: bool) -> bool {
        false
    }
    /// `UnloadDeferredData` (virtual): false, no deferred storage.
    pub fn unload_deferred_data(&self) -> bool {
        false
    }
}

impl TdfAttribute for TDataStdNamedData {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        NAMED_DATA_ID
    }
    /// `Restore(With)`: every category of `With` with bindings is copied (the arrays
    /// deeply, bound over the existing ones); the categories `With` lacks are kept.
    fn restore(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        let other = with.cast::<Self>();
        let Some(other) = other.get() else {
            return Ok(());
        };
        restore_category::<TDataStdHDataMapOfStringInteger>(self, other, true)?;
        restore_category::<TDataStdHDataMapOfStringReal>(self, other, true)?;
        restore_category::<TDataStdHDataMapOfStringString>(self, other, true)?;
        restore_category::<TDataStdHDataMapOfStringByte>(self, other, true)?;
        restore_category::<TDataStdHDataMapOfStringHArray1OfInteger>(self, other, false)?;
        restore_category::<TDataStdHDataMapOfStringHArray1OfReal>(self, other, false)
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::new()).cast()
    }
    /// `Paste(Into, RT)`: `Into` takes every category this attribute has bindings in.
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        _table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        let target = into.cast::<Self>();
        let Some(target) = target.get() else {
            return Ok(());
        };
        paste_category::<TDataStdHDataMapOfStringInteger>(self, target, true)?;
        paste_category::<TDataStdHDataMapOfStringReal>(self, target, true)?;
        paste_category::<TDataStdHDataMapOfStringString>(self, target, true)?;
        paste_category::<TDataStdHDataMapOfStringByte>(self, target, true)?;
        paste_category::<TDataStdHDataMapOfStringHArray1OfInteger>(self, target, false)?;
        paste_category::<TDataStdHDataMapOfStringHArray1OfReal>(self, target, false)
    }
    /// `Dump`: `"NamedData: "` and the binding count of each category.
    fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        write!(stream, "NamedData: ")?;
        write!(
            stream,
            "\tIntegers = {}",
            extent_if_any::<TDataStdHDataMapOfStringInteger>(self)
        )?;
        write!(
            stream,
            "\tReals = {}",
            extent_if_any::<TDataStdHDataMapOfStringReal>(self)
        )?;
        write!(
            stream,
            "\tStrings = {}",
            extent_if_any::<TDataStdHDataMapOfStringString>(self)
        )?;
        write!(
            stream,
            "\tBytes = {}",
            extent_if_any::<TDataStdHDataMapOfStringByte>(self)
        )?;
        let arrays_of_integers = extent_if_any::<TDataStdHDataMapOfStringHArray1OfInteger>(self);
        write!(stream, "\tArraysOfIntegers = {arrays_of_integers}")?;
        write!(
            stream,
            "\tArraysOfReals = {}",
            extent_if_any::<TDataStdHDataMapOfStringHArray1OfReal>(self)
        )
    }
    /// `DumpJson`: the size of every category's map that exists.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "TDataStd_NamedData")?;
        dump_attribute_json_base(self, stream, depth)?;
        let sizes = [
            (
                "myIntegers->Map().Size()",
                size_if_set::<TDataStdHDataMapOfStringInteger>(self),
            ),
            (
                "myReals->Map().Size()",
                size_if_set::<TDataStdHDataMapOfStringReal>(self),
            ),
            (
                "myStrings->Map().Size()",
                size_if_set::<TDataStdHDataMapOfStringString>(self),
            ),
            (
                "myBytes->Map().Size()",
                size_if_set::<TDataStdHDataMapOfStringByte>(self),
            ),
            (
                "myArraysOfIntegers->Map().Size()",
                size_if_set::<TDataStdHDataMapOfStringHArray1OfInteger>(self),
            ),
            (
                "myArraysOfReals->Map().Size()",
                size_if_set::<TDataStdHDataMapOfStringHArray1OfReal>(self),
            ),
        ];
        for (field, size) in sizes {
            if let Some(size) = size {
                field_integer(stream, field, size as i32)?;
            }
        }
        Ok(())
    }
}
