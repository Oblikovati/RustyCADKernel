//! A genuine move constructor transfers ownership without requiring a receiver or item traits.
use rustycad_tkernel::ncollection::{BaseAllocator, DynamicArray};
use std::{cell::Cell, fmt::Write, rc::Rc};
#[derive(Default)]
struct LifetimeProbe {
    constructed: Cell<usize>,
    destroyed: Cell<usize>,
}
struct MoveItem {
    value: i32,
    probe: Rc<LifetimeProbe>,
}
impl MoveItem {
    fn new(value: i32, probe: &Rc<LifetimeProbe>) -> Self {
        probe.constructed.set(probe.constructed.get() + 1);
        Self {
            value,
            probe: Rc::clone(probe),
        }
    }
}
impl Drop for MoveItem {
    fn drop(&mut self) {
        self.probe.destroyed.set(self.probe.destroyed.get() + 1);
    }
}
struct MoveFixture {
    source: DynamicArray<MoveItem>,
    allocator: Option<BaseAllocator>,
    probe: Rc<LifetimeProbe>,
}
impl MoveFixture {
    fn new(increment: i32, count: i32, reserve: usize, retain: bool, custom: bool) -> Self {
        let probe = Rc::new(LifetimeProbe::default());
        let allocator = custom.then(BaseAllocator::common);
        let mut source = prepared_source(increment, count, reserve, &allocator, &probe);
        if retain {
            source.clear(false);
        }
        probe.constructed.set(0);
        probe.destroyed.set(0);
        Self {
            source,
            allocator,
            probe,
        }
    }
    fn move_row(&mut self, name: &str) -> String {
        let first = self.source.storage_extension().block_at(0).ok();
        let mut moved = DynamicArray::from_moved(&mut self.source);
        let stable = first.is_none() || first == moved.storage_extension().block_at(0).ok();
        let mut row = self.destination_row(name, &mut moved);
        row += &self.source_row(stable);
        self.source
            .emplace_append(|| MoveItem::new(88, &self.probe));
        row += &self.reuse_row(&moved);
        let count = moved.size();
        drop(moved);
        assert_eq!(self.probe.destroyed.get(), count);
        row
    }
    fn destination_row(&self, name: &str, moved: &mut DynamicArray<MoveItem>) -> String {
        format!(
            "{name} size={} blocks={} available={} allocator={}",
            moved.size(),
            moved.storage_extension().block_count(),
            moved.storage_extension().available_size(),
            usize::from(moved.allocator() == self.allocator.as_ref())
        )
    }
    fn source_row(&mut self, stable: bool) -> String {
        format!(
            " source={},{},{} stable={} operations={},{}",
            self.source.size(),
            self.source.storage_extension().block_count(),
            usize::from(self.source.allocator().is_none()),
            usize::from(stable),
            self.probe.constructed.get(),
            self.probe.destroyed.get()
        )
    }
    fn reuse_row(&mut self, moved: &DynamicArray<MoveItem>) -> String {
        let mut row = format!(
            " reused={},{} values=",
            self.source.storage_extension().available_size(),
            self.source[0].value
        );
        for value in moved.iter() {
            write!(row, "{},", value.value).unwrap();
        }
        row
    }
}
fn prepared_source(
    increment: i32,
    count: i32,
    reserve: usize,
    allocator: &Option<BaseAllocator>,
    probe: &Rc<LifetimeProbe>,
) -> DynamicArray<MoveItem> {
    let mut source = DynamicArray::with_allocator_i32(increment, allocator.clone());
    for value in 0..count {
        source.emplace_append(|| MoveItem::new(value + 10, probe));
    }
    source
        .storage_extension()
        .ensure_block_count(reserve, size_of::<MoveItem>())
        .unwrap();
    source
}
macro_rules! native_move {
    ($name:ident, $increment:expr, $count:expr, $reserve:expr, $retain:expr, $custom:expr) => {
        #[test]
        fn $name() {
            let mut fixture = MoveFixture::new($increment, $count, $reserve, $retain, $custom);
            let row = fixture.move_row(stringify!($name));
            let expected =
                include_str!("../../../tools/oracle/tkernel_dynamic_move_constructor.expected")
                    .lines()
                    .find(|line| line.starts_with(concat!(stringify!($name), " ")))
                    .unwrap();
            assert_eq!(row, expected);
        }
    };
}
native_move!(empty_default, 256, 0, 0, false, false);
native_move!(empty_custom, 3, 0, 0, false, true);
native_move!(one, 1, 1, 1, false, true);
native_move!(multiple, 3, 5, 3, false, true);
native_move!(retained, 3, 5, 3, true, true);
native_move!(reserved_empty, 3, 0, 3, false, false);
native_move!(zero_increment, 0, 2, 3, false, false);
native_move!(negative_increment, -7, 2, 3, false, true);
#[test]
fn move_constructor_transfers_initialized_tail_and_does_not_touch_payload() {
    let mut source = DynamicArray::<i32>::with_increment(4);
    source.append(10);
    source[3] = 77;
    let mut moved = DynamicArray::from_moved(&mut source);
    assert_eq!(moved.size(), 1);
    assert_eq!(moved.value(3), Ok(&77));
    assert_moved_tail_survives_panic(&mut moved);
    moved.emplace_value(3, || 99).unwrap();
    assert_eq!(moved.iter().copied().collect::<Vec<_>>(), [10, 0, 0, 99]);
    assert_eq!(source.size(), 0);
    source.append(7);
    assert_eq!(source.storage_extension().available_size(), 4);
}

fn assert_moved_tail_survives_panic(moved: &mut DynamicArray<i32>) {
    let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        moved
            .emplace_value(3, || {
                panic!("index 3: expected injected constructor failure")
            })
            .unwrap();
    }));
    assert!(failure.is_err());
    assert_eq!(moved.size(), 1);
    assert_eq!(moved.value(3), Ok(&77));
}
