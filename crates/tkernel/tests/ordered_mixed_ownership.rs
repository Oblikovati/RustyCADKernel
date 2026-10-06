//! Each mixed native overload on insertion, replacement, try-hit and copy failure.
use rustycad_tkernel::ncollection::{BaseAllocator, HashPolicy, OrderedDataMap};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

struct Trace {
    events: RefCell<String>,
    active: Cell<bool>,
    fail: Cell<bool>,
    arena: BaseAllocator,
}
impl Trace {
    fn new() -> Rc<Self> {
        Rc::new(Self {
            events: RefCell::new(String::new()),
            active: Cell::new(false),
            fail: Cell::new(false),
            arena: BaseAllocator::incremental(4096),
        })
    }
    fn push(&self, event: &str) {
        if self.active.get() {
            self.events.borrow_mut().push_str(event);
        }
    }
    fn copying(&self, label: &str) {
        let allocated = u8::from(self.arena.reset(false).is_err());
        self.push(&format!("{label}{allocated},"));
        assert!(
            !self.fail.get(),
            "injected copy failure for {label}; expected successful copy"
        );
    }
}
struct Key(i32, Rc<Trace>);
impl Clone for Key {
    fn clone(&self) -> Self {
        self.1.copying("K");
        Self(self.0, Rc::clone(&self.1))
    }
}
struct Value(i32, Rc<Trace>);
impl Clone for Value {
    fn clone(&self) -> Self {
        self.1.copying("V");
        Self(self.0, Rc::clone(&self.1))
    }
    fn clone_from(&mut self, _: &Self) {
        panic!("expected copy construction of a temporary, not clone_from");
    }
}
struct Hasher;
impl HashPolicy<Key> for Hasher {
    fn hash(&self, key: &Key) -> usize {
        key.1.push("H,");
        key.0 as usize
    }
    fn equal(&self, left: &Key, right: &Key) -> bool {
        left.1.push("E,");
        left.0 == right.0
    }
}
type Map = OrderedDataMap<Key, Value, Hasher>;
fn dispatch(map: &mut Map, key: Key, value: Value, method: usize) -> i32 {
    match method {
        0 => i32::from(map.bind_key_ref(&key, value)),
        1 => map.bound_key_ref(&key, value).0,
        2 => i32::from(map.try_bind_key_ref(&key, value)),
        3 => map.try_bound_key_ref(&key, value).0,
        4 => i32::from(map.bind_value_ref(key, &value)),
        5 => map.bound_value_ref(key, &value).0,
        6 => i32::from(map.try_bind_value_ref(key, &value)),
        7 => map.try_bound_value_ref(key, &value).0,
        _ => unreachable!("method {method}; expected 0..8"),
    }
}
fn invoke(map: &mut Map, trace: &Rc<Trace>, method: usize, fail: bool) -> i32 {
    let key = Key(1, Rc::clone(trace));
    let value = Value(20, Rc::clone(trace));
    trace.active.set(true);
    trace.fail.set(fail);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        dispatch(map, key, value, method)
    }));
    trace.active.set(false);
    trace.fail.set(false);
    result.unwrap_or(-1)
}
fn scenario(method: usize, hit: bool, fail: bool) {
    let trace = Trace::new();
    let mut map = Map::with_allocator(Hasher, 101, trace.arena.clone());
    if hit {
        map.bind(Key(1, Rc::clone(&trace)), Value(10, Rc::clone(&trace)));
    }
    let query = Key(1, Rc::clone(&trace));
    let before = map.seek(&query).map(std::ptr::from_ref);
    let result = invoke(&mut map, &trace, method, fail);
    let observed = observed_binding(&map, &trace, result, before, hit);
    assert_native(method, hit, fail, &observed);
    map.clear(true);
    // Rust ownership reclaims failed construction; native IncAllocator retains its block.
    assert!(trace.arena.reset(false).is_ok());
}
fn observed_binding(
    map: &Map,
    trace: &Rc<Trace>,
    result: i32,
    before: Option<*const Value>,
    hit: bool,
) -> String {
    let after = map.seek(&Key(1, Rc::clone(trace)));
    let address_stable = !hit || before == after.map(std::ptr::from_ref);
    format!(
        "{result} {} {} {} {}",
        map.len(),
        after.map_or(-1, |v| v.0),
        u8::from(address_stable),
        trace.events.borrow()
    )
}
fn assert_native(method: usize, hit: bool, fail: bool, observed: &str) {
    let rows = include_str!("../../../tools/oracle/tkernel_ordered_mixed_ownership.expected");
    let prefix = format!("{method}_{}_{} ", u8::from(hit), u8::from(fail));
    let expected = rows.lines().find(|row| row.starts_with(&prefix)).unwrap();
    assert_eq!(format!("{prefix}{observed}"), expected);
}
macro_rules! mixed_test {
    ($name:ident, $method:literal) => {
        #[test]
        fn $name() {
            for hit in [false, true] {
                for fail in [false, true] {
                    scenario($method, hit, fail);
                }
            }
        }
    };
}
mixed_test!(bind_borrowed_key_owned_value, 0);
mixed_test!(bound_borrowed_key_owned_value, 1);
mixed_test!(try_bind_borrowed_key_owned_value, 2);
mixed_test!(try_bound_borrowed_key_owned_value, 3);
mixed_test!(bind_owned_key_borrowed_value, 4);
mixed_test!(bound_owned_key_borrowed_value, 5);
mixed_test!(try_bind_owned_key_borrowed_value, 6);
mixed_test!(try_bound_owned_key_borrowed_value, 7);
