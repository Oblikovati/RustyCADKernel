//! Native legacy Iterator identity includes its bucket position, including past the end.
#![allow(unsafe_code)]
use super::{BaseMap, BaseMapStorage, MapLinkNode, bucket_scan::scan_bucket_links};

/// A borrowed bucket table with native legacy `BaseMap::Iterator` state.
/// Example: `let cursor = BaseMapBucketIterator::new(&map); assert!(!cursor.more());`.
/// The borrowed table cannot be resized while the cursor lives; nodes remain caller-owned.
#[derive(Clone, Debug)]
pub struct BaseMapBucketIterator<'a> {
    table: Option<&'a [*mut MapLinkNode]>,
    bucket_count: usize,
    bucket: usize,
    node: *mut MapLinkNode,
}

impl Default for BaseMapBucketIterator<'_> {
    fn default() -> Self {
        Self {
            table: None,
            bucket_count: 0,
            bucket: 0,
            node: std::ptr::null_mut(),
        }
    }
}

impl<'a> BaseMapBucketIterator<'a> {
    /// Reads the first occupied bucket without dereferencing nodes; `BaseMapBucketIterator::new(&map)`.
    pub fn new(map: &'a BaseMapStorage) -> Self {
        let mut cursor = Self::default();
        cursor.initialize(map);
        cursor
    }
    /// Attaches to another map and starts from bucket zero; `cursor.initialize(&map)`.
    pub fn initialize(&mut self, map: &'a BaseMapStorage) {
        self.table = map.table.as_ref().map(|_| map.bucket_slots());
        self.bucket_count = map.nb_buckets();
        self.reset();
    }
    /// Rescans the attached table from bucket zero; `cursor.reset()`.
    pub fn reset(&mut self) {
        self.bucket = 0;
        self.node = std::ptr::null_mut();
        self.find_first();
    }
    /// Tests native `pMore` without accessing the node; `cursor.more()`.
    pub fn more(&self) -> bool {
        !self.node.is_null()
    }
    /// Returns the current raw node identity; `cursor.node()`.
    pub fn node(&self) -> *mut MapLinkNode {
        self.node
    }
    /// Exposes the protected native bucket position; `cursor.bucket_index()`.
    pub fn bucket_index(&self) -> usize {
        self.bucket
    }
    /// Matches `IsEqual`, comparing bucket and node even for exhausted cursors; `a.is_equal(&b)`.
    pub fn is_equal(&self, other: &Self) -> bool {
        self.bucket == other.bucket && self.node == other.node
    }
    /// Performs native `pNext`, including advancing an allocated exhausted cursor; `unsafe { cursor.advance() }`.
    ///
    /// # Safety
    /// A nonnull current node must remain a valid readable `MapLinkNode` through this call.
    pub unsafe fn advance(&mut self) {
        if self.table.is_none() {
            return;
        }
        if !self.node.is_null() {
            // SAFETY: caller guarantees the node is readable through this call.
            self.node = unsafe { (*self.node).next() };
            if !self.node.is_null() {
                return;
            }
        }
        self.bucket = self.bucket.wrapping_add(1);
        self.find_first();
    }
    fn find_first(&mut self) {
        let Some(table) = self.table else {
            return;
        };
        self.node = scan_bucket_links(&table[..=self.bucket_count], &mut self.bucket);
    }
}

impl PartialEq for BaseMapBucketIterator<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.is_equal(other)
    }
}
impl Eq for BaseMapBucketIterator<'_> {}
