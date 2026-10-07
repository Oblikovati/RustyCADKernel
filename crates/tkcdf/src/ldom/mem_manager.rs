// Rust port of ApplicationFramework/TKCDF/LDOM/LDOM_MemManager.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `LDOM_MemManager`: the storage of one LDOM document (ocaf-design.md section 9). The
//! native block allocator is replaced by the node arena of [`DocumentStore`]; the hash
//! table of names is kept.
use super::arena::LdomNodeId;
use super::basic_string::{LdomBasicString, LdomStringType};
use super::c_text::c_str;
use super::element_ops::compare_hashed;
use super::store::{DocumentStore, crc_hash};
use rustycad_tkernel::{
    impl_transient,
    standard::{SharedState, StandardTransient},
};
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

/// `LDOM_MemManager`; `Handle::new(LdomMemManager::new(10000))`.
///
/// ```
/// use rustycad_tkcdf::ldom::LdomMemManager;
/// let manager = LdomMemManager::new(10000);
/// let mut hash = 0;
/// let name = manager.hashed_allocate(b"name", 4, &mut hash);
/// assert_eq!(hash, LdomMemManager::hash(b"name", 4));
/// assert!(LdomMemManager::compare_strings(b"name", hash, &name));
/// ```
#[derive(Default)]
pub struct LdomMemManager {
    transient: StandardTransient,
    /// `myBlockSize`: the native block size in `int` units, kept for the record.
    block_size: i32,
    /// The nodes, the hash table (`myHashTable`) and the root (`myRootElement`).
    store: SharedState<DocumentStore>,
}
impl_transient!(
    LdomMemManager,
    "LDOM_MemManager",
    StandardTransient,
    transient
);

impl LdomMemManager {
    /// `LDOM_MemManager(aBlockSize)`: an empty document storage; the block size in bytes
    /// is rounded up to `int` units as natively.
    pub fn new(block_size: i32) -> Self {
        let block_size = (block_size - 1) / std::mem::size_of::<i32>() as i32 + 1;
        Self {
            block_size,
            ..Self::default()
        }
    }
    /// The block size in `int` units (`myBlockSize`).
    pub fn block_size(&self) -> i32 {
        self.block_size
    }
    /// `HashedAllocate(aString, theLen, theHash)`: the hashed copy of the first `length`
    /// bytes (shared with an equal name allocated before) and its hash in `hash`.
    pub fn hashed_allocate(&self, text: &[u8], length: i32, hash: &mut i32) -> LdomBasicString {
        let (stored, bucket) = self.write().hashed_allocate(prefix(text, length));
        *hash = bucket;
        let mut result = LdomBasicString::new();
        result.set_direct(LdomStringType::LdomAsciiHashed, stored);
        result
    }
    /// `HashedAllocate(aString, theLen, theResult)`: `result` becomes the hashed string.
    pub fn hashed_allocate_into(&self, text: &[u8], length: i32, result: &mut LdomBasicString) {
        let mut hash = 0;
        *result = self.hashed_allocate(text, length, &mut hash);
    }
    /// `Hash(theString, theLen)`: the CRC-16 bucket (0..=255) of the first `length` bytes.
    pub fn hash(text: &[u8], length: i32) -> i32 {
        crc_hash(prefix(text, length))
    }
    /// `CompareStrings(theString, theHashValue, theHashedStr)`: the hash stored with the
    /// hashed string is `hash_value` and the C strings are equal.
    pub fn compare_strings(text: &[u8], hash_value: i32, hashed: &LdomBasicString) -> bool {
        hashed
            .text_ref()
            .is_some_and(|stored| compare_hashed(c_str(text), hash_value, stored))
    }
    /// `Self`: this storage.
    pub fn self_ref(&self) -> &Self {
        self
    }
    /// `RootElement`: the element `createDocument` or the parser made the root.
    pub fn root_element(&self) -> Option<LdomNodeId> {
        self.read().root
    }
    /// Reads the storage.
    pub(crate) fn read(&self) -> RwLockReadGuard<'_, DocumentStore> {
        self.store.read()
    }
    /// Writes the storage.
    pub(crate) fn write(&self) -> RwLockWriteGuard<'_, DocumentStore> {
        self.store.write()
    }
}

/// The first `length` bytes of `text` (all of it when `length` is beyond it).
fn prefix(text: &[u8], length: i32) -> &[u8] {
    let length = usize::try_from(length).unwrap_or(0);
    &text[..length.min(text.len())]
}
