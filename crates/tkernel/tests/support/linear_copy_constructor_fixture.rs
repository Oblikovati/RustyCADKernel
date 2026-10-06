//! Shared native observations; this module contains no allocation recovery or kernel policy.
use super::LinearVector;
use std::cell::{Cell, RefCell};

#[derive(Default)]
pub struct CopyTrace {
    pub attempts: Cell<usize>,
    pub completed: Cell<usize>,
    pub failing: Cell<usize>,
    pub resources: Cell<usize>,
    pub active: Cell<bool>,
    pub events: RefCell<String>,
}

struct CopyPayload<'a> {
    value: i32,
    trace: &'a CopyTrace,
}
impl<'a> CopyPayload<'a> {
    fn new(value: i32, trace: &'a CopyTrace) -> Box<Self> {
        trace.resources.set(trace.resources.get() + 1);
        Box::new(Self { value, trace })
    }
}
impl Drop for CopyPayload<'_> {
    fn drop(&mut self) {
        self.trace.resources.set(self.trace.resources.get() - 1);
    }
}

pub struct CopyItem<'a> {
    pub value: i32,
    payload: Box<CopyPayload<'a>>,
}
impl<'a> CopyItem<'a> {
    pub fn new(value: i32, trace: &'a CopyTrace) -> Self {
        Self {
            value,
            payload: CopyPayload::new(value, trace),
        }
    }
}
impl Clone for CopyItem<'_> {
    fn clone(&self) -> Self {
        let trace = self.payload.trace;
        let payload = CopyPayload::new(self.payload.value, trace);
        trace
            .events
            .borrow_mut()
            .push_str(&format!("C{},", self.value));
        let attempt = trace.attempts.get() + 1;
        trace.attempts.set(attempt);
        if attempt == trace.failing.get() {
            std::panic::panic_any(attempt);
        }
        trace.completed.set(trace.completed.get() + 1);
        Self {
            value: self.value,
            payload,
        }
    }
}
impl Drop for CopyItem<'_> {
    fn drop(&mut self) {
        if self.payload.trace.active.get() {
            self.payload
                .trace
                .events
                .borrow_mut()
                .push_str(&format!("D{},", self.value));
        }
    }
}

pub struct CopySpec {
    pub name: &'static str,
    pub capacity: usize,
    pub count: usize,
    pub fail: usize,
    pub assigning: bool,
    pub receiver_count: usize,
}

pub const COPY_CASES: [CopySpec; 14] = [
    CopySpec {
        name: "clone-empty",
        capacity: 0,
        count: 0,
        fail: 0,
        assigning: false,
        receiver_count: 0,
    },
    CopySpec {
        name: "clone-reserved-empty",
        capacity: 8,
        count: 0,
        fail: 0,
        assigning: false,
        receiver_count: 0,
    },
    CopySpec {
        name: "clone-single",
        capacity: 8,
        count: 1,
        fail: 0,
        assigning: false,
        receiver_count: 0,
    },
    CopySpec {
        name: "clone-retained",
        capacity: 8,
        count: 3,
        fail: 0,
        assigning: false,
        receiver_count: 0,
    },
    CopySpec {
        name: "clone-first",
        capacity: 8,
        count: 3,
        fail: 1,
        assigning: false,
        receiver_count: 0,
    },
    CopySpec {
        name: "clone-middle",
        capacity: 8,
        count: 3,
        fail: 2,
        assigning: false,
        receiver_count: 0,
    },
    CopySpec {
        name: "clone-last",
        capacity: 8,
        count: 3,
        fail: 3,
        assigning: false,
        receiver_count: 0,
    },
    CopySpec {
        name: "clone-long-final",
        capacity: 8,
        count: 5,
        fail: 5,
        assigning: false,
        receiver_count: 0,
    },
    CopySpec {
        name: "assign-empty-target",
        capacity: 8,
        count: 3,
        fail: 0,
        assigning: true,
        receiver_count: 0,
    },
    CopySpec {
        name: "assign-occupied",
        capacity: 8,
        count: 3,
        fail: 0,
        assigning: true,
        receiver_count: 1,
    },
    CopySpec {
        name: "assign-first",
        capacity: 8,
        count: 3,
        fail: 1,
        assigning: true,
        receiver_count: 1,
    },
    CopySpec {
        name: "assign-middle",
        capacity: 8,
        count: 3,
        fail: 2,
        assigning: true,
        receiver_count: 1,
    },
    CopySpec {
        name: "assign-last",
        capacity: 8,
        count: 3,
        fail: 3,
        assigning: true,
        receiver_count: 1,
    },
    CopySpec {
        name: "assign-empty-first",
        capacity: 8,
        count: 1,
        fail: 1,
        assigning: true,
        receiver_count: 0,
    },
];

pub fn filled(
    capacity: usize,
    count: usize,
    start: i32,
    trace: &CopyTrace,
) -> LinearVector<CopyItem<'_>> {
    let mut vector = LinearVector::with_capacity(capacity);
    for index in 0..count {
        vector.append(CopyItem::new(start + i32::try_from(index).unwrap(), trace));
    }
    vector
}

pub fn row(
    phase: &str,
    spec: &CopySpec,
    source: &LinearVector<CopyItem<'_>>,
    result: Option<&LinearVector<CopyItem<'_>>>,
    before: *const CopyItem<'_>,
    trace: &CopyTrace,
    success: bool,
) -> String {
    let outcome = if success { "ok" } else { "throw" };
    let values = |vector: &LinearVector<CopyItem<'_>>| {
        vector
            .iter()
            .map(|item| format!("{},", item.value))
            .collect::<String>()
    };
    let (len, capacity, same, result_values) =
        result.map_or((0, 0, false, String::new()), |vector| {
            (
                vector.len(),
                vector.capacity(),
                before == vector.data_ptr(),
                values(vector),
            )
        });
    format!(
        "{phase}:{}:{outcome}:{len}:{capacity}:{}:{}:{}:{result_values}:{}",
        spec.name,
        u8::from(same),
        trace.events.borrow(),
        values(source),
        trace.resources.get()
    )
}
