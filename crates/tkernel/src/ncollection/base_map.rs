//! The public surface `NCollection_BaseMap` gives every hashed map.
use super::{BaseAllocator, DataMap, DoubleMap, HashPolicy, IndexedDataMap, IndexedMap, Map};
use std::fmt::Debug;
mod basic_iterator;
mod basic_node;
mod bucket_scan;
pub use basic_iterator::BaseMapBasicIterator;
pub use basic_node::{BasicMapIteratorSource, BasicMapNode, BasicMapNodeMut};
mod bucket_iterator;
mod bucket_table;
mod link_node;
mod resize_outputs;
mod resize_plan;
use resize_plan::next_resize_count;
mod storage;
pub use bucket_iterator::BaseMapBucketIterator;
pub use link_node::MapLinkNode;
pub use resize_plan::BaseMapResize;
pub use storage::BaseMapStorage;
mod double_storage;
mod indexed_storage;
pub use double_storage::{BaseDoubleMapResize, BaseDoubleMapStorage};
pub use indexed_storage::BaseIndexedMapStorage;
mod indexed_cursor;
mod indexed_source;
pub use indexed_cursor::BaseMapIndexedIterator;
pub use indexed_source::{IndexedCursorSource, IndexedCursorSourceMut};

/// Bucket and size queries shared by the hashed maps, as `NCollection_BaseMap` declares them.
///
/// ```
/// use rustycad_tkernel::ncollection::{BaseMap, Map};
/// let mut map = Map::<i32>::default();
/// map.add(7);
/// assert_eq!((BaseMap::extent(&map), BaseMap::nb_buckets(&map)), (1, 101));
/// ```
pub trait BaseMap {
    /// `NbBuckets()`: the hash modulus, kept even after `Clear(true)` releases the buckets.
    fn nb_buckets(&self) -> usize;
    /// `Size()`: the number of stored entries.
    fn size(&self) -> usize;
    /// `Allocator()`: the node allocator.
    fn allocator(&self) -> &BaseAllocator;
    /// `Extent()`: the entry count narrowed like the native `static_cast<int>`.
    fn extent(&self) -> i32 {
        self.size() as i32
    }
    /// `Length()`: the same narrowed entry count as `Extent()`.
    fn length(&self) -> i32 {
        self.size() as i32
    }
    /// `IsEmpty()`: true when no entry is stored.
    fn is_empty(&self) -> bool {
        self.size() == 0
    }
}

impl<K: Debug, V, H: HashPolicy<K>> BaseMap for DataMap<K, V, H> {
    fn nb_buckets(&self) -> usize {
        self.bucket_count()
    }
    fn size(&self) -> usize {
        self.len()
    }
    fn allocator(&self) -> &BaseAllocator {
        DataMap::allocator(self)
    }
}

impl<K: Debug, H: HashPolicy<K>> BaseMap for Map<K, H> {
    fn nb_buckets(&self) -> usize {
        self.bucket_count()
    }
    fn size(&self) -> usize {
        self.len()
    }
    fn allocator(&self) -> &BaseAllocator {
        Map::allocator(self)
    }
}

impl<K: Debug, H: HashPolicy<K>> BaseMap for IndexedMap<K, H> {
    fn nb_buckets(&self) -> usize {
        self.bucket_count()
    }
    fn size(&self) -> usize {
        self.len()
    }
    fn allocator(&self) -> &BaseAllocator {
        IndexedMap::allocator(self)
    }
}

impl<K: Debug, V, H: HashPolicy<K>> BaseMap for IndexedDataMap<K, V, H> {
    fn nb_buckets(&self) -> usize {
        self.bucket_count()
    }
    fn size(&self) -> usize {
        self.len()
    }
    fn allocator(&self) -> &BaseAllocator {
        IndexedDataMap::allocator(self)
    }
}

impl<A: Debug, B: Debug, H1: HashPolicy<A>, H2: HashPolicy<B>> BaseMap for DoubleMap<A, B, H1, H2> {
    fn nb_buckets(&self) -> usize {
        self.bucket_count()
    }
    fn size(&self) -> usize {
        self.len()
    }
    fn allocator(&self) -> &BaseAllocator {
        DoubleMap::allocator(self)
    }
}

/// `Clear(const Handle(NCollection_BaseAllocator)&)`: bucket memory is released only when the
/// allocator changes, and a null handle (`None`) selects the common allocator.
pub(super) fn replace_allocator(
    current: &mut BaseAllocator,
    requested: Option<BaseAllocator>,
) -> bool {
    let release = requested.as_ref() != Some(current);
    *current = requested.unwrap_or_else(BaseAllocator::common);
    release
}

/// The Standard_NoSuchObject an OCCT map `Iterator` raises when read past its end.
pub(super) fn exhausted_iterator(method: &str) -> crate::standard::Failure {
    crate::standard::Failure::with_type(
        "Standard_NoSuchObject",
        &format!("{method}: iterator exhausted; expected More() to be true"),
        "",
    )
}
