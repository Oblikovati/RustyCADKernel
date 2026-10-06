//! Raw BaseSequence topology for native subclasses which own their concrete node allocations.
use super::{BaseAllocator, SeqNode};
use crate::standard::Failure;
mod insertion;
mod iterator;
mod removal;
mod reorder;
pub use iterator::BaseSequenceRawIterator;

/// Native deletion callback type; `let delete: DelSeqNode = Sequence::<i32>::del_node`.
pub type DelSeqNode = unsafe fn(*mut SeqNode, &mut BaseAllocator);

/// Native BaseSequence metadata without an implicit node destructor.
/// Example: `let base = BaseSequence::new(None); assert!(base.is_empty())`.
/// Raw operations require exclusive, live, well-formed links. Drop never deletes nodes,
/// matching the native base destructor; subclasses must explicitly call clear_seq.
pub struct BaseSequence {
    /// Native protected allocator for derived-class composition.
    pub allocator: BaseAllocator,
    /// Native protected first link; raw operations require consistent metadata.
    pub first: *mut SeqNode,
    /// Native protected last link; raw operations require consistent metadata.
    pub last: *mut SeqNode,
    /// Native protected cached link; must agree with current_index.
    pub current: *mut SeqNode,
    /// Native protected one-based cached index.
    pub current_index: usize,
    /// Native protected sequence size.
    pub size: usize,
}
impl Default for BaseSequence {
    fn default() -> Self {
        Self::new(None)
    }
}
impl BaseSequence {
    /// Uses the common allocator when no resource is supplied; `BaseSequence::new(Some(arena))`.
    pub fn new(allocator: Option<BaseAllocator>) -> Self {
        Self {
            allocator: allocator.unwrap_or_default(),
            first: std::ptr::null_mut(),
            last: std::ptr::null_mut(),
            current: std::ptr::null_mut(),
            current_index: 0,
            size: 0,
        }
    }
    /// Returns the native Size; `assert_eq!(base.size(), 0)`.
    pub fn size(&self) -> usize {
        self.size
    }
    /// Returns the legacy signed Length cast; `assert_eq!(base.length(), 0)`.
    pub fn length(&self) -> i32 {
        self.size as i32
    }
    /// Tests the native size field; `base.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
    /// Borrows the allocator passed to deletion callbacks; `base.allocator()`.
    pub fn allocator(&self) -> &BaseAllocator {
        &self.allocator
    }
    /// Exposes the protected allocator handle to subclasses; `*base.change_allocator() = arena`.
    pub fn change_allocator(&mut self) -> &mut BaseAllocator {
        &mut self.allocator
    }
    /// Exposes the protected first node; `base.first_node()`.
    pub fn first_node(&self) -> *mut SeqNode {
        self.first
    }
    /// Exposes the protected last node; `base.last_node()`.
    pub fn last_node(&self) -> *mut SeqNode {
        self.last
    }
    /// Exposes the protected cached node; `base.current_node()`.
    pub fn current_node(&self) -> *mut SeqNode {
        self.current
    }
    /// Exposes the protected cached one-based index; `base.current_index()`.
    pub fn current_index(&self) -> usize {
        self.current_index
    }
    /// Sets the subclass cache as native typed Value/ChangeValue do; `unsafe { base.cache_position(2)? }`.
    /// # Safety
    /// All retained nodes must be live, correctly linked, and exclusively managed by this base.
    pub unsafe fn cache_position(&mut self, index: usize) -> Result<*mut SeqNode, Failure> {
        self.check_index(index)?;
        self.current = unsafe { self.find(index) };
        self.current_index = index;
        Ok(self.current)
    }
    /// Finds a one-based node using the native first/current/last cache heuristic; `unsafe { base.find(1) }`.
    /// # Safety
    /// Index must be in 1..=size, all links live, and the cached node/index consistent.
    pub unsafe fn find(&self, index: usize) -> *mut SeqNode {
        let (mut node, start) = self.search_origin(index);
        for _ in index..start {
            node = unsafe { (*node).previous() };
        }
        for _ in start..index {
            node = unsafe { (*node).next() };
        }
        node
    }
    fn search_origin(&self, index: usize) -> (*mut SeqNode, usize) {
        if index <= self.current_index {
            return if index < self.current_index / 2 {
                (self.first, 1)
            } else {
                (self.current, self.current_index)
            };
        }
        if index < self.current_index.wrapping_add(self.size) / 2 {
            return (self.current, self.current_index);
        }
        (self.last, self.size)
    }
    fn check_index(&self, index: usize) -> Result<(), Failure> {
        if index == 0 || index > self.size {
            return Err(Failure::out_of_range(&format!(
                "BaseSequence index {index}; expected 1..={}",
                self.size
            )));
        }
        Ok(())
    }
    fn nullify(&mut self) {
        self.first = std::ptr::null_mut();
        self.last = std::ptr::null_mut();
        self.current = std::ptr::null_mut();
        self.current_index = 0;
        self.size = 0;
    }
    fn cache_first(&mut self) {
        self.current = self.first;
        self.current_index = 1;
    }
}
