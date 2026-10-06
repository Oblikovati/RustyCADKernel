use super::{IndexedCursorSource, IndexedCursorSourceMut};
use crate::standard::Failure;
mod arithmetic;
mod comparisons;
mod value_policies;

/// OCCT's BaseMap::IndexedIterator with explicit owner arguments for safe access.
/// Example: `BaseMapIndexedIterator::<_, false>::new(&map, 1).value(&map)?`.
/// The owner pointer identifies the map and is never dereferenced; moving the map
/// invalidates existing cursor identities, just as moving a native container does.
pub struct BaseMapIndexedIterator<C, const MUTABLE: bool = false> {
    owner: *const C,
    index: usize,
}

impl<C, const M: bool> Default for BaseMapIndexedIterator<C, M> {
    fn default() -> Self {
        Self {
            owner: std::ptr::null(),
            index: 1,
        }
    }
}
impl<C, const M: bool> Copy for BaseMapIndexedIterator<C, M> {}
impl<C, const M: bool> Clone for BaseMapIndexedIterator<C, M> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<C, const M: bool> BaseMapIndexedIterator<C, M> {
    /// Attaches at any native one-based index, without range checking; `Self::new(&map, 1)`.
    pub fn new(container: &C, index: usize) -> Self {
        Self {
            owner: container,
            index,
        }
    }
    /// Reads the full native size_t index; `cursor.index()`.
    pub fn index(&self) -> usize {
        self.index
    }
    /// Returns this iterator through OCCT's Iterator wrapper; `cursor.iterator()`.
    pub fn iterator(&self) -> &Self {
        self
    }
    /// Returns this iterator exclusively through ChangeIterator; `cursor.change_iterator()`.
    pub fn change_iterator(&mut self) -> &mut Self {
        self
    }
    /// Copies owner and index, returning the destination; `cursor.assign(&source)`.
    pub fn assign(&mut self, source: &Self) -> &mut Self {
        *self = *source;
        self
    }
    /// Preserves both fields for native self assignment; `cursor.assign_self()`.
    pub fn assign_self(&mut self) -> &mut Self {
        self
    }
    /// Converts to the read-only native specialization; `cursor.as_const()`.
    pub fn as_const(&self) -> BaseMapIndexedIterator<C, false> {
        BaseMapIndexedIterator {
            owner: self.owner,
            index: self.index,
        }
    }
    fn check_owner(&self, container: &C) -> Result<(), Failure> {
        if self.owner == std::ptr::from_ref(container) {
            return Ok(());
        }
        Err(Failure::with_type(
            "Standard_DomainError",
            &format!(
                "indexed iterator owner {:p}; expected supplied map {:p}",
                self.owner, container
            ),
            "",
        ))
    }
}

impl<C> BaseMapIndexedIterator<C, false> {
    /// Assigns the mutable specialization without changing its position; `readonly.assign_from_mutable(&writable)`.
    pub fn assign_from_mutable(&mut self, source: &BaseMapIndexedIterator<C, true>) -> &mut Self {
        *self = source.as_const();
        self
    }
}

impl<C: IndexedCursorSource, const M: bool> BaseMapIndexedIterator<C, M> {
    /// Checks against the live size, including native index-zero behavior; `cursor.more(&map)?`.
    pub fn more(&self, container: &C) -> Result<bool, Failure> {
        self.more_with(container, C::indexed_size)
    }
    /// Reads the native dereference, forwarding the map's index error; `cursor.value(&map)?`.
    pub fn value<'a>(&self, container: &'a C) -> Result<&'a C::Value, Failure> {
        self.value_with(container, C::indexed_value)
    }
    /// Reads the current key separately from its mapped value; `cursor.key(&map)?`.
    pub fn key<'a>(&self, container: &'a C) -> Result<&'a C::Key, Failure> {
        self.key_with(container, C::indexed_key)
    }
    /// Returns the value address for native operator-> identity; `cursor.value_ptr(&map)?`.
    pub fn value_ptr(&self, container: &C) -> Result<*const C::Value, Failure> {
        self.value(container).map(std::ptr::from_ref)
    }
    /// Reads a signed offset without moving this cursor; `cursor.value_at(&map, 1)?`.
    pub fn value_at<'a>(&self, container: &'a C, offset: isize) -> Result<&'a C::Value, Failure> {
        self.value_at_with(container, offset, C::indexed_value)
    }
}

impl<C: IndexedCursorSourceMut> BaseMapIndexedIterator<C, true> {
    /// Mutates a mapped value while retaining cursor identity; `*cursor.change_value(&mut map)? = value`.
    pub fn change_value<'a>(&self, container: &'a mut C) -> Result<&'a mut C::Value, Failure> {
        self.change_value_with(container, C::indexed_value_mut)
    }
    /// Mutates the offset value through native writable subscript; `*cursor.change_value_at(&mut map, 1)? = value`.
    pub fn change_value_at<'a>(
        &self,
        container: &'a mut C,
        offset: isize,
    ) -> Result<&'a mut C::Value, Failure> {
        self.change_value_at_with(container, offset, C::indexed_value_mut)
    }
    /// Returns native mutable arrow identity; `cursor.change_value_ptr(&mut map)?`.
    pub fn change_value_ptr(&self, container: &mut C) -> Result<*mut C::Value, Failure> {
        self.change_value(container).map(std::ptr::from_mut)
    }
}
