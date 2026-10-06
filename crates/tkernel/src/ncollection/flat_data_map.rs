//! Contiguous key/value slots backed by the shared OCCT Robin Hood engine.
use super::flat_table::{FlatItems, FlatSlot, FlatTable, Inserted};
use super::{DefaultHashPolicy, HashPolicy};
use crate::standard::Failure;
use std::fmt::Debug;
mod borrowed_binding;
/// A contiguous Robin Hood key/value map; `FlatDataMap::<i32,i32>::default().bind(1,42)`.
pub struct FlatDataMap<K, V, H = DefaultHashPolicy> {
    table: FlatTable<K, V, H>,
}
/// Borrowed pairs in slot order; `map.items()`.
pub struct FlatDataItems<'a, K, V> {
    inner: FlatItems<'a, K, V>,
}
/// Borrowed values in slot order; `map.iter()`.
pub struct FlatDataValues<'a, K, V> {
    inner: FlatItems<'a, K, V>,
}
/// Exclusive values in slot order; `map.iter_mut()`.
pub struct FlatDataValuesMut<'a, K, V> {
    slots: std::slice::IterMut<'a, Option<FlatSlot<K, V>>>,
}
impl<K: Debug, V, H: HashPolicy<K>> FlatDataMap<K, V, H> {
    /// Injects hash/equality policy and an element-capacity hint; `FlatDataMap::with_hasher(policy,10)`.
    pub fn with_hasher(hasher: H, hint: usize) -> Self {
        Self {
            table: FlatTable::new(hasher, hint),
        }
    }
    /// Returns the number of bindings; `map.len()==3`.
    pub fn len(&self) -> usize {
        self.table.count
    }
    /// Returns OCCT's binding extent; `map.extent()==3`.
    pub fn extent(&self) -> usize {
        self.len()
    }
    /// Checks for no bindings; `map.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Returns contiguous slot count; `map.capacity()`.
    pub fn capacity(&self) -> usize {
        self.table.slots.len()
    }
    /// Borrows the injected hash policy; `map.hasher().salt`.
    pub fn hasher(&self) -> &H {
        &self.table.hasher
    }
    /// Reserves space for an element count at the native load limit; `map.reserve(100)`.
    pub fn reserve(&mut self, count: usize) {
        self.table.reserve(count);
    }
    /// Tests key membership; `map.is_bound(&42)`.
    pub fn is_bound(&self, key: &K) -> bool {
        self.table.lookup(key).is_some()
    }
    /// Optionally borrows a value; `map.seek(&42)`.
    pub fn seek(&self, key: &K) -> Option<&V> {
        Some(&self.table.slots[self.table.lookup(key)?].as_ref()?.value)
    }
    /// Optionally borrows a value exclusively; `map.change_seek(&42)`.
    pub fn change_seek(&mut self, key: &K) -> Option<&mut V> {
        let index = self.table.lookup(key)?;
        Some(&mut self.table.slots[index].as_mut()?.value)
    }
    /// Borrows the stored key and its value, OCCT's optional pair; `map.contained(&42)`.
    pub fn contained(&self, key: &K) -> Option<(&K, &V)> {
        let slot = self.table.slots[self.table.lookup(key)?].as_ref()?;
        Some((&slot.key, &slot.value))
    }
    /// Borrows the stored key and its value exclusively; `map.contained_mut(&42)`.
    pub fn contained_mut(&mut self, key: &K) -> Option<(&K, &mut V)> {
        let index = self.table.lookup(key)?;
        let slot = self.table.slots[index].as_mut()?;
        Some((&slot.key, &mut slot.value))
    }
    /// Finds a value or reports NoSuchObject; `map.find(&42)?`.
    pub fn find(&self, key: &K) -> Result<&V, Failure> {
        self.seek(key).ok_or_else(|| missing_flat_key(key))
    }
    /// Finds an exclusive value or reports NoSuchObject; `*map.change_find(&42)?=7`.
    pub fn change_find(&mut self, key: &K) -> Result<&mut V, Failure> {
        self.change_seek(key).ok_or_else(|| missing_flat_key(key))
    }
    /// Removes a pair and compacts its probe cluster; `map.unbind(&42)`.
    pub fn unbind(&mut self, key: &K) -> bool {
        self.table.remove(key)
    }
    /// Drops bindings and optionally releases slot storage; `map.clear(false)`.
    pub fn clear(&mut self, release: bool) {
        self.table.clear(release);
    }
    /// Swaps hash policies and storage; `map.exchange(&mut other)`.
    pub fn exchange(&mut self, other: &mut Self) {
        std::mem::swap(self, other);
    }
}
impl<K: Debug, V, H: HashPolicy<K> + Default> FlatDataMap<K, V, H> {
    /// Selects capacity with a default policy; `FlatDataMap::<i32,i32>::new(10)`.
    pub fn new(hint: usize) -> Self {
        Self::with_hasher(H::default(), hint)
    }
    /// Moves storage and leaves an empty source; `FlatDataMap::move_from(&mut source)`.
    pub fn move_from(source: &mut Self) -> Self {
        Self::move_from_with(source, std::mem::take)
    }
    /// Replaces storage with a moved source; `map.move_assign(&mut source)`.
    pub fn move_assign(&mut self, source: &mut Self) {
        self.move_assign_returning(source);
    }
}
impl<K: Debug, V, H: HashPolicy<K> + Default> Default for FlatDataMap<K, V, H> {
    fn default() -> Self {
        Self::new(0)
    }
}
impl<K: Clone, V: Clone, H: Clone> FlatDataMap<K, V, H> {
    /// Copies storage layout and hash policy; `map.assign(&source)`.
    pub fn assign(&mut self, source: &Self) {
        self.assign_returning(source);
    }
    /// Copies after releasing destination pairs and returns it, as `operator=`; `map.assign_returning(&source)`.
    pub fn assign_returning(&mut self, source: &Self) -> &mut Self {
        self.table.clone_from(&source.table);
        self
    }
}
impl<K: Clone, V: Clone, H: Clone> Clone for FlatDataMap<K, V, H> {
    fn clone(&self) -> Self {
        Self {
            table: self.table.clone(),
        }
    }
    fn clone_from(&mut self, source: &Self) {
        self.assign(source);
    }
}
impl<K: Debug, V, H: HashPolicy<K>> std::ops::Index<&K> for FlatDataMap<K, V, H> {
    type Output = V;
    fn index(&self, key: &K) -> &V {
        self.find(key)
            .expect("FlatDataMap index must be a bound key")
    }
}
impl<K: Debug, V, H: HashPolicy<K>> std::ops::IndexMut<&K> for FlatDataMap<K, V, H> {
    fn index_mut(&mut self, key: &K) -> &mut V {
        self.change_find(key)
            .expect("FlatDataMap index must be a bound key")
    }
}
fn missing_flat_key<K: Debug>(key: &K) -> Failure {
    Failure::with_type(
        "Standard_NoSuchObject",
        &format!("FlatDataMap key {key:?}; expected a bound key"),
        "",
    )
}
mod binding;
mod cursor;
mod iteration;
pub use cursor::FlatDataCursorMut;
mod transfer;

mod borrowed_emplacement;
mod cursor_contracts;
mod pointer_contracts;

impl<K, V, H> FlatDataMap<K, V, H> {
    /// Preserves self-assignment without copying keys, values or policy; `map.assign_self()`.
    pub fn assign_self(&mut self) -> &mut Self {
        self
    }
}
