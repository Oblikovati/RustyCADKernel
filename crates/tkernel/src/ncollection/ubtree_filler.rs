//! Fills a UBTree in a pseudo-random order, which keeps nearby objects from forming chains.
use super::{BaseAllocator, BoundingVolume, DynamicArray, UBTree};
mod balance;
mod mersenne;
use mersenne::MersenneTwister;

/// A pending object with its bound (native `ObjBnd`, public fields `myObj`/`myBnd`).
#[derive(Clone, Default)]
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

/// Collects objects, then adds them to the borrowed tree in `std::mt19937` order.
/// Items still pending when the filler is dropped are added then, as the native destructor does.
/// Example: `let mut filler = UBTreeFiller::new(&mut tree, None, true); filler.add(1, b); filler.fill();`
pub struct UBTreeFiller<'t, T: Clone + Default, B: BoundingVolume> {
    tree: &'t mut UBTree<T, B>,
    pending: DynamicArray<ObjBnd<T, B>>,
    generator: MersenneTwister,
    full_random: bool,
}
impl<'t, T: Clone + Default, B: BoundingVolume> UBTreeFiller<'t, T, B> {
    /// Fill `tree`; `allocator` backs the pending buffer (256-item blocks as natively) and
    /// `full_random` selects the better-balanced of the two native orders.
    /// Example: `UBTreeFiller::new(&mut tree, None, true)`.
    pub fn new(
        tree: &'t mut UBTree<T, B>,
        allocator: Option<BaseAllocator>,
        full_random: bool,
    ) -> Self {
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
    /// Add every queued object to the tree in random order and empty the queue.
    /// Returns the number of objects added; `filler.fill()`.
    pub fn fill(&mut self) -> i32 {
        let count: i32 = self.pending.length();
        for remaining in (1..=count).rev() {
            let index: i32 = self.draw_index(remaining);
            let item: ObjBnd<T, B> = self.pending.value(index).unwrap().clone();
            self.tree.add(item.object, item.bounds);
            // The drawn slot takes the last live item, shrinking the pool by one.
            let last: ObjBnd<T, B> = self.pending.value(remaining - 1).unwrap().clone();
            *self.pending.change_value(index).unwrap() = last;
        }
        self.pending.clear(false);
        count
    }
    /// Discard queued objects so dropping the filler adds nothing; `filler.reset()`.
    pub fn reset(&mut self) {
        self.pending.clear(false);
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
impl<T: Clone + Default, B: BoundingVolume> Drop for UBTreeFiller<'_, T, B> {
    fn drop(&mut self) {
        if !self.pending.is_empty() {
            self.fill();
        }
    }
}
