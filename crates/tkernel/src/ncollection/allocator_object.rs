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
use super::{ACC_DEFAULT_BLOCK_SIZE, AllocatorSource, BaseAllocator, INC_DEFAULT_BLOCK_SIZE};
use crate::standard::Handle;
use crate::standard::{Failure, StandardTransient};
use std::{ptr::NonNull, sync::OnceLock};
/// Polymorphic allocator object; `AllocatorObject::default().resource().allocate(16)`.
#[repr(C)]
pub struct AllocatorObject {
    parent: StandardTransient,
    resource: BaseAllocator,
}
impl AllocatorObject {
    /// Borrow the compatibility resource; `BaseAllocator::from_handle(&owner); owner.resource().allocate_value(42)`.
    /// Derived virtual dispatch requires prior `from_handle` registration; standalone objects use
    /// only their embedded policy. Registration keeps a weak original-object alias, never a copy.
    pub fn resource(&self) -> &BaseAllocator {
        // A borrowed policy does not acquire intrusive allocator ownership.
        &self.resource
    }
    /// Native static const handle; `AllocatorObject::common_base_allocator().ref_count()`.
    pub fn common_base_allocator() -> &'static Handle<Self> {
        static OWNER: OnceLock<Handle<AllocatorObject>> = OnceLock::new();
        OWNER.get_or_init(|| Handle::new(Self::default()))
    }
    /// Invoke the parent Allocate body; `object.allocate_base(16)`.
    pub fn allocate_base(&self, size: usize) -> Option<NonNull<u8>> {
        // Bypass derived dispatch to prevent recursive base-body calls.
        self.resource.allocate_policy(size, true)
    }
    /// Invoke independent parent AllocateOptimal; `object.allocate_optimal_base(16)`.
    pub fn allocate_optimal_base(&self, size: usize) -> Option<NonNull<u8>> {
        // The native base Optimal body is independent of virtual Allocate.
        self.resource.allocate_optimal_builtin(size)
    }
    /// Invoke parent Free, bypassing derived dispatch; `unsafe { object.free_base(pointer) }?`.
    /// # Safety
    /// The nullable pointer must belong to this parent's live allocation policy.
    pub unsafe fn free_base(&self, pointer: *mut u8) -> Result<(), Failure> {
        // SAFETY: the caller supplies the original nullable allocation pointer.
        unsafe { self.resource.free_builtin_nullable(pointer) }
    }
    pub(super) fn allocate_calls_optimal(&self) -> bool {
        // Read the actual embedded policy without creating or retargeting a handled base.
        self.resource.allocate_calls_optimal()
    }
    pub(super) fn optimal_calls_allocate(&self) -> bool {
        // The inherited native forwarding body remains virtual in derived allocators.
        self.resource.optimal_calls_allocate()
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
    parent,
    interfaces: [dyn AllocatorSource]
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
                // The compatibility façade aliases the actual embedded base policy.
                self.parent.resource()
            }
            fn wrapping(resource: BaseAllocator) -> Self {
                Self {
                    parent: AllocatorObject::wrapping(resource),
                }
            }
        }
        crate::impl_transient!($name, $occt, AllocatorObject, parent, interfaces: [dyn AllocatorSource]);
        // SAFETY: these concrete policies own disjoint allocations until their documented release.
        unsafe impl AllocatorSource for $name {
            fn allocator_object_base(&self) -> &AllocatorObject {
                // Project the embedded parent without changing object identity or counts.
                &self.parent
            }
        }
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
        // Construction creates an unregistered base until the genuine containing handle exists.
        Self::wrapping(BaseAllocator::incremental_policy(block_size))
    }
}
impl Default for IncrementalAllocatorObject {
    /// The 12 KB default block size.
    fn default() -> Self {
        // Preserve the native default block size without creating an extra handled allocator.
        Self::new(INC_DEFAULT_BLOCK_SIZE)
    }
}

allocator_subclass!(
    /// Global-heap allocator object; `HeapAllocatorObject::default()`.
    HeapAllocatorObject,
    "NCollection_HeapAllocator"
);
impl Default for HeapAllocatorObject {
    /// Creates its own protected heap base; the process-wide handle is a separate identity.
    fn default() -> Self {
        // Fresh protected construction must not alias the process-wide heap object.
        Self::wrapping(BaseAllocator::heap_policy())
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
        Ok(Self::wrapping(BaseAllocator::accumulating_policy(
            block_size,
        )?))
    }
    /// `new NCollection_AccAllocator()` with `DefaultBlockSize`.
    pub fn with_default_block_size() -> Self {
        // The native default size is valid without exceptional construction.
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
        // Keep the alignment on this exact embedded base, not a replacement handled object.
        Self::wrapping(BaseAllocator::aligned_policy(alignment))
    }
}

impl Default for AllocatorObject {
    fn default() -> Self {
        // Protected construction starts with native count zero and no self-owning alias.
        Self::wrapping(BaseAllocator::common_policy())
    }
}
// SAFETY: Standard allocation provides disjoint ranges and matching nullable Free.
unsafe impl AllocatorSource for AllocatorObject {
    fn allocator_object_base(&self) -> &AllocatorObject {
        // The concrete base is its own canonical AllocatorSource projection.
        self
    }
}
impl HeapAllocatorObject {
    /// Borrow native GlobalHeapAllocator's actual handle; `HeapAllocatorObject::global_heap_allocator()`.
    pub fn global_heap_allocator() -> &'static Handle<Self> {
        static OWNER: OnceLock<Handle<HeapAllocatorObject>> = OnceLock::new();
        OWNER.get_or_init(|| Handle::new(Self::default()))
    }
}
