// Rust translation of the `NCollection_IndexedMap<occ::handle<T>>` and
// `NCollection_IndexedDataMap<occ::handle<T>, bool>` members of the BinTools sets
// (BinTools_CurveSet.cxx, BinTools_Curve2dSet.cxx, BinTools_SurfaceSet.cxx,
// BinTools_ShapeSet.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The indexed tables of handles behind the BinTools sets: the key is the object
//! identity (`NCollection_DefaultHasher` of a handle hashes the address), a null
//! handle is a valid key, and indices start at 1 in insertion order.
use super::errors::index_out_of_range;
use rustycad_tkernel::ncollection::{HashPolicy, IndexedDataMap, IndexedMap};
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use std::fmt;

/// A handle compared by object identity.
pub(crate) struct HandleKey<T: ?Sized + RuntimeTyped>(pub(crate) Handle<T>);

impl<T: ?Sized + RuntimeTyped> fmt::Debug for HandleKey<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "HandleKey({:#x})", self.0.hash_code())
    }
}

/// The address hash and pointer equality of `occ::handle`.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct IdentityHasher;

impl<T: ?Sized + RuntimeTyped> HashPolicy<HandleKey<T>> for IdentityHasher {
    fn hash(&self, key: &HandleKey<T>) -> usize {
        key.0.hash_code()
    }
    fn equal(&self, first: &HandleKey<T>, second: &HandleKey<T>) -> bool {
        first.0 == second.0
    }
}

/// `NCollection_IndexedMap<occ::handle<T>>`.
pub(crate) struct HandleTable<T: ?Sized + RuntimeTyped> {
    map: IndexedMap<HandleKey<T>, IdentityHasher>,
    /// The class that owns the table, for the OutOfRange message.
    owner: &'static str,
}

impl<T: ?Sized + RuntimeTyped> HandleTable<T> {
    /// An empty table of the named owner class.
    pub(crate) fn new(owner: &'static str) -> Self {
        Self {
            map: IndexedMap::default(),
            owner,
        }
    }

    /// `myMap.Clear()`.
    pub(crate) fn clear(&mut self) {
        self.map.clear(true);
    }

    /// `myMap.Add(handle)`: the index of the object, a null handle included.
    pub(crate) fn add(&mut self, handle: &Handle<T>) -> i32 {
        self.map.add(HandleKey(handle.clone())) as i32
    }

    /// `myMap.FindIndex(handle)`: 0 when absent.
    pub(crate) fn index(&self, handle: &Handle<T>) -> i32 {
        self.map.find_index(&HandleKey(handle.clone())) as i32
    }

    /// `myMap(I)` / `myMap.FindKey(I)`: Standard_OutOfRange outside `1..=Extent()`.
    pub(crate) fn get(&self, index: i32) -> Result<Handle<T>, Failure> {
        usize::try_from(index)
            .ok()
            .and_then(|position| self.map.find_key(position).ok())
            .map(|key| key.0.clone())
            .ok_or_else(|| index_out_of_range(self.owner, index, self.map.extent()))
    }

    /// `myMap.Extent()`.
    pub(crate) fn extent(&self) -> i32 {
        self.map.extent() as i32
    }
}

/// `NCollection_IndexedDataMap<occ::handle<Poly_Triangulation>, bool>`: each handle
/// with the flag of the first `Add` (a later `Add` of the same handle keeps it).
pub(crate) struct FlaggedHandleTable<T: ?Sized + RuntimeTyped> {
    map: IndexedDataMap<HandleKey<T>, bool, IdentityHasher>,
    owner: &'static str,
}

impl<T: ?Sized + RuntimeTyped> FlaggedHandleTable<T> {
    /// An empty table of the named owner class.
    pub(crate) fn new(owner: &'static str) -> Self {
        Self {
            map: IndexedDataMap::default(),
            owner,
        }
    }

    /// `myMap.Clear()`.
    pub(crate) fn clear(&mut self) {
        self.map.clear(true);
    }

    /// `myMap.Add(handle, flag)`: the index; an existing handle keeps its flag.
    pub(crate) fn add(&mut self, handle: &Handle<T>, flag: bool) -> i32 {
        self.map.add(HandleKey(handle.clone()), flag) as i32
    }

    /// `myMap.FindIndex(handle)`: 0 when absent.
    pub(crate) fn index(&self, handle: &Handle<T>) -> i32 {
        self.map.find_index(&HandleKey(handle.clone())) as i32
    }

    /// `myMap.FindKey(I)` and `myMap.FindFromIndex(I)`: Standard_OutOfRange outside
    /// `1..=Extent()`.
    pub(crate) fn get(&self, index: i32) -> Result<(Handle<T>, bool), Failure> {
        let position = usize::try_from(index).unwrap_or(0);
        let key = self.map.find_key(position);
        let flag = self.map.find_from_index(position);
        match (key, flag) {
            (Ok(key), Ok(flag)) => Ok((key.0.clone(), *flag)),
            _ => Err(index_out_of_range(self.owner, index, self.map.extent())),
        }
    }

    /// `myMap.Extent()`.
    pub(crate) fn extent(&self) -> i32 {
        self.map.extent() as i32
    }
}
