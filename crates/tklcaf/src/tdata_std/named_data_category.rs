// Rust port of the per-category members of ApplicationFramework/TKLCAF/TDataStd/
// TDataStd_NamedData.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The six categories of `TDataStd_NamedData` (integers, reals, strings, bytes, arrays of
//! integers, arrays of reals) repeat one code body natively; here it is written once over
//! [`NamedCategory`] (ocaf-design.md section 7).
use super::h_data_maps::{
    TDataStdHDataMapOfStringByte, TDataStdHDataMapOfStringHArray1OfInteger,
    TDataStdHDataMapOfStringHArray1OfReal, TDataStdHDataMapOfStringInteger,
    TDataStdHDataMapOfStringReal, TDataStdHDataMapOfStringString, TDataStdNamedMap,
};
use super::named_data::TDataStdNamedData;
use crate::live;
use crate::tdf::TdfAttributeState;
use rustycad_tkernel::{
    ncollection::{DataMap, HArray1},
    standard::{Failure, Handle},
    tcollection::ExtendedString,
};

/// The fields `myIntegers` .. `myArraysOfReals` (each null until first used).
#[derive(Default)]
pub(crate) struct NamedDataState {
    pub(crate) integers: Handle<TDataStdHDataMapOfStringInteger>,
    pub(crate) reals: Handle<TDataStdHDataMapOfStringReal>,
    pub(crate) strings: Handle<TDataStdHDataMapOfStringString>,
    pub(crate) bytes: Handle<TDataStdHDataMapOfStringByte>,
    pub(crate) arrays_of_integers: Handle<TDataStdHDataMapOfStringHArray1OfInteger>,
    pub(crate) arrays_of_reals: Handle<TDataStdHDataMapOfStringHArray1OfReal>,
}

/// One category: its field, and the copy of a value in `Restore` and `Paste` (deep for
/// the arrays, plain otherwise).
pub(crate) trait NamedCategory: TDataStdNamedMap {
    /// The field of this category.
    fn slot(state: &mut NamedDataState) -> &mut Handle<Self>;
    /// The value `Restore` and `Paste` bind.
    fn copied(value: &Self::Value) -> Result<Self::Value, Failure> {
        Ok(value.clone())
    }
}

/// Implements [`NamedCategory`] for a map class whose values copy plainly.
macro_rules! plain_category {
    ($map:ty, $field:ident) => {
        impl NamedCategory for $map {
            fn slot(state: &mut NamedDataState) -> &mut Handle<Self> {
                &mut state.$field
            }
        }
    };
}
plain_category!(TDataStdHDataMapOfStringInteger, integers);
plain_category!(TDataStdHDataMapOfStringReal, reals);
plain_category!(TDataStdHDataMapOfStringString, strings);
plain_category!(TDataStdHDataMapOfStringByte, bytes);

impl NamedCategory for TDataStdHDataMapOfStringHArray1OfInteger {
    fn slot(state: &mut NamedDataState) -> &mut Handle<Self> {
        &mut state.arrays_of_integers
    }
    fn copied(value: &Handle<HArray1<i32>>) -> Result<Handle<HArray1<i32>>, Failure> {
        Ok(deep_copy(value))
    }
}

impl NamedCategory for TDataStdHDataMapOfStringHArray1OfReal {
    fn slot(state: &mut NamedDataState) -> &mut Handle<Self> {
        &mut state.arrays_of_reals
    }
    fn copied(value: &Handle<HArray1<f64>>) -> Result<Handle<HArray1<f64>>, Failure> {
        Ok(deep_copy(value))
    }
}

/// The "deep copy of the array" of `setArrayOf*`, `Restore` and `Paste`: a new array with
/// the same bounds and items; a null array stays null.
pub(crate) fn deep_copy<T: Clone + Send + Sync + 'static>(
    array: &Handle<HArray1<T>>,
) -> Handle<HArray1<T>> {
    array.get().map_or_else(Handle::default, |items| {
        Handle::new(HArray1::from_array(&items.array1()))
    })
}

/// The field of a category (possibly null).
pub(crate) fn current<C: NamedCategory>(attribute: &TDataStdNamedData) -> Handle<C> {
    C::slot(&mut attribute.state.write()).clone()
}

/// `HasIntegers()` and the like: the map exists and is not empty.
pub(crate) fn has_any<C: NamedCategory>(attribute: &TDataStdNamedData) -> bool {
    current::<C>(attribute)
        .get()
        .is_some_and(|map| !map.map().is_empty())
}

/// `HasInteger(name)` and the like.
pub(crate) fn has<C: NamedCategory>(attribute: &TDataStdNamedData, name: &ExtendedString) -> bool {
    has_any::<C>(attribute)
        && current::<C>(attribute)
            .get()
            .is_some_and(|map| map.map().is_bound(name))
}

/// The native `if (!HasIntegers()) myIntegers = new ...(aMap);`: a missing or empty map is
/// replaced by a new empty one; the map of the category.
pub(crate) fn ensure<C: NamedCategory>(attribute: &TDataStdNamedData) -> Handle<C> {
    if !has_any::<C>(attribute) {
        *C::slot(&mut attribute.state.write()) = Handle::new(C::from_map(&DataMap::default()));
    }
    current::<C>(attribute)
}

/// `GetInteger(name)` and the like; `Standard_NoSuchObject` for an unknown name.
pub(crate) fn get<C: NamedCategory>(
    attribute: &TDataStdNamedData,
    name: &ExtendedString,
) -> Result<C::Value, Failure> {
    let map = ensure::<C>(attribute);
    let bindings = live(&map, "TDataStd_NamedData")?.map();
    bindings.find(name).cloned()
}

/// `setInteger(name, value)` and the like: binds without backup.
pub(crate) fn bind<C: NamedCategory>(
    attribute: &TDataStdNamedData,
    name: &ExtendedString,
    value: C::Value,
) {
    if let Some(map) = ensure::<C>(attribute).get() {
        map.change_map().bind(name.clone(), value);
    }
}

/// `SetInteger(name, value)` and the like: nothing when the name holds the value; else
/// backs up and binds. `backup_new` is false for `SetReal`, which natively binds a new name
/// without backup.
pub(crate) fn set<C: NamedCategory>(
    attribute: &TDataStdNamedData,
    name: &ExtendedString,
    value: C::Value,
    backup_new: bool,
) -> Result<(), Failure> {
    let map = ensure::<C>(attribute);
    let existing = live(&map, "TDataStd_NamedData")?
        .map()
        .find(name)
        .ok()
        .cloned();
    match existing {
        Some(old) if old == value => return Ok(()),
        Some(_) => attribute.backup()?,
        None if backup_new => attribute.backup()?,
        None => {}
    }
    live(&map, "TDataStd_NamedData")?
        .change_map()
        .bind(name.clone(), value);
    Ok(())
}

/// `ChangeIntegers(theIntegers)` and the like: nothing when `map` is the category's own
/// map (the native address test); else backs up and copies the bindings.
pub(crate) fn change_all<C: NamedCategory>(
    attribute: &TDataStdNamedData,
    map: &DataMap<ExtendedString, C::Value>,
) -> Result<(), Failure> {
    let own = ensure::<C>(attribute);
    let own = live(&own, "TDataStd_NamedData")?;
    if std::ptr::eq(&*own.map(), map) {
        return Ok(());
    }
    attribute.backup()?;
    let copy = clone_map::<C>(map);
    own.change_map().assign(&copy);
    Ok(())
}

/// A copy of a map's bindings (so that no lock of the source is held while assigning).
fn clone_map<C: NamedCategory>(
    map: &DataMap<ExtendedString, C::Value>,
) -> DataMap<ExtendedString, C::Value> {
    let mut copy = DataMap::default();
    copy.assign(map);
    copy
}

/// The category part of `Restore`: when `source`'s map (created empty if missing, as the
/// native `GetIntegersContainer` does) is not empty, `target` takes its bindings; the array
/// categories bind deep copies over the existing bindings instead of replacing them.
pub(crate) fn restore_category<C: NamedCategory>(
    target: &TDataStdNamedData,
    source: &TDataStdNamedData,
    replace: bool,
) -> Result<(), Failure> {
    let source_map = ensure::<C>(source);
    let bindings = clone_map::<C>(&live(&source_map, "TDataStd_NamedData::Restore")?.map());
    if bindings.is_empty() {
        return Ok(());
    }
    copy_bindings::<C>(&ensure::<C>(target), &bindings, replace)
}

/// The category part of `Paste`: when `source` has bindings, `target` takes them as in
/// [`restore_category`] (without creating `source`'s map).
pub(crate) fn paste_category<C: NamedCategory>(
    source: &TDataStdNamedData,
    target: &TDataStdNamedData,
    replace: bool,
) -> Result<(), Failure> {
    if !has_any::<C>(source) {
        return Ok(());
    }
    let source_map = current::<C>(source);
    let bindings = clone_map::<C>(&live(&source_map, "TDataStd_NamedData::Paste")?.map());
    copy_bindings::<C>(&ensure::<C>(target), &bindings, replace)
}

/// `Assign` (when `replace`) or one `Bind` of a copied value per binding.
fn copy_bindings<C: NamedCategory>(
    target: &Handle<C>,
    bindings: &DataMap<ExtendedString, C::Value>,
    replace: bool,
) -> Result<(), Failure> {
    let target = live(target, "TDataStd_NamedData")?;
    if replace {
        target.change_map().assign(bindings);
        return Ok(());
    }
    for (name, value) in bindings.items() {
        let copy = C::copied(value)?;
        target.change_map().bind(name.clone(), copy);
    }
    Ok(())
}

/// `Extent` of a category's map when it has bindings, else 0 (for `Dump`).
pub(crate) fn extent_if_any<C: NamedCategory>(attribute: &TDataStdNamedData) -> usize {
    if !has_any::<C>(attribute) {
        return 0;
    }
    current::<C>(attribute)
        .get()
        .map_or(0, |map| map.map().extent())
}

/// `Map().Size()` of a category's map when the field is not null (for `DumpJson`).
pub(crate) fn size_if_set<C: NamedCategory>(attribute: &TDataStdNamedData) -> Option<usize> {
    current::<C>(attribute).get().map(|map| map.map().extent())
}
