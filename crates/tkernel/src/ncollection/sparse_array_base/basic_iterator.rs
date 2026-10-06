//! Generic native cursor caches its end span and block state, with custom item projection.
use super::{SparseArrayBase, SparseArrayBlock};
use std::marker::PhantomData;
mod access;
mod movement;

/// Custom sparse-container projection; `unsafe impl BasicSparseArraySource<i32> for MySparse`.
/// # Safety
/// Storage must remain the same valid raw base while borrowed. Item projection must return
/// correctly aligned, initialized V objects for occupied slots and preserve pointer provenance.
pub unsafe trait BasicSparseArraySource<V> {
    /// Provides native inherited storage; `container.sparse_storage()`.
    fn sparse_storage(&self) -> &SparseArrayBase;
    /// Dispatches native Capacity, which may be overridden; `container.sparse_capacity()`.
    fn sparse_capacity(&self) -> usize {
        self.sparse_storage().capacity()
    }
    /// Projects native getItem, including custom layouts; `unsafe { source.sparse_item(block, 0) }`.
    /// # Safety
    /// The block and offset must identify a live occupied item in this container.
    unsafe fn sparse_item(&self, block: SparseArrayBlock, offset: usize) -> *mut V {
        unsafe { self.sparse_storage().get_item(block, offset).cast() }
    }
}

/// Writable custom sparse projection; `unsafe impl BasicSparseArraySourceMut<i32> for MySparse {}`.
/// # Safety
/// Projected objects must permit mutation through raw pointers when callers ensure exclusivity.
pub unsafe trait BasicSparseArraySourceMut<V>: BasicSparseArraySource<V> {}

/// Native BasicIterator for an arbitrary container/value pair; `BaseSparseArrayBasicIterator::<C, i32>::new(&source)`.
/// Mutable copies share raw identities, so dereferencing either specialization remains unsafe.
pub struct BaseSparseArrayBasicIterator<
    'a,
    C: BasicSparseArraySource<V>,
    V,
    const MUTABLE: bool = false,
> {
    source: Option<&'a C>,
    index: usize,
    end: usize,
    block_number: usize,
    offset: usize,
    block: Option<SparseArrayBlock>,
    more: bool,
    value_type: PhantomData<fn() -> V>,
}
impl<C: BasicSparseArraySource<V>, V, const M: bool> Copy
    for BaseSparseArrayBasicIterator<'_, C, V, M>
{
}
impl<C: BasicSparseArraySource<V>, V, const M: bool> Clone
    for BaseSparseArrayBasicIterator<'_, C, V, M>
{
    fn clone(&self) -> Self {
        *self
    }
}
impl<C: BasicSparseArraySource<V>, V, const M: bool> Default
    for BaseSparseArrayBasicIterator<'_, C, V, M>
{
    fn default() -> Self {
        Self {
            source: None,
            index: 0,
            end: 0,
            block_number: 0,
            offset: 0,
            block: None,
            more: false,
            value_type: PhantomData,
        }
    }
}
impl<'a, C: BasicSparseArraySource<V>, V, const M: bool> BaseSparseArrayBasicIterator<'a, C, V, M> {
    /// Starts native traversal and captures Capacity once; `Self::new(&source)`.
    pub fn new(source: &'a C) -> Self {
        Self::from_index(source, 0)
    }
    /// Starts at an explicit index, preserving beyond-end indices; `Self::from_index(&source, 5)`.
    pub fn from_index(source: &'a C, index: usize) -> Self {
        let mut cursor = Self {
            source: Some(source),
            index,
            end: source.sparse_capacity(),
            ..Self::default()
        };
        // SAFETY: the trait guarantees currently valid storage when borrowed for construction.
        unsafe { cursor.skip_unset() };
        cursor
    }
    /// Reports cached native More; `cursor.more()`.
    pub fn more(&self) -> bool {
        self.more
    }
    /// Returns native Key; `cursor.key()`.
    pub fn key(&self) -> usize {
        self.index
    }
    /// Returns native Index alias; `cursor.index()`.
    pub fn index(&self) -> usize {
        self.index
    }
    /// Assigns the same specialization and returns destination; `target.assign(&source)`.
    pub fn assign(&mut self, source: &Self) -> &mut Self {
        *self = *source;
        self
    }
    /// Preserves native self-assignment identity; `cursor.assign_self()`.
    pub fn assign_self(&mut self) -> &mut Self {
        self
    }
    /// Performs native mutable-to-const conversion; `mutable.as_const()`.
    pub fn as_const(&self) -> BaseSparseArrayBasicIterator<'a, C, V, false> {
        BaseSparseArrayBasicIterator {
            source: self.source,
            index: self.index,
            end: self.end,
            block_number: self.block_number,
            offset: self.offset,
            block: self.block,
            more: self.more,
            value_type: PhantomData,
        }
    }
    /// Compares native container/index identities or any two exhausted states; `left.is_equal(&right)`.
    pub fn is_equal<const O: bool>(
        &self,
        other: &BaseSparseArrayBasicIterator<'_, C, V, O>,
    ) -> bool {
        if !self.more && !other.more {
            return true;
        }
        let owners = match (self.source, other.source) {
            (Some(a), Some(b)) => std::ptr::eq(a, b),
            (None, None) => true,
            _ => false,
        };
        owners && self.index == other.index
    }
}
impl<'a, C: BasicSparseArraySource<V>, V> BaseSparseArrayBasicIterator<'a, C, V, false> {
    /// Converts a mutable source on assignment; `readonly.assign_mutable(&mutable)`.
    pub fn assign_mutable(
        &mut self,
        source: &BaseSparseArrayBasicIterator<'a, C, V, true>,
    ) -> &mut Self {
        *self = source.as_const();
        self
    }
}
impl<C: BasicSparseArraySource<V>, V, const M: bool, const O: bool>
    PartialEq<BaseSparseArrayBasicIterator<'_, C, V, O>>
    for BaseSparseArrayBasicIterator<'_, C, V, M>
{
    fn eq(&self, other: &BaseSparseArrayBasicIterator<'_, C, V, O>) -> bool {
        self.is_equal(other)
    }
}
impl<C: BasicSparseArraySource<V>, V, const M: bool> Eq
    for BaseSparseArrayBasicIterator<'_, C, V, M>
{
}
