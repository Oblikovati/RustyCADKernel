//! Native Remove/UnBind and bucket-chain destruction, including recycled slots.
use rustycad_tkernel::ncollection::{HashPolicy, OrderedDataMap, OrderedMap};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

#[derive(Debug, Default)]
struct Trace {
    events: RefCell<String>,
    active: Cell<bool>,
    fail: Cell<bool>,
}
impl Trace {
    fn push(&self, event: String) {
        if self.active.get() {
            self.events.borrow_mut().push_str(&format!("{event},"));
        }
    }
}
#[derive(Debug)]
struct Key(i32, Rc<Trace>);
impl Drop for Key {
    fn drop(&mut self) {
        self.1.push(format!("K{}", self.0));
    }
}
struct Value(i32, Rc<Trace>);
impl Drop for Value {
    fn drop(&mut self) {
        self.1.push(format!("V{}", self.0));
    }
}
struct Hasher(Rc<Trace>);
impl HashPolicy<Key> for Hasher {
    fn hash(&self, key: &Key) -> usize {
        self.0.push(format!("H{}", key.0));
        assert!(
            !self.0.fail.get(),
            "injected hash failure for key {}",
            key.0
        );
        key.0 as usize % 2
    }
    fn equal(&self, left: &Key, right: &Key) -> bool {
        self.0.push(format!("E{}:{}", left.0, right.0));
        left.0 == right.0
    }
}
trait OrderedFixture {
    fn fixture(trace: &Rc<Trace>) -> Self;
    fn insert(&mut self, number: i32, trace: &Rc<Trace>);
    fn remove(&mut self, query: &Key) -> bool;
    fn len(&self) -> usize;
    fn clear(&mut self, release: bool);
    fn resize(&mut self, buckets: usize);
    fn keys(&self) -> Vec<i32>;
}
type Set = OrderedMap<Key, Hasher>;
type Map = OrderedDataMap<Key, Value, Hasher>;
impl OrderedFixture for Set {
    fn fixture(trace: &Rc<Trace>) -> Self {
        Self::with_hasher(Hasher(Rc::clone(trace)), 101)
    }
    fn insert(&mut self, number: i32, trace: &Rc<Trace>) {
        self.add(Key(number, Rc::clone(trace)));
    }
    fn remove(&mut self, query: &Key) -> bool {
        Self::remove(self, query)
    }
    fn len(&self) -> usize {
        Self::len(self)
    }
    fn clear(&mut self, release: bool) {
        Self::clear(self, release);
    }
    fn resize(&mut self, buckets: usize) {
        Self::resize(self, buckets);
    }
    fn keys(&self) -> Vec<i32> {
        self.iter().map(|key| key.0).collect()
    }
}
impl OrderedFixture for Map {
    fn fixture(trace: &Rc<Trace>) -> Self {
        Self::with_hasher(Hasher(Rc::clone(trace)), 101)
    }
    fn insert(&mut self, number: i32, trace: &Rc<Trace>) {
        self.bind(
            Key(number, Rc::clone(trace)),
            Value(number, Rc::clone(trace)),
        );
    }
    fn remove(&mut self, query: &Key) -> bool {
        self.unbind(query)
    }
    fn len(&self) -> usize {
        Self::len(self)
    }
    fn clear(&mut self, release: bool) {
        Self::clear(self, release);
    }
    fn resize(&mut self, buckets: usize) {
        Self::resize(self, buckets);
    }
    fn keys(&self) -> Vec<i32> {
        self.items().map(|(key, _)| key.0).collect()
    }
}
fn populate<T: OrderedFixture>(map: &mut T, trace: &Rc<Trace>) {
    for number in [4, 1, 2, 3, 6, 5] {
        map.insert(number, trace);
    }
    assert!(map.remove(&Key(1, Rc::clone(trace))));
    map.insert(9, trace);
    assert_eq!(map.keys(), [4, 2, 3, 6, 5, 9]);
}
fn removal<T: OrderedFixture>(name: &str, number: i32, empty: bool, fail: bool) {
    let trace = Rc::new(Trace::default());
    let mut map = T::fixture(&trace);
    if !empty {
        populate(&mut map, &trace);
    }
    let query = Key(number, Rc::clone(&trace));
    let result = remove_traced(&mut map, &query, &trace, fail);
    let observed = format!(
        "{} {} {}",
        u8::from(result),
        map.len(),
        trace.events.borrow()
    );
    assert_oracle(name, &observed);
    if fail {
        assert_eq!(map.keys(), [4, 2, 3, 6, 5, 9]);
        assert!(map.remove(&query));
    }
}
fn remove_traced<T: OrderedFixture>(map: &mut T, query: &Key, trace: &Trace, fail: bool) -> bool {
    trace.active.set(true);
    trace.fail.set(fail);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| map.remove(query)));
    trace.active.set(false);
    trace.fail.set(false);
    assert_eq!(result.is_err(), fail);
    result.unwrap_or(true)
}

fn destruction<T: OrderedFixture>(name: &str, mode: u8) {
    let trace = Rc::new(Trace::default());
    {
        let mut map = T::fixture(&trace);
        populate(&mut map, &trace);
        if mode == 3 {
            map.resize(200);
        }
        trace.active.set(true);
        if mode != 2 {
            map.clear(mode == 1);
        }
    }
    trace.active.set(false);
    assert_oracle(name, &trace.events.borrow());
}
fn assert_oracle(name: &str, result: &str) {
    let expected = include_str!("../../../tools/oracle/tkernel_ordered_map_destruction.expected");
    let prefix = format!("{name} ");
    let row = expected
        .lines()
        .find(|line| line.starts_with(&prefix))
        .unwrap();
    assert_eq!(format!("{prefix}{result}"), row);
}
macro_rules! removal_test {
    ($test:ident, $map:ty, $number:literal, $empty:literal, $fail:literal) => {
        #[test]
        fn $test() {
            removal::<$map>(stringify!($test), $number, $empty, $fail);
        }
    };
}
macro_rules! destruction_test {
    ($test:ident, $map:ty, $mode:literal) => {
        #[test]
        fn $test() {
            destruction::<$map>(stringify!($test), $mode);
        }
    };
}
removal_test!(set_hit, Set, 3, false, false);
removal_test!(set_miss, Set, 7, false, false);
removal_test!(set_empty, Set, 3, true, false);
removal_test!(set_throw, Set, 3, false, true);
removal_test!(map_hit, Map, 3, false, false);
removal_test!(map_miss, Map, 7, false, false);
removal_test!(map_empty, Map, 3, true, false);
removal_test!(map_throw, Map, 3, false, true);
destruction_test!(set_clear, Set, 0);
destruction_test!(set_release, Set, 1);
destruction_test!(set_drop, Set, 2);
destruction_test!(set_resized, Set, 3);
destruction_test!(map_clear, Map, 0);
destruction_test!(map_release, Map, 1);
destruction_test!(map_drop, Map, 2);
destruction_test!(map_resized, Map, 3);
