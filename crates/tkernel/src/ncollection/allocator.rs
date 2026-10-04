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

//! Allocator handles: NCollection_BaseAllocator and its Heap, Inc, Acc and Aligned subclasses.
//!
//! The virtual `Allocate`/`AllocateOptimal`/`Free` interface becomes one shared handle whose
//! policy is chosen at construction. Raw allocation mirrors the native pointer API; RAII owners
//! (`AllocatedBytes`, `AllocatedBox`, `AllocatedArray`) and the `allocator_api2::Allocator`
//! implementation serve safe Rust code and collections.
// Unsafe is confined to implementing allocator-api2's contract and forwarding frees.
use crate::standard::Failure;
use allocator_api2::alloc::{AllocError, Allocator, Layout};
use std::{
    ptr::NonNull,
    sync::{Arc, Mutex, MutexGuard, OnceLock},
};
mod accumulating;
mod incremental;
mod owned;
mod raw_heap;
use accumulating::AccEngine;
pub use accumulating::{ACC_ALIGN, ACC_DEFAULT_BLOCK_SIZE, ACC_MAX_LOOKUP_BLOCKS, AccBlockState};
use incremental::IncEngine;
pub use incremental::{
    INC_DEFAULT_BLOCK_SIZE, INC_MINIMUM_BLOCK_SIZE, IncBlock, IncBlockSizeLevel,
};
pub use owned::{AllocatedArray, AllocatedBox, AllocatedBytes};
use raw_heap::{MALLOC_ALIGNMENT, heap_allocate, heap_free, mm_malloc_alignment};

/// A shared allocator identity and policy; `BaseAllocator::common().allocate(16)`.
#[derive(Clone)]
pub struct BaseAllocator {
    inner: Arc<AllocatorKind>,
}

/// The concrete NCollection allocator class behind a handle.
enum AllocatorKind {
    /// NCollection_BaseAllocator: `Standard::Allocate` (calloc) / `AllocateOptimal` (malloc).
    Common,
    /// NCollection_HeapAllocator: malloc of the size rounded up to 4 bytes.
    Heap,
    /// NCollection_IncAllocator.
    Incremental(Mutex<IncEngine>),
    /// NCollection_AccAllocator.
    Accumulating(Mutex<AccEngine>),
    /// NCollection_AlignedAllocator with its requested alignment.
    Aligned(usize),
}

fn locked<T>(engine: &Mutex<T>) -> MutexGuard<'_, T> {
    engine.lock().expect("allocator mutex poisoned")
}

impl BaseAllocator {
    fn with_kind(kind: AllocatorKind) -> Self {
        Self {
            inner: Arc::new(kind),
        }
    }

    /// The process-wide allocator (`CommonBaseAllocator`); `BaseAllocator::common() == BaseAllocator::common()`.
    pub fn common() -> Self {
        static COMMON: OnceLock<BaseAllocator> = OnceLock::new();
        COMMON
            .get_or_init(|| Self::with_kind(AllocatorKind::Common))
            .clone()
    }

    /// The process-wide malloc allocator (`GlobalHeapAllocator`), distinct from `common`.
    ///
    /// Example: `BaseAllocator::global_heap() != BaseAllocator::common()`.
    pub fn global_heap() -> Self {
        static HEAP: OnceLock<BaseAllocator> = OnceLock::new();
        HEAP.get_or_init(|| Self::with_kind(AllocatorKind::Heap))
            .clone()
    }

    /// A new incremental allocator (`NCollection_IncAllocator(theBlockSize)`); sizes below
    /// `INC_MINIMUM_BLOCK_SIZE` select the 12 KB default. Example: `BaseAllocator::incremental(12 * 1024)`.
    pub fn incremental(block_size: usize) -> Self {
        Self::with_kind(AllocatorKind::Incremental(Mutex::new(IncEngine::new(
            block_size,
        ))))
    }

    /// Accepts the native thread-safety request for an incremental allocator; its internal
    /// mutex remains enabled for Rust memory safety even when `is_thread_safe` is false.
    /// Example: `BaseAllocator::incremental(4096).set_thread_safe(true)?`.
    pub fn set_thread_safe(&self, _is_thread_safe: bool) -> Result<(), Failure> {
        let actual = match &*self.inner {
            AllocatorKind::Incremental(_) => return Ok(()),
            AllocatorKind::Common => "NCollection_BaseAllocator",
            AllocatorKind::Heap => "NCollection_HeapAllocator",
            AllocatorKind::Accumulating(_) => "NCollection_AccAllocator",
            AllocatorKind::Aligned(_) => "NCollection_AlignedAllocator",
        };
        Err(Failure::with_message(
            "Standard_TypeMismatch",
            &format!("{actual} cannot set thread safety; expected NCollection_IncAllocator"),
        ))
    }

    /// A new accumulating allocator (`NCollection_AccAllocator(theBlockSize)`), which allocates
    /// its first block at once. A zero size, which divides by zero natively, is refused.
    ///
    /// Example: `BaseAllocator::accumulating(ACC_DEFAULT_BLOCK_SIZE)?`.
    pub fn accumulating(block_size: usize) -> Result<Self, Failure> {
        let engine = AccEngine::new(block_size)?;
        Ok(Self::with_kind(AllocatorKind::Accumulating(Mutex::new(
            engine,
        ))))
    }

    /// A new aligned allocator (`NCollection_AlignedAllocator(theAlignment)`).
    ///
    /// Example: `BaseAllocator::aligned(16).allocate_raw(64)` is 16-byte aligned.
    pub fn aligned(alignment: usize) -> Self {
        Self::with_kind(AllocatorKind::Aligned(alignment))
    }

    /// Raw `Allocate(theSize)`; `None` where upstream returns NULL or throws Standard_OutOfMemory.
    ///
    /// The block stays allocated until `free_raw` (or, for arenas, `reset`/drop).
    /// Example: `let p = allocator.allocate_raw(16).unwrap(); unsafe { allocator.free_raw(p) }?`.
    pub fn allocate_raw(&self, size: usize) -> Option<NonNull<u8>> {
        self.allocate_policy(size, true)
    }

    /// Raw `AllocateOptimal(theSize)`: uninitialized `malloc` for the common allocator, the same
    /// policy as `allocate_raw` for every subclass. Example: `allocator.allocate_optimal_raw(8)`.
    pub fn allocate_optimal_raw(&self, size: usize) -> Option<NonNull<u8>> {
        self.allocate_policy(size, !matches!(*self.inner, AllocatorKind::Common))
    }

    /// Raw `Free(theAddress)`: heap kinds free, the incremental allocator ignores the call, the
    /// accumulating allocator counts it down and raises `Standard_ProgramError` for an address
    /// it does not own or a block with no live allocation.
    ///
    /// # Safety
    /// `address` must come from `allocate_raw`/`allocate_optimal_raw` of this allocator and not
    /// be freed yet; for the accumulating allocator a foreign address is detected, but a double
    /// free may release a block other allocations still use.
    pub unsafe fn free_raw(&self, address: NonNull<u8>) -> Result<(), Failure> {
        // SAFETY: the non-null pointer has the same origin/ownership requirements.
        unsafe { self.free_raw_nullable(address.as_ptr()) }
    }

    /// Raw nullable `Free(theAddress)`, including native allocator behavior for null.
    /// Example: `unsafe { allocator.free_raw_nullable(std::ptr::null_mut()) }?`.
    ///
    /// # Safety
    /// A non-null address must come from this allocator and still be live. Null is valid input;
    /// the accumulating allocator reports it as `Standard_ProgramError`.
    pub unsafe fn free_raw_nullable(&self, address: *mut u8) -> Result<(), Failure> {
        match &*self.inner {
            // SAFETY: forwarded from this function's contract.
            AllocatorKind::Common | AllocatorKind::Heap | AllocatorKind::Aligned(_) => unsafe {
                if let Some(address) = NonNull::new(address) {
                    heap_free(address);
                }
                Ok(())
            },
            AllocatorKind::Incremental(_) => Ok(()),
            AllocatorKind::Accumulating(engine) => locked(engine).free(address as usize),
        }
    }

    /// One allocation of the handle's policy; `zeroed` only matters for heap kinds.
    fn allocate_policy(&self, size: usize, zeroed: bool) -> Option<NonNull<u8>> {
        match &*self.inner {
            AllocatorKind::Common => heap_allocate(size, MALLOC_ALIGNMENT, zeroed),
            AllocatorKind::Heap => heap_allocate(heap_round(size)?, MALLOC_ALIGNMENT, zeroed),
            AllocatorKind::Incremental(engine) => locked(engine).allocate(size),
            AllocatorKind::Accumulating(engine) => locked(engine).allocate(size),
            AllocatorKind::Aligned(alignment) => {
                heap_allocate(size, mm_malloc_alignment(*alignment)?, zeroed)
            }
        }
    }

    /// Whether `Free(address)` would succeed, without freeing: only the accumulating allocator
    /// raises (`Standard_ProgramError` for an address it cannot find, including null).
    ///
    /// Example: `BaseAllocator::common().check_free(std::ptr::null()).is_ok()`.
    pub fn check_free(&self, address: *const u8) -> Result<(), Failure> {
        match &*self.inner {
            AllocatorKind::Accumulating(engine) => {
                locked(engine).freeable_block(address as usize).map(|_| ())
            }
            _ => Ok(()),
        }
    }

    /// Rewinds (`release == false`) or frees (`release == true`) an incremental allocator's
    /// blocks (`NCollection_IncAllocator::Reset`); other allocators have nothing to reset.
    ///
    /// Rust refuses while RAII or collection allocations from it are alive, because rewinding
    /// would alias them. Example: `allocator.reset(false)?`.
    pub fn reset(&self, release: bool) -> Result<(), Failure> {
        match &*self.inner {
            AllocatorKind::Incremental(engine) => locked(engine).reset(release),
            _ => Ok(()),
        }
    }

    /// The accumulating block owning `address` (protected `findBlock`), for inspection.
    ///
    /// Example: `allocator.accumulating_block(p.as_ptr() as usize).unwrap().alloc_count == 1`.
    pub fn accumulating_block(&self, address: usize) -> Option<AccBlockState> {
        match &*self.inner {
            AllocatorKind::Accumulating(engine) => locked(engine).block_state(address),
            _ => None,
        }
    }

    /// The accumulating blocks from the last one back (protected `mypLastBlock` chain).
    ///
    /// Example: `BaseAllocator::accumulating(64)?.accumulating_chain().len() == 1`.
    pub fn accumulating_chain(&self) -> Vec<AccBlockState> {
        match &*self.inner {
            AllocatorKind::Accumulating(engine) => locked(engine).chain(),
            _ => Vec::new(),
        }
    }

    /// `Standard::Reallocate` for blocks of the common allocator's raw policy.
    ///
    /// # Safety
    /// `address` must come from the common allocator's raw policy and not be freed yet.
    pub(crate) unsafe fn standard_reallocate(
        address: NonNull<u8>,
        size: usize,
    ) -> Option<NonNull<u8>> {
        // SAFETY: forwarded from this function's contract.
        unsafe { raw_heap::heap_reallocate(address, size) }
    }

    /// Stable identity address of the shared allocator object (native `myAllocator.get()`).
    pub fn identity(&self) -> *const () {
        Arc::as_ptr(&self.inner).cast()
    }
}

/// NCollection_HeapAllocator's `(theSize + 3) & ~0x3`; `None` where the addition would wrap.
fn heap_round(size: usize) -> Option<usize> {
    Some(size.checked_add(3)? & !3)
}

impl Default for BaseAllocator {
    fn default() -> Self {
        Self::common()
    }
}
impl PartialEq for BaseAllocator {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}
impl Eq for BaseAllocator {}
impl std::fmt::Debug for BaseAllocator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("BaseAllocator")
            .field(&Arc::as_ptr(&self.inner))
            .finish()
    }
}

impl BaseAllocator {
    /// Typed allocation for RAII owners and collections, honouring the layout alignment.
    fn allocate_typed(&self, layout: Layout) -> Option<NonNull<u8>> {
        let (size, align) = (layout.size(), layout.align());
        match &*self.inner {
            AllocatorKind::Common => heap_allocate(size, align, false),
            AllocatorKind::Heap => heap_allocate(heap_round(size)?, align, false),
            AllocatorKind::Incremental(engine) => {
                let mut engine = locked(engine);
                let address = engine.allocate_aligned(size, align)?;
                engine.acquire();
                Some(address)
            }
            AllocatorKind::Accumulating(engine) => locked(engine).allocate_aligned(size, align),
            AllocatorKind::Aligned(alignment) => {
                heap_allocate(size, mm_malloc_alignment(*alignment)?.max(align), false)
            }
        }
    }

    /// Releases a typed allocation made by `allocate_typed`.
    ///
    /// # Safety
    /// `address` must come from `allocate_typed` on this allocator and not be freed yet.
    unsafe fn free_typed(&self, address: NonNull<u8>) {
        match &*self.inner {
            AllocatorKind::Incremental(engine) => locked(engine).release(),
            AllocatorKind::Accumulating(engine) => {
                // Only a zero-size byte allocation at a fresh block's end can fail: upstream's
                // Free raises there and the allocation stays counted, which Drop cannot report.
                let _ = locked(engine).free(address.as_ptr() as usize);
            }
            // SAFETY: forwarded from this function's contract.
            _ => unsafe { heap_free(address) },
        }
    }
}

// SAFETY: every allocation is a live, correctly aligned region of at least `layout.size()`
// bytes: heap kinds own a dedicated block; arena kinds hand out disjoint ranges of blocks
// that stay alive while any clone of the handle exists. Arena blocks are only rewound or
// released by `reset`, which refuses while typed allocations are live (incremental), or
// when a block's count reaches zero (accumulating). Zero-size layouts never reach a policy.
unsafe impl Allocator for BaseAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        if layout.size() == 0 {
            // Box/Vec never free zero-size blocks, so they must not be counted by arenas.
            let dangling = NonNull::new(layout.align() as *mut u8).ok_or(AllocError)?;
            return Ok(NonNull::slice_from_raw_parts(dangling, 0));
        }
        let address = self.allocate_typed(layout).ok_or(AllocError)?;
        Ok(NonNull::slice_from_raw_parts(address, layout.size()))
    }
    unsafe fn deallocate(&self, pointer: NonNull<u8>, layout: Layout) {
        if layout.size() != 0 {
            // SAFETY: the Allocator caller returns a block of this allocator exactly once.
            unsafe { self.free_typed(pointer) };
        }
    }
}
