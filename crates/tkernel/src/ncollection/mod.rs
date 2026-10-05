//! Collections preserving OCCT's indexing and storage contracts.
mod array1;
pub use array1::Array1;
mod array_error;
pub use array_error::ArrayError;
mod h_array1_of_real;
pub use h_array1_of_real::HArray1OfReal;
mod local_array;
pub use local_array::LocalArray;
mod array2;
pub use array2::Array2;
mod generic_transient;
mod harray;
pub use harray::{HArray1, HArray2};
mod linear_vector;
pub use linear_vector::LinearVector;
mod vector;
pub use vector::{ElementCast, Vec2, Vec3, Vec4, VectorScalarDivision};
mod square_matrix;
pub use square_matrix::MatrixScalar;
mod mat3;
pub use mat3::Mat3;
mod lerp;
mod mat4;
pub use lerp::Lerp;
pub use mat4::Mat4;
// The allocator trait requires unsafe implementation; ownership proofs are local.
#[allow(unsafe_code)]
mod allocator;
pub use allocator::{
    ACC_ALIGN, ACC_DEFAULT_BLOCK_SIZE, ACC_MAX_LOOKUP_BLOCKS, AccAlignedPtr, AccAlignedSize,
    AccAllocatorExtension, AccBlock, AccBlockKey, AccBlockKeyHasher, AccBlockLookup, AccBlockState,
    AllocatedArray, AllocatedBox, AllocatedBytes, BaseAllocator, INC_DEFAULT_BLOCK_SIZE,
    INC_MINIMUM_BLOCK_SIZE, IncBlock, IncBlockSizeLevel,
};
mod dynamic_array;
pub use dynamic_array::{DynamicArray, DynamicCursor};

mod buffer;
pub use buffer::Buffer;
mod object_handle;
pub use object_handle::ObjectHandle;
mod shared;
pub use shared::Shared;

// Raw typed allocation forwards to the allocator's unsafe free/realloc contracts.
#[allow(unsafe_code)]
mod occ_allocator;
pub use occ_allocator::{AllocatorTypes, AllocatorVector, OccAllocator, StandardAllocator};

mod list;
pub use list::List as AllocatorList;
pub use list::{List, ListCursor, ListIter, ListIterMut};

mod sparse_array;
pub use sparse_array::SparseArray;

mod sequence;
pub use sequence::{Sequence, SequenceCursor, SequenceIter, SequenceIterMut};
mod h_sequence;
pub use h_sequence::HSequence;
mod hash_policy;
pub use hash_policy::{DefaultHashPolicy, HashKey, HashPolicy, buckets_from_int, next_map_prime};
mod base_map;
pub use base_map::BaseMap;
mod data_map;
pub use data_map::{DataMap, DataMapItems, DataMapIterator, DataMapIteratorMut};

mod map;
pub use map::{Map, MapIter, MapIterator};
pub mod map_algo;

mod indexed_map;
pub use indexed_map::{IndexedMap, IndexedMapItems, IndexedMapIter, IndexedMapIterator};

mod indexed_data_map;
pub use indexed_data_map::{
    IndexedDataItems, IndexedDataMap, IndexedDataMapIterator, IndexedDataMapIteratorMut,
    IndexedDataValues, IndexedDataValuesMut,
};

mod double_map;
pub use double_map::{DoubleMap, DoubleMapItems, DoubleMapIterator, DoubleMapValues};

mod flat_map;
mod flat_table;
pub use flat_map::{FlatMap, FlatMapIter};

mod flat_data_map;
pub use flat_data_map::{
    FlatDataCursorMut, FlatDataItems, FlatDataMap, FlatDataValues, FlatDataValuesMut,
};

mod ordered_map;
mod ordered_table;
pub use ordered_map::{OrderedMap, OrderedMapIter};
pub use ordered_table::{OrderedNodeMut, OrderedNodeRef};

mod ordered_data_map;
pub use ordered_data_map::{
    OrderedDataCursor, OrderedDataItems, OrderedDataItemsMut, OrderedDataMap, OrderedDataValues,
    OrderedDataValuesMut,
};

pub mod forward_range;

mod cell_filter;
pub use cell_filter::{CellAction, CellFilter, CellGeometry, CellInspector};

mod ebtree;
mod ubtree;
pub use ebtree::EBTree;
pub use ubtree::{BoundingNode, BoundingVolume, FallibleTreeSelector, TreeSelector, UBTree};
mod ubtree_filler;
pub use ubtree_filler::{BoundingTreeInsertion, ObjBnd, UBTreeFiller};

mod packed_map;
pub use packed_map::{PackedCursor, PackedInteger, PackedMap, PackedMapIter, PackedWord};
pub mod packed_map_algo;

mod aliased_array;
pub use aliased_array::{AliasedArray, ArrayValue};

mod allocator_object;
pub use allocator_object::{
    AccumulatingAllocatorObject, AlignedAllocatorObject, AllocatorObject, HeapAllocatorObject,
    IncrementalAllocatorObject,
};
mod kdtree;
pub use kdtree::{KDTree, KdPoint};

mod utf_iterator;
pub use utf_iterator::{UtfBufferError, UtfIterator, UtfUnit, WideChar};
mod utf_string;
#[cfg(not(target_os = "android"))]
pub use utf_string::UtfStringTool;
pub use utf_string::{String8, UtfString};
mod collection_cursor;
pub use collection_cursor::{CollectionCursor, CollectionCursorMut, CursorPositions};
mod items_view;
pub use items_view::{
    ItemsSource, ItemsSourceMut, ItemsView, ItemsViewMut, KeyIndexItems, KeyValueIndexItems,
    KeyValueItems,
};
