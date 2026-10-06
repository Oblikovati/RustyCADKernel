use rustycad_tkernel::ncollection::{
    BaseAllocator, HashPolicy, IndexedDataMap, IndexedMapObserver,
};
use std::{cell::RefCell, fmt::Write, rc::Rc};
struct Trace {
    arena: BaseAllocator,
    observer: Option<IndexedMapObserver>,
    events: String,
}
impl Trace {
    fn live(&self) -> i32 {
        self.arena
            .accumulating_chain()
            .iter()
            .map(|block| block.alloc_count)
            .sum()
    }
}
struct Item {
    kind: char,
    value: i32,
    tracked: bool,
    trace: Rc<RefCell<Trace>>,
}
impl std::fmt::Debug for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}
impl Clone for Item {
    fn clone(&self) -> Self {
        let mut trace = self.trace.borrow_mut();
        let live = trace.live();
        write!(trace.events, "C{}{live};", self.kind).unwrap();
        Self {
            kind: self.kind,
            value: self.value,
            tracked: true,
            trace: self.trace.clone(),
        }
    }
}
impl Drop for Item {
    fn drop(&mut self) {
        if !self.tracked {
            return;
        }
        let mut trace = self.trace.borrow_mut();
        let Some(observer) = &trace.observer else {
            return;
        };
        let state = observer.snapshot();
        let live = trace.live();
        let slots = state.indices.iter().filter(|p| **p != 0).count();
        let heads = state.buckets.iter().filter(|p| **p != 0).count();
        let old = u8::from(state.allocator == trace.arena.identity().addr());
        write!(
            trace.events,
            "D{}{},{},{slots},{heads},{live},{old};",
            self.kind, self.value, state.size
        )
        .unwrap();
    }
}
struct Policy;
impl HashPolicy<Item> for Policy {
    fn hash(&self, key: &Item) -> usize {
        key.value as usize % 2
    }
    fn equal(&self, a: &Item, b: &Item) -> bool {
        a.value == b.value
    }
}
fn setup() -> (IndexedDataMap<Item, Item, Policy>, Rc<RefCell<Trace>>) {
    let arena = BaseAllocator::accumulating(4096).unwrap();
    let map = IndexedDataMap::with_allocator(Policy, 1, arena.clone());
    let trace = Rc::new(RefCell::new(Trace {
        arena,
        observer: Some(map.observer()),
        events: String::new(),
    }));
    (map, trace)
}
fn fill(mode: usize, map: &mut IndexedDataMap<Item, Item, Policy>, trace: &Rc<RefCell<Trace>>) {
    for value in 1..=3 {
        let key = source_item('K', value, trace);
        let item = source_item('V', value, trace);
        trace.borrow_mut().events.clear();
        map.add_ref(&key, &item);
        let observed = trace.borrow();
        let label = format!("construct{mode}-{value}");
        super::native_row(
            &label,
            &format!("{label} {} {}", observed.events, observed.live()),
        );
    }
}
fn observed(mode: usize) {
    let (mut map, trace) = setup();
    fill(mode, &mut map, &trace);
    trace.borrow_mut().events.clear();
    match mode {
        0 => map.remove_last().unwrap(),
        1 => map.clear(false),
        _ => map.clear_with_allocator(Some(BaseAllocator::fresh_common())),
    }
    compare_observation(mode, &map, &trace.borrow());
    trace.borrow_mut().observer = None;
}
#[test]
fn native_remove_last_drops_key_then_value_before_free_and_decrement() {
    observed(0);
}
#[test]
fn native_clear_preserves_each_node_key_value_callback_phase() {
    observed(1);
}
#[test]
fn native_allocator_replacement_follows_all_key_value_drops() {
    observed(2);
}

fn source_item(kind: char, value: i32, trace: &Rc<RefCell<Trace>>) -> Item {
    Item {
        kind,
        value,
        tracked: false,
        trace: trace.clone(),
    }
}
fn compare_observation(mode: usize, map: &IndexedDataMap<Item, Item, Policy>, trace: &Trace) {
    let old = u8::from(map.allocator().identity().addr() == trace.arena.identity().addr());
    let label = format!("observer{mode}");
    let row = format!(
        "{label} {} {} {} {old}",
        trace.events,
        map.len(),
        trace.live()
    );
    super::native_row(&label, &row);
}
