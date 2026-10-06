//! Raw typed-node cursor: advancing/dereferencing requires the native node lifetime contract.
#![allow(unsafe_code)]
use super::{BasicMapIteratorSource, BasicMapNode, MapLinkNode, bucket_scan::scan_bucket_links};
mod access;
mod construction;

/// Native forward `BasicIterator` state, distinct from the legacy bucket-sensitive cursor.
/// Example: `let cursor = BaseMapBasicIterator::<MyNode>::new(&map); assert!(cursor.more());`.
/// `MUTABLE` selects the writable specialization, `KEY_ONLY` prohibits writable keys, and
/// `ORDERED` follows `OrderNext` instead of bucket chains. Copies share raw node identities;
/// raw-node access remains unsafe so those copies cannot manufacture aliased Rust references.
pub struct BaseMapBasicIterator<
    'a,
    N: BasicMapNode,
    const MUTABLE: bool = false,
    const KEY_ONLY: bool = false,
    const ORDERED: bool = false,
> {
    table: Option<&'a [*mut MapLinkNode]>,
    bucket: usize,
    node: *mut N,
}

impl<N: BasicMapNode, const M: bool, const K: bool, const O: bool> Copy
    for BaseMapBasicIterator<'_, N, M, K, O>
{
}
impl<N: BasicMapNode, const M: bool, const K: bool, const O: bool> Clone
    for BaseMapBasicIterator<'_, N, M, K, O>
{
    fn clone(&self) -> Self {
        *self
    }
}
impl<N: BasicMapNode, const M: bool, const K: bool, const O: bool> Default
    for BaseMapBasicIterator<'_, N, M, K, O>
{
    fn default() -> Self {
        Self {
            table: None,
            bucket: 0,
            node: std::ptr::null_mut(),
        }
    }
}

impl<'a, N: BasicMapNode, const M: bool, const K: bool, const O: bool>
    BaseMapBasicIterator<'a, N, M, K, O>
{
    /// Reports native `More()` without dereferencing a node; `cursor.more()`.
    pub fn more(&self) -> bool {
        !self.node.is_null()
    }
    /// Returns native const `Node()` identity in every specialization; `cursor.node()`.
    pub fn node(&self) -> *const N {
        self.node.cast_const()
    }
    /// Compares only node identity, including ends from different containers; `left.is_equal(&right)`.
    pub fn is_equal<const OTHER: bool>(
        &self,
        other: &BaseMapBasicIterator<'_, N, OTHER, K, O>,
    ) -> bool {
        self.node == other.node
    }
    /// Copies native iterator state and returns the destination; `target.assign(&source)`.
    pub fn assign(&mut self, source: &Self) -> &mut Self {
        *self = *source;
        self
    }
    /// Preserves state for native self assignment; `cursor.assign_self()`.
    pub fn assign_self(&mut self) -> &mut Self {
        self
    }
    /// Copies native mutable-to-const state without advancing; `cursor.as_const()`.
    pub fn as_const(&self) -> BaseMapBasicIterator<'a, N, false, K, O> {
        BaseMapBasicIterator {
            table: self.table,
            bucket: self.bucket,
            node: self.node,
        }
    }
    /// Performs native `Next()`, doing nothing at any exhausted state; `unsafe { cursor.advance() }`.
    ///
    /// # Safety
    /// A nonnull current node must be live, aligned and readable through this call.
    pub unsafe fn advance(&mut self) {
        if self.node.is_null() {
            return;
        }
        // SAFETY: caller guarantees the current node can be read through this call.
        let node = unsafe { &*self.node };
        self.node = if O {
            node.order_next()
        } else {
            node.bucket_next()
        };
        if self.node.is_null() && !O && self.table.is_some() {
            self.bucket += 1;
            self.scan_buckets();
        }
    }
    /// Implements native prefix increment's returned identity; `unsafe { cursor.advance_returning() }`.
    ///
    /// # Safety
    /// The current node must satisfy `advance`'s readable-node contract.
    pub unsafe fn advance_returning(&mut self) -> &mut Self {
        // SAFETY: caller forwards the same readable-node contract.
        unsafe { self.advance() };
        self
    }
    /// Implements native postfix increment's old-state copy; `let old = unsafe { cursor.advance_post() };`.
    ///
    /// # Safety
    /// The current node must satisfy `advance`'s readable-node contract.
    pub unsafe fn advance_post(&mut self) -> Self {
        let previous = *self;
        // SAFETY: caller forwards the same readable-node contract.
        unsafe { self.advance() };
        previous
    }
    fn scan_buckets(&mut self) {
        if let Some(table) = self.table {
            self.node = scan_bucket_links(table, &mut self.bucket).cast();
        }
    }
}

impl<N: BasicMapNode, const M: bool, const OTHER: bool, const K: bool, const O: bool>
    PartialEq<BaseMapBasicIterator<'_, N, OTHER, K, O>> for BaseMapBasicIterator<'_, N, M, K, O>
{
    fn eq(&self, other: &BaseMapBasicIterator<'_, N, OTHER, K, O>) -> bool {
        self.is_equal(other)
    }
}
impl<N: BasicMapNode, const M: bool, const K: bool, const O: bool> Eq
    for BaseMapBasicIterator<'_, N, M, K, O>
{
}
