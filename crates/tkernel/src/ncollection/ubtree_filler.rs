//! Fills a UBTree in a pseudo-random order, which keeps nearby objects from forming chains.
use super::{BaseAllocator, BoundingVolume, DynamicArray, UBTree};
mod balance;
mod insertion;
use crate::stl::MersenneTwister;
pub use insertion::BoundingTreeInsertion;

/// A pending object with its bound (native `ObjBnd`, public fields `myObj`/`myBnd`).
#[derive(Default)]
pub struct ObjBnd<T, B> {
    /// The object to insert.
    pub object: T,
    /// Its bounding geometry.
    pub bounds: B,
}
impl<T, B> ObjBnd<T, B> {
    /// Pair an object with its bound; `ObjBnd::new(1, bounds)`.
    pub fn new(object: T, bounds: B) -> Self {
        Self { object, bounds }
    }
}
impl<T: Clone, B: Clone> ObjBnd<T, B> {
    /// Copies the object before its bounds, as native ObjBnd does; `ObjBnd::new_ref(&1, &bounds)`.
    pub fn new_ref(object: &T, bounds: &B) -> Self {
        Self {
            object: object.clone(),
            bounds: bounds.clone(),
        }
    }
}
impl<T: Clone, B: Clone> Clone for ObjBnd<T, B> {
    fn clone(&self) -> Self {
        Self::new_ref(&self.object, &self.bounds)
    }
    fn clone_from(&mut self, source: &Self) {
        self.object.clone_from(&source.object);
        self.bounds.clone_from(&source.bounds);
    }
}

/// Collects objects, then adds them to the borrowed tree in `std::mt19937` order.
/// Items still pending when the filler is dropped are added then, as the native destructor does.
/// Example: `let mut filler = UBTreeFiller::new(&mut tree, None, true); filler.add(1, b); filler.fill();`
pub struct UBTreeFiller<
    't,
    T: Clone + Default,
    B: BoundingVolume,
    Tree: BoundingTreeInsertion<T, B> + ?Sized = UBTree<T, B>,
> {
    tree: &'t mut Tree,
    pending: DynamicArray<ObjBnd<T, B>>,
    generator: MersenneTwister,
    full_random: bool,
}
impl<'t, T: Clone + Default, B: BoundingVolume, Tree: BoundingTreeInsertion<T, B> + ?Sized>
    UBTreeFiller<'t, T, B, Tree>
{
    /// Fill `tree`; `allocator` backs the pending buffer (256-item blocks as natively) and
    /// `full_random` selects the better-balanced of the two native orders.
    /// Example: `UBTreeFiller::new(&mut tree, None, true)`.
    pub fn new(tree: &'t mut Tree, allocator: Option<BaseAllocator>, full_random: bool) -> Self {
        Self {
            tree,
            pending: DynamicArray::with_allocator(256, allocator),
            // 5489 is std::mt19937::default_seed, spelled out natively for old compilers.
            generator: MersenneTwister::new(5489),
            full_random,
        }
    }
    /// Queue an object for the next `fill`; `filler.add(1, bounds)`.
    pub fn add(&mut self, object: T, bounds: B) {
        self.pending.append(ObjBnd::new(object, bounds));
    }
    /// Queues borrowed arguments using native object/bound copy order; `filler.add_ref(&1, &bounds)`.
    pub fn add_ref(&mut self, object: &T, bounds: &B) {
        self.pending.append(ObjBnd::new_ref(object, bounds));
    }
    /// Add every queued object to the tree in random order and empty the queue.
    /// Returns the number of objects added; `filler.fill()`.
    pub fn fill(&mut self) -> i32 {
        let count: i32 = self.pending.length();
        for remaining in (1..=count).rev() {
            let index: i32 = self.draw_index(remaining);
            let item = self.pending.value(index).unwrap();
            self.tree.insert_bound_ref(&item.object, &item.bounds);
            // The drawn slot takes the last live item, shrinking the pool by one.
            self.assign_pending_slot(index, remaining - 1);
        }
        self.pending.clear(false);
        count
    }
    /// Discard queued objects so dropping the filler adds nothing; `filler.reset()`.
    pub fn reset(&mut self) {
        self.pending.clear(false);
    }
    fn assign_pending_slot(&mut self, selected: i32, last: i32) {
        if selected == last {
            return;
        }
        let mut entries = self.pending.iter_mut();
        let target = entries.nth(selected as usize).unwrap();
        let source = entries.nth((last - selected - 1) as usize).unwrap();
        target.clone_from(source);
    }
    fn draw_index(&mut self, remaining: i32) -> i32 {
        // `static_cast<unsigned int>(myRandGen()) % i` promotes `i` to unsigned.
        let draw: i32 = (self.generator.next_u32() % remaining as u32) as i32;
        if self.full_random {
            draw
        } else {
            remaining - draw - 1
        }
    }
}
impl<T: Clone + Default, B: BoundingVolume, Tree: BoundingTreeInsertion<T, B> + ?Sized> Drop
    for UBTreeFiller<'_, T, B, Tree>
{
    fn drop(&mut self) {
        if !self.pending.is_empty() {
            self.fill();
        }
    }
}
