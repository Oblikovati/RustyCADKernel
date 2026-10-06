//! Native typed allocator moves retain source handles; hint and count are ignored on free.
use super::{BaseAllocator, Failure, NonNull, OccAllocator, StandardAllocator};

impl<T> OccAllocator<T> {
    /// Native propagate_on_container_move_assignment is false; `assert!(!OccAllocator::<i32>::PROPAGATE_ON_CONTAINER_MOVE_ASSIGNMENT)`.
    pub const PROPAGATE_ON_CONTAINER_MOVE_ASSIGNMENT: bool = false;
    /// Copies a possibly null allocator handle; `OccAllocator::<i32>::with_allocator_ref(&resource)`.
    pub fn with_allocator_ref(resource: &Option<BaseAllocator>) -> Self {
        Self::with_allocator(resource.clone())
    }
    /// Matches the native rvalue-handle constructor, which retains its donor; `OccAllocator::<i32>::with_allocator_move_from(&mut resource)`.
    pub fn with_allocator_move_from(resource: &mut Option<BaseAllocator>) -> Self {
        Self::with_allocator_ref(resource)
    }
    /// Copies the native handle during move construction, leaving the donor intact; `OccAllocator::move_from(&mut source)`.
    pub fn move_from(source: &mut Self) -> Self {
        source.clone()
    }
    /// Copies another element type's handle and returns this allocator; `target.assign_returning(&source)`.
    pub fn assign_returning<U>(&mut self, source: &OccAllocator<U>) -> &mut Self {
        self.resource = source.resource.clone();
        self
    }
    /// Performs native move assignment without emptying the donor; `target.move_assign_returning(&mut source)`.
    pub fn move_assign_returning(&mut self, source: &mut Self) -> &mut Self {
        self.assign_returning(source)
    }
    /// Preserves self-copy and self-move identity; `allocator.assign_self()`.
    pub fn assign_self(&mut self) -> &mut Self {
        self
    }
    /// Ignores the optional native placement hint; `allocator.allocate_with_hint(4, std::ptr::null())`.
    pub fn allocate_with_hint(&self, count: usize, _hint: *const ()) -> Option<NonNull<T>> {
        self.allocate(count)
    }
    /// Releases a possibly null pointer of another element type through the native resource; `unsafe { allocator.deallocate_ptr(pointer, 1) }`.
    ///
    /// # Safety
    /// A non-null pointer must come from this allocator's raw allocation policy and still be live.
    pub unsafe fn deallocate_ptr<U>(&self, pointer: *mut U, _count: usize) -> Result<(), Failure> {
        // SAFETY: forwards raw pointer provenance without adding typed allocator headers.
        unsafe { self.effective().free_raw_nullable(pointer.cast()) }
    }
}

impl<T> StandardAllocator<T> {
    /// Ignores a nullable native allocator handle; `StandardAllocator::<i32>::from_optional_base(None)`.
    pub fn from_optional_base(_resource: Option<&BaseAllocator>) -> Self {
        Self::new()
    }
    /// Returns this stateless allocator on cross-type assignment; `target.assign_returning(&source)`.
    pub fn assign_returning<U>(&mut self, _source: &StandardAllocator<U>) -> &mut Self {
        self
    }
    /// Preserves self-assignment identity; `allocator.assign_self()`.
    pub fn assign_self(&mut self) -> &mut Self {
        self
    }
    /// Ignores the optional native placement hint; `allocator.allocate_with_hint(4, std::ptr::null())`.
    pub fn allocate_with_hint(&self, count: usize, _hint: *const ()) -> Option<NonNull<T>> {
        self.allocate(count)
    }
    /// Releases raw typed storage, treating null as native free(null); `unsafe { allocator.deallocate_ptr(std::ptr::null_mut(), 0) }`.
    ///
    /// # Safety
    /// A non-null pointer must come from a standard allocator's raw allocate/reallocate and still be live.
    pub unsafe fn deallocate_ptr(&self, pointer: *mut T, _count: usize) -> Result<(), Failure> {
        // SAFETY: raw allocation provenance is required by the caller's contract.
        unsafe { BaseAllocator::common().free_raw_nullable(pointer.cast()) }
    }
    /// Handles native reallocate(null, count) as allocation; `unsafe { allocator.reallocate_ptr(std::ptr::null_mut(), 4) }`.
    ///
    /// # Safety
    /// A non-null pointer must satisfy `reallocate`'s contract; its failure/zero-size rules apply.
    pub unsafe fn reallocate_ptr(&self, pointer: *mut T, count: usize) -> Option<NonNull<T>> {
        let Some(pointer) = NonNull::new(pointer) else {
            return self.allocate(count);
        };
        // SAFETY: the non-null branch retains the caller's original allocation provenance.
        unsafe { self.reallocate(pointer, count) }
    }
}
