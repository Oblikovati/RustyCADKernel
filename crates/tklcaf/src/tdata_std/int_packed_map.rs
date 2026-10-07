// Rust port of ApplicationFramework/TKLCAF/TDataStd/TDataStd_IntPackedMap and
// TDataStd_DeltaOnModificationOfIntPackedMap (.hxx and .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDataStd_IntPackedMap`: a packed set of integers, with its modification delta.
use super::support::cast_attribute;
use crate::guid_from_text;
use crate::json::dump_attribute_json_base;
use crate::live;
use crate::tdf::{
    TdfAttribute, TdfAttributeBase, TdfAttributeDelta, TdfAttributeDeltaBase, TdfAttributeState,
    TdfDefaultDeltaOnModification, TdfDeltaOnModification, TdfLabel, TdfRelocationTable,
    dump_attribute,
};
use rustycad_tkcdf::json_dump::{class_begin, field_integer};
use rustycad_tkernel::{
    impl_transient,
    ncollection::{PackedMap, packed_map_algo},
    standard::{Failure, Guid, Handle, SharedState, StandardTransient, dump::DumpStream},
    tcol_std::HPackedMapOfInteger,
};
use std::io;

/// `TDataStd_IntPackedMap::GetID()`.
const INT_PACKED_MAP_ID: Guid = guid_from_text("7031faff-161e-44df-8239-7c264a81f5a1");

/// The native fields `myMap` and `myIsDelta`.
struct IntPackedMapState {
    map: Handle<HPackedMapOfInteger>,
    is_delta: bool,
}

/// `TDataStd_IntPackedMap`; `TDataStdIntPackedMap::set(&label, false)`. The map is never
/// null through this API (the constructor and `Clear` allocate it); the accessors read a
/// null map as empty.
///
/// ```
/// use rustycad_tklcaf::{tdata_std::TDataStdIntPackedMap, tdf::TdfData};
/// let data = TdfData::new();
/// let map = TDataStdIntPackedMap::set(&data.root(), false).unwrap();
/// assert!(map.add(7).unwrap() && !map.add(7).unwrap());
/// assert!(map.contains(7) && map.extent() == 1);
/// ```
pub struct TDataStdIntPackedMap {
    transient: StandardTransient,
    base: TdfAttributeBase,
    state: SharedState<IntPackedMapState>,
}
impl_transient!(TDataStdIntPackedMap, "TDataStd_IntPackedMap", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

impl Default for TDataStdIntPackedMap {
    /// `TDataStd_IntPackedMap()`: an empty map, delta off.
    fn default() -> Self {
        Self {
            transient: StandardTransient::default(),
            base: TdfAttributeBase::default(),
            state: SharedState::new(IntPackedMapState {
                map: Handle::new(HPackedMapOfInteger::default()),
                is_delta: false,
            }),
        }
    }
}

impl TDataStdIntPackedMap {
    /// `GetID`.
    pub fn get_id() -> Guid {
        INT_PACKED_MAP_ID
    }
    /// `TDataStd_IntPackedMap()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Set(label, isDelta)`: the attribute of the label, added (cleared, with the delta
    /// flag) when absent.
    pub fn set(label: &TdfLabel, is_delta: bool) -> Result<Handle<Self>, Failure> {
        let mut found: Handle<Self> = Handle::default();
        if label.find_attribute_as(&INT_PACKED_MAP_ID, &mut found)? {
            return Ok(found);
        }
        let created = Handle::new(Self::new());
        let attribute = live(&created, "TDataStd_IntPackedMap::Set")?;
        attribute.clear()?;
        attribute.set_delta(is_delta);
        label.add_attribute(&created.cast(), false)?;
        Ok(created)
    }
    /// `ChangeMap(theMap)` with a handle: false for a null or the same map, or equal
    /// contents; else backs up and copies the contents.
    pub fn change_h_map(&self, map: &Handle<HPackedMapOfInteger>) -> Result<bool, Failure> {
        let Some(other) = map.get() else {
            return Ok(false);
        };
        if *map == self.get_h_map() {
            return Ok(false);
        }
        let contents = other.map().clone();
        self.change_map(&contents)
    }
    /// `ChangeMap(theMap)`: false for equal contents; else backs up and copies them.
    pub fn change_map(&self, map: &PackedMap<i32>) -> Result<bool, Failure> {
        if packed_map_algo::is_equal(&self.get_map(), map) {
            return Ok(false);
        }
        self.backup()?;
        if let Some(current) = self.get_h_map().get() {
            current.change_map().assign(map);
        }
        Ok(true)
    }
    /// `GetMap`: a copy of the contents.
    pub fn get_map(&self) -> PackedMap<i32> {
        self.get_h_map()
            .get()
            .map(|map| map.map().clone())
            .unwrap_or_default()
    }
    /// `GetHMap`: the map, shared with the attribute.
    pub fn get_h_map(&self) -> Handle<HPackedMapOfInteger> {
        self.state.read().map.clone()
    }
    /// `Clear`: false when empty; else backs up and takes a new empty map.
    pub fn clear(&self) -> Result<bool, Failure> {
        if self.is_empty() {
            return Ok(false);
        }
        self.backup()?;
        self.state.write().map = Handle::new(HPackedMapOfInteger::default());
        Ok(true)
    }
    /// `Add(key)`: false when present; else backs up and adds.
    pub fn add(&self, key: i32) -> Result<bool, Failure> {
        if self.contains(key) {
            return Ok(false);
        }
        self.backup()?;
        Ok(self
            .get_h_map()
            .get()
            .is_some_and(|map| map.change_map().add(key)))
    }
    /// `Remove(key)`: false when absent; else backs up and removes.
    pub fn remove(&self, key: i32) -> Result<bool, Failure> {
        if !self.contains(key) {
            return Ok(false);
        }
        self.backup()?;
        Ok(self
            .get_h_map()
            .get()
            .is_some_and(|map| map.change_map().remove(key)))
    }
    /// `Contains(key)`.
    pub fn contains(&self, key: i32) -> bool {
        self.get_h_map()
            .get()
            .is_some_and(|map| map.map().contains(key))
    }
    /// `Extent`.
    pub fn extent(&self) -> i32 {
        self.get_h_map().get().map_or(0, |map| map.map().extent())
    }
    /// `IsEmpty`.
    pub fn is_empty(&self) -> bool {
        self.extent() == 0
    }
    /// `GetDelta`.
    pub fn get_delta(&self) -> bool {
        self.state.read().is_delta
    }
    /// `SetDelta`: no backup, as natively.
    pub fn set_delta(&self, is_delta: bool) {
        self.state.write().is_delta = is_delta;
    }
}

impl TdfAttribute for TDataStdIntPackedMap {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        INT_PACKED_MAP_ID
    }
    /// `Restore(With)`: a copy of the other map and its delta flag (no map when the other
    /// has none).
    fn restore(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        let other = cast_attribute::<Self>(with, "TDataStd_IntPackedMap::Restore")?;
        let other = live(&other, "TDataStd_IntPackedMap::Restore")?;
        let (map, is_delta) = (other.get_h_map(), other.get_delta());
        let Some(contents) = map.get().map(|map| map.map().clone()) else {
            self.state.write().map = Handle::default();
            return Ok(());
        };
        let mut state = self.state.write();
        state.map = Handle::new(HPackedMapOfInteger::from_map(contents));
        state.is_delta = is_delta;
        Ok(())
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::new()).cast()
    }
    /// `Paste(Into, RT)`: an `Into` of the same class takes the map and the delta flag.
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        _table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        let target = into.cast::<Self>();
        let Some(target) = target.get() else {
            return Ok(());
        };
        target.change_h_map(&self.get_h_map())?;
        target.set_delta(self.get_delta());
        Ok(())
    }
    /// `Dump`: the attribute dump, the size and the delta flag.
    fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        dump_attribute(self, stream)?;
        let flag = if self.get_delta() { "ON" } else { "OFF" };
        writeln!(
            stream,
            "IntPackedMap size = {} Delta is {flag}",
            self.extent()
        )
    }
    /// `DumpJson`: the keys in map order, then the delta flag.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "TDataStd_IntPackedMap")?;
        dump_attribute_json_base(self, stream, depth)?;
        for key in self.get_map().iter() {
            field_integer(stream, "aKey", key)?;
        }
        field_integer(stream, "myIsDelta", i32::from(self.get_delta()))
    }
    /// `DeltaOnModification(OldAttribute)`: the packed-map delta when the delta flag is
    /// on, else the default delta.
    fn delta_on_modification(
        &self,
        old_attribute: &Handle<dyn TdfAttribute>,
    ) -> Result<Handle<dyn TdfDeltaOnModification>, Failure> {
        if !self.get_delta() {
            return Ok(TdfDefaultDeltaOnModification::new(old_attribute).cast());
        }
        Ok(TDataStdDeltaOnModificationOfIntPackedMap::new(&old_attribute.cast())?.cast())
    }
}

/// `TDataStd_DeltaOnModificationOfIntPackedMap`: the keys a modification removed
/// (`myAddition`, added back on undo) and added (`myDeletion`, removed on undo).
pub struct TDataStdDeltaOnModificationOfIntPackedMap {
    transient: StandardTransient,
    base: TdfAttributeDeltaBase,
    addition: Option<PackedMap<i32>>,
    deletion: Option<PackedMap<i32>>,
}
impl_transient!(TDataStdDeltaOnModificationOfIntPackedMap, "TDataStd_DeltaOnModificationOfIntPackedMap",
    dyn TdfDeltaOnModification, transient, interfaces: [dyn TdfAttributeDelta, dyn TdfDeltaOnModification]);

impl TDataStdDeltaOnModificationOfIntPackedMap {
    /// The constructor `(OldAtt)`: compares the backup with the current attribute and
    /// records the key differences (the backup keeps its map, as natively).
    pub fn new(old_attribute: &Handle<TDataStdIntPackedMap>) -> Result<Handle<Self>, Failure> {
        let old = live(old_attribute, "TDataStd_DeltaOnModificationOfIntPackedMap")?;
        let mut current: Handle<TDataStdIntPackedMap> = Handle::default();
        let found = old.label().find_attribute_as(&old.id(), &mut current)?;
        let (addition, deletion) = if found {
            key_differences(&old.get_h_map(), &current.get_h_map())
        } else {
            (None, None)
        };
        Ok(TdfAttributeDeltaBase::share(Self {
            transient: StandardTransient::default(),
            base: TdfAttributeDeltaBase::new(&old_attribute.cast()),
            addition,
            deletion,
        }))
    }
}

/// The native subset cases: the keys only in the old map (`myAddition`) and only in the
/// current one (`myDeletion`); a disjoint pair records both maps whole.
fn key_differences(
    old: &Handle<HPackedMapOfInteger>,
    current: &Handle<HPackedMapOfInteger>,
) -> (Option<PackedMap<i32>>, Option<PackedMap<i32>>) {
    let (Some(map1), Some(map2)) = (old.get(), current.get()) else {
        return (None, None);
    };
    if old == current {
        return (None, None);
    }
    let (map1, map2) = (map1.map().clone(), map2.map().clone());
    let minus = |left: &PackedMap<i32>, right: &PackedMap<i32>| {
        let mut result = PackedMap::default();
        packed_map_algo::subtraction(&mut result, left, right);
        result
    };
    if packed_map_algo::is_subset(&map1, &map2) {
        return (None, Some(minus(&map2, &map1)));
    }
    if packed_map_algo::is_subset(&map2, &map1) {
        return (Some(minus(&map1, &map2)), None);
    }
    if packed_map_algo::has_intersection(&map1, &map2) {
        return (Some(minus(&map1, &map2)), Some(minus(&map2, &map1)));
    }
    (Some(map1), Some(map2))
}

impl TdfAttributeDelta for TDataStdDeltaOnModificationOfIntPackedMap {
    fn attribute_delta_base(&self) -> &TdfAttributeDeltaBase {
        &self.base
    }
    /// `Apply`: the backup is added back when its label lost the attribute; otherwise the
    /// current attribute backs up, loses `myDeletion` and gains `myAddition`.
    fn apply(&self) -> Result<(), Failure> {
        let backup = self.attribute().cast::<TDataStdIntPackedMap>();
        if backup.is_null() {
            return Ok(());
        }
        let mut current: Handle<TDataStdIntPackedMap> = Handle::default();
        if !self
            .label()
            .find_attribute_as(&INT_PACKED_MAP_ID, &mut current)?
        {
            return self.label().add_attribute(&backup.cast(), false);
        }
        let current = live(
            &current,
            "TDataStd_DeltaOnModificationOfIntPackedMap::Apply",
        )?;
        current.backup()?;
        let map = current.get_h_map();
        let Some(map) = map.get() else {
            return Ok(());
        };
        let mut map = map.change_map();
        if let Some(deletion) = self.deletion.as_ref().filter(|keys| !keys.is_empty()) {
            packed_map_algo::subtract(&mut map, deletion);
        }
        if let Some(addition) = self.addition.as_ref().filter(|keys| !keys.is_empty()) {
            packed_map_algo::unite(&mut map, addition);
        }
        Ok(())
    }
}
impl TdfDeltaOnModification for TDataStdDeltaOnModificationOfIntPackedMap {}
