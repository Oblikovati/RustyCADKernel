//! Native Erase moves between live slots before dropping the vacated tail.
use rustycad_tkernel::ncollection::LinearVector;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[derive(Default)]
struct EraseTrace {
    events: RefCell<String>,
    addresses: RefCell<Vec<*const EraseItem>>,
    active: Cell<bool>,
    fail_value: Cell<i32>,
}

impl EraseTrace {
    fn position(&self, item: &EraseItem) -> usize {
        self.addresses
            .borrow()
            .iter()
            .position(|&pointer| std::ptr::eq(pointer, item))
            .expect("destructor or callback: expected an original vector slot")
    }
}

struct EraseItem {
    value: i32,
    resource: Option<Box<i32>>,
    trace: Rc<EraseTrace>,
}

impl EraseItem {
    fn new(value: i32, trace: &Rc<EraseTrace>) -> Self {
        Self {
            value,
            resource: Some(Box::new(value)),
            trace: Rc::clone(trace),
        }
    }

    fn move_assign(target: &mut Self, source: &mut Self) {
        target.trace.events.borrow_mut().push_str(&format!(
            "A{}<-{}:{}<-{},",
            target.trace.position(target),
            target.trace.position(source),
            target.value,
            source.value
        ));
        target.value = source.value + 100;
        assert_ne!(
            source.value,
            target.trace.fail_value.get(),
            "assignment source {}: expected non-failing source",
            source.value
        );
        target.value = std::mem::replace(&mut source.value, -1);
        target.resource = source.resource.take();
    }
}

impl Drop for EraseItem {
    fn drop(&mut self) {
        if self.trace.active.get() {
            self.trace.events.borrow_mut().push_str(&format!(
                "D{}:{},",
                self.trace.position(self),
                self.value
            ));
        }
    }
}

struct EraseFixture {
    vector: LinearVector<EraseItem>,
    trace: Rc<EraseTrace>,
    resources: Vec<*const i32>,
    buffer: *const EraseItem,
}

impl EraseFixture {
    fn new(count: usize, fail_value: i32) -> Self {
        let trace = Rc::new(EraseTrace::default());
        let mut vector = LinearVector::with_capacity(8);
        for index in 0..count {
            vector.emplace_append(|| EraseItem::new(10 + i32::try_from(index).unwrap(), &trace));
        }
        let resources = vector.iter().map(resource_pointer).collect();
        *trace.addresses.borrow_mut() = vector.iter().map(std::ptr::from_ref).collect();
        trace.fail_value.set(fail_value);
        let buffer = vector.data_ptr();
        Self {
            vector,
            trace,
            resources,
            buffer,
        }
    }

    fn apply(&mut self, from: usize, to: usize, single: bool) -> &'static str {
        self.trace.active.set(true);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            if single {
                self.vector.erase_with_move(from, EraseItem::move_assign)
            } else {
                self.vector
                    .erase_range_with_move(from, to, EraseItem::move_assign)
            }
        }));
        self.trace.active.set(false);
        match result {
            Ok(Ok(())) => "ok",
            Ok(Err(_)) => "range",
            Err(_) => "throw",
        }
    }

    fn resource_id(&self, item: &EraseItem) -> String {
        let pointer = resource_pointer(item);
        self.resources
            .iter()
            .position(|&old| old == pointer)
            .map_or_else(|| "-1".to_owned(), |index| index.to_string())
    }

    fn row(&self, name: &str, outcome: &str) -> String {
        let values: String = self
            .vector
            .iter()
            .map(|item| format!("{},", item.value))
            .collect();
        let resources: String = self
            .vector
            .iter()
            .map(|item| format!("{},", self.resource_id(item)))
            .collect();
        format!(
            "{name}:{outcome}:{}:{}:{}:{}:{values}:{resources}",
            self.vector.len(),
            self.vector.capacity(),
            u8::from(self.buffer == self.vector.data_ptr()),
            self.trace.events.borrow()
        )
    }
}

fn resource_pointer(item: &EraseItem) -> *const i32 {
    item.resource
        .as_deref()
        .map_or(std::ptr::null(), std::ptr::from_ref)
}

fn erase_case(name: &str, count: usize, from: usize, to: usize, single: bool, fail: i32) {
    let mut fixture = EraseFixture::new(count, fail);
    let outcome = fixture.apply(from, to, single);
    let row = fixture.row(name, outcome);
    let expected = include_str!("../../../tools/oracle/tkernel_linear_erase.expected");
    assert!(
        expected.lines().any(|line| line == row),
        "missing native oracle row {row:?}"
    );
}

macro_rules! native_erase {
    ($test:ident, $row:literal, $count:expr, $from:expr, $to:expr, $single:expr, $fail:expr) => {
        #[test]
        fn $test() {
            erase_case($row, $count, $from, $to, $single, $fail);
        }
    };
}

native_erase!(single_head, "single-head", 5, 0, 0, true, -99);
native_erase!(single_middle, "single-middle", 5, 2, 0, true, -99);
native_erase!(single_tail, "single-tail", 5, 4, 0, true, -99);
native_erase!(range_overlap, "range-overlap", 5, 0, 2, false, -99);
native_erase!(range_disjoint, "range-disjoint", 5, 1, 4, false, -99);
native_erase!(range_tail, "range-tail", 5, 2, 5, false, -99);
native_erase!(range_all, "range-all", 5, 0, 5, false, -99);
native_erase!(empty_single, "empty-single", 0, 0, 0, true, -99);
native_erase!(empty_range, "empty-range", 0, 0, 1, false, -99);
native_erase!(past_end, "past-end", 5, 5, 0, true, -99);
native_erase!(empty_interval, "empty-interval", 5, 2, 2, false, -99);
native_erase!(reversed, "reversed", 5, 4, 1, false, -99);
native_erase!(range_past_end, "range-past-end", 5, 1, 6, false, -99);
native_erase!(throw_first, "throw-first", 5, 1, 0, true, 12);
native_erase!(throw_after_one, "throw-after-one", 5, 1, 0, true, 13);
native_erase!(throw_range, "throw-range", 5, 0, 2, false, 13);

#[test]
fn borrowed_values_survive_overlapping_assignment_without_trait_bounds() {
    let owner = String::from("borrowed");
    let mut vector = LinearVector::with_capacity(4);
    for _ in 0..3 {
        vector.append(owner.as_str());
    }
    vector
        .erase_with_move(0, |target, source| *target = std::mem::take(source))
        .unwrap();
    assert_eq!(vector.as_slice(), [owner.as_str(), owner.as_str()]);
}

#[test]
fn largest_invalid_indices_return_errors_without_overflow_or_callbacks() {
    let mut vector = LinearVector::<i32>::default();
    let callback = |_: &mut i32, _: &mut i32| panic!("invalid indices: expected no assignment");
    assert!(vector.erase_with_move(usize::MAX, callback).is_err());
    assert!(
        vector
            .erase_range_with_move(0, usize::MAX, callback)
            .is_err()
    );
    assert!(
        vector
            .erase_range_with_move(usize::MAX, 0, callback)
            .is_err()
    );
}
