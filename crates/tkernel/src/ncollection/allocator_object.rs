// Purpose:     Basic class for memory allocation wizards.
//              Defines  the  interface  for devising  different  allocators
//              firstly to be used  by collections of NCollection, though it
//              it is not  deferred. It allocates/frees  the memory  through
//              Standard procedures, thus it is  unnecessary (and  sometimes
//              injurious) to have  more than one such  allocator.  To avoid
//              creation  of multiple  objects the  constructors  were  maid
//              inaccessible.  To  create the  BaseAllocator use  the method
//              CommonBaseAllocator.
//              Note that this object is managed by Handle.

//! Runtime-typed allocator objects delegate storage to the existing allocator implementation.
//!
//! Each OCCT allocator class is a transient whose descriptor chains to
//! `NCollection_BaseAllocator`, so handles cast between them as natively.
use super::{ACC_DEFAULT_BLOCK_SIZE, BaseAllocator, INC_DEFAULT_BLOCK_SIZE};
use crate::standard::{Failure, StandardTransient};
/// Polymorphic allocator object; `AllocatorObject::default().resource().allocate(16)`.
#[repr(C)]
#[derive(Default)]
pub struct AllocatorObject {
    parent: StandardTransient,
    resource: BaseAllocator,
}
impl AllocatorObject {
    /// Borrow the real allocation owner; `object.resource().allocate_value(42)`.
    pub fn resource(&self) -> &BaseAllocator {
        &self.resource
    }
    fn wrapping(resource: BaseAllocator) -> Self {
        Self {
            parent: StandardTransient::default(),
            resource,
        }
    }
}
crate::impl_transient!(
    AllocatorObject,
    "NCollection_BaseAllocator",
    StandardTransient,
    parent
);

/// Declares a transient allocator subclass embedding its `AllocatorObject` base.
macro_rules! allocator_subclass {
    ($(#[$doc:meta])* $name:ident, $occt:literal) => {
        $(#[$doc])*
        #[repr(C)]
        pub struct $name {
            parent: AllocatorObject,
        }
        impl $name {
            /// Borrow the backing allocator; `object.resource().allocate(16)`.
            pub fn resource(&self) -> &BaseAllocator {
                self.parent.resource()
            }
            fn wrapping(resource: BaseAllocator) -> Self {
                Self {
                    parent: AllocatorObject::wrapping(resource),
                }
            }
        }
        crate::impl_transient!($name, $occt, AllocatorObject, parent);
    };
}

allocator_subclass!(
    /// Incremental arena object supporting base-handle casts; `IncrementalAllocatorObject::default()`.
    IncrementalAllocatorObject,
    "NCollection_IncAllocator"
);
impl IncrementalAllocatorObject {
    /// `new NCollection_IncAllocator(theBlockSize)`; `IncrementalAllocatorObject::new(4096)`.
    pub fn new(block_size: usize) -> Self {
        Self::wrapping(BaseAllocator::incremental(block_size))
    }
}
impl Default for IncrementalAllocatorObject {
    /// The 12 KB default block size.
    fn default() -> Self {
        Self::new(INC_DEFAULT_BLOCK_SIZE)
    }
}

allocator_subclass!(
    /// Global-heap allocator object; `HeapAllocatorObject::default()`.
    HeapAllocatorObject,
    "NCollection_HeapAllocator"
);
impl Default for HeapAllocatorObject {
    /// Wraps the process-wide heap allocator (`GlobalHeapAllocator`), not the common one.
    fn default() -> Self {
        Self::wrapping(BaseAllocator::global_heap())
    }
}

allocator_subclass!(
    /// Accumulating allocator object; `AccumulatingAllocatorObject::new(1024)?`.
    AccumulatingAllocatorObject,
    "NCollection_AccAllocator"
);
impl AccumulatingAllocatorObject {
    /// `new NCollection_AccAllocator(theBlockSize)`; a zero size is refused.
    pub fn new(block_size: usize) -> Result<Self, Failure> {
        Ok(Self::wrapping(BaseAllocator::accumulating(block_size)?))
    }
    /// `new NCollection_AccAllocator()` with `DefaultBlockSize`.
    pub fn with_default_block_size() -> Self {
        Self::new(ACC_DEFAULT_BLOCK_SIZE).expect("the default block size is nonzero")
    }
}

allocator_subclass!(
    /// Aligned allocator object; `AlignedAllocatorObject::new(16)`.
    AlignedAllocatorObject,
    "NCollection_AlignedAllocator"
);
impl AlignedAllocatorObject {
    /// `new NCollection_AlignedAllocator(theAlignment)`; `AlignedAllocatorObject::new(32)`.
    pub fn new(alignment: usize) -> Self {
        Self::wrapping(BaseAllocator::aligned(alignment))
    }
}
