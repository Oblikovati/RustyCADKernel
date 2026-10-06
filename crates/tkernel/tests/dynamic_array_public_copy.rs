//! Public copy policies expose native pre-recovery construction/assignment observations.
use rustycad_tkernel::ncollection::{
    BaseAllocator, DynamicArray, DynamicArrayCopyConstructionFailure,
};
use rustycad_tkernel::standard::Failure;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
mod dynamic_array_public_copy {
    pub mod contracts;
    pub mod oracle;
    pub mod trivial;
}
struct CopyWorld {
    source: BaseAllocator,
    target: BaseAllocator,
    events: RefCell<String>,
    attempts: Cell<usize>,
    fail_at: usize,
    live: Cell<usize>,
}
impl CopyWorld {
    fn new(fail_at: usize) -> Rc<Self> {
        Rc::new(Self {
            source: BaseAllocator::accumulating(4096).unwrap(),
            target: BaseAllocator::accumulating(4096).unwrap(),
            events: RefCell::new(String::new()),
            attempts: Cell::new(0),
            fail_at,
            live: Cell::new(0),
        })
    }
    fn event(&self, kind: char, value: i32) {
        self.events.borrow_mut().push_str(&format!(
            "{kind}{value}@{},{};",
            live_allocations(&self.source),
            live_allocations(&self.target)
        ));
    }
}
struct PublicCopyItem {
    value: i32,
    world: Rc<CopyWorld>,
}
impl PublicCopyItem {
    fn new(value: i32, world: &Rc<CopyWorld>) -> Self {
        world.live.set(world.live.get() + 1);
        Self {
            value,
            world: Rc::clone(world),
        }
    }
    fn construct_copy(&self) -> Result<Self, Failure> {
        self.world.event('C', self.value);
        let attempt = self.world.attempts.get() + 1;
        self.world.attempts.set(attempt);
        if attempt == self.world.fail_at {
            return Err(Failure::with_type("Standard_Failure", "copy failure", ""));
        }
        Ok(Self::new(self.value, &self.world))
    }
}
impl Drop for PublicCopyItem {
    fn drop(&mut self) {
        self.world.event('X', self.value);
        self.world.live.set(self.world.live.get() - 1);
    }
}
struct CopyFixture {
    source: DynamicArray<PublicCopyItem>,
    target: DynamicArray<PublicCopyItem>,
    world: Rc<CopyWorld>,
}
impl CopyFixture {
    fn new(failure: usize) -> Self {
        let world = CopyWorld::new(failure);
        let mut source = DynamicArray::with_allocator(2, Some(world.source.clone()));
        let mut target = DynamicArray::with_allocator(4, Some(world.target.clone()));
        fill_items(&mut source, 1..=3, &world);
        source
            .storage_extension()
            .ensure_block_count(3, size_of::<PublicCopyItem>())
            .unwrap();
        fill_items(&mut target, 90..=92, &world);
        Self {
            source,
            target,
            world,
        }
    }
}
fn live_allocations(allocator: &BaseAllocator) -> usize {
    allocator
        .accumulating_chain()
        .iter()
        .map(|block| block.alloc_count as usize)
        .sum()
}

fn fill_items(
    array: &mut DynamicArray<PublicCopyItem>,
    values: std::ops::RangeInclusive<i32>,
    world: &Rc<CopyWorld>,
) {
    for value in values {
        array.append(PublicCopyItem::new(value, world));
    }
}
