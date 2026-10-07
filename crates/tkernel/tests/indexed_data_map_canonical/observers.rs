//! Copy and destructor order of IndexedDataMap against the native rows. The probe also printed,
//! inside each callback, the map metadata and the arena's live allocation count; those are
//! protected native state (plan D1 class B6) that slot storage does not model, so they are
//! stripped from the native row and the remaining order and final state are compared.
use rustycad_tkernel::ncollection::{BaseAllocator, HashPolicy, IndexedDataMap};
use std::{cell::RefCell, fmt::Write, rc::Rc};
struct Item {
    kind: char,
    value: i32,
    tracked: bool,
    events: Rc<RefCell<String>>,
}
impl std::fmt::Debug for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}
impl Clone for Item {
    fn clone(&self) -> Self {
        write!(self.events.borrow_mut(), "C{};", self.kind).unwrap();
        Self {
            kind: self.kind,
            value: self.value,
            tracked: true,
            events: self.events.clone(),
        }
    }
}
impl Drop for Item {
    fn drop(&mut self) {
        if self.tracked {
            write!(self.events.borrow_mut(), "D{}{};", self.kind, self.value).unwrap();
        }
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
type ItemMap = IndexedDataMap<Item, Item, Policy>;
fn source_item(kind: char, value: i32, events: &Rc<RefCell<String>>) -> Item {
    Item {
        kind,
        value,
        tracked: false,
        events: events.clone(),
    }
}
/// The native row without its B6 fields: copy events lose the live count after the kind,
/// destructor events keep kind and value, and of the trailing fields only `Extent()` and the
/// allocator identity flag remain.
fn native_order(label: &str) -> String {
    let row = super::native_row_text(label);
    let fields: Vec<&str> = row.split(' ').collect();
    let events: String = fields[1]
        .split_terminator(';')
        .map(|event| event.split(',').next().unwrap())
        .map(|event| match event.starts_with('C') {
            true => {
                event
                    .trim_end_matches(|c: char| c.is_ascii_digit())
                    .to_owned()
                    + ";"
            }
            false => event.to_owned() + ";",
        })
        .collect();
    match fields.len() {
        3 => format!("{label} {events}"),
        _ => format!("{label} {events} {} {}", fields[2], fields[4]),
    }
}
fn fill(mode: usize, map: &mut ItemMap, events: &Rc<RefCell<String>>) {
    for value in 1..=3 {
        let key = source_item('K', value, events);
        let item = source_item('V', value, events);
        events.borrow_mut().clear();
        map.add_ref(&key, &item);
        let label = format!("construct{mode}-{value}");
        assert_eq!(format!("{label} {}", events.borrow()), native_order(&label));
    }
}
fn observed(mode: usize) {
    let arena = BaseAllocator::accumulating(4096).unwrap();
    let mut map = ItemMap::with_allocator(Policy, 1, arena.clone());
    let events = Rc::new(RefCell::new(String::new()));
    fill(mode, &mut map, &events);
    events.borrow_mut().clear();
    match mode {
        0 => map.remove_last().unwrap(),
        1 => map.clear(false),
        _ => map.clear_with_allocator(Some(BaseAllocator::fresh_common())),
    }
    let old = u8::from(map.allocator() == &arena);
    let label = format!("observer{mode}");
    let row = format!("{label} {} {} {old}", events.borrow(), map.len());
    assert_eq!(row, native_order(&label));
}
#[test]
fn native_remove_last_drops_key_then_value() {
    observed(0);
}
#[test]
fn native_clear_drops_each_key_then_value_in_bucket_order() {
    observed(1);
}
#[test]
fn native_allocator_replacement_follows_all_key_value_drops() {
    observed(2);
}
