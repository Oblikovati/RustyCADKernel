//! Shared public and instrumented observations for native growth failure ownership.
use super::LinearVector;
use std::cell::{Cell, RefCell};
#[derive(Default)]
pub struct GrowthTrace {
    pub attempts: Cell<usize>,
    pub completed: Cell<usize>,
    pub failing: Cell<usize>,
    pub resources: Cell<usize>,
    pub active: Cell<bool>,
    pub events: RefCell<String>,
}
struct GrowthPayload<'a>(&'a GrowthTrace);
impl Drop for GrowthPayload<'_> {
    fn drop(&mut self) {
        self.0.resources.set(self.0.resources.get() - 1);
    }
}
pub struct GrowthItem<'a> {
    pub value: i32,
    payload: Option<Box<GrowthPayload<'a>>>,
    trace: &'a GrowthTrace,
}
impl<'a> GrowthItem<'a> {
    pub fn new(value: i32, trace: &'a GrowthTrace) -> Self {
        trace.resources.set(trace.resources.get() + 1);
        Self {
            value,
            payload: Some(Box::new(GrowthPayload(trace))),
            trace,
        }
    }
    pub fn relocate(&mut self, moving: bool) -> Self {
        let value = self.value;
        let payload = if moving {
            self.value = -1;
            self.payload.take()
        } else {
            self.trace.resources.set(self.trace.resources.get() + 1);
            Some(Box::new(GrowthPayload(self.trace)))
        };
        let trace = self.trace;
        let attempt = trace.attempts.get() + 1;
        trace
            .events
            .borrow_mut()
            .push_str(&format!("{}{value},", if moving { 'M' } else { 'C' }));
        trace.attempts.set(attempt);
        if attempt == trace.failing.get() {
            std::panic::panic_any(attempt);
        }
        trace.completed.set(trace.completed.get() + 1);
        Self {
            value,
            payload,
            trace,
        }
    }
    pub fn encoded(&self) -> String {
        format!("{}/{},", self.value, u8::from(self.payload.is_some()))
    }
}
impl Drop for GrowthItem<'_> {
    fn drop(&mut self) {
        if self.trace.active.get() {
            self.trace
                .events
                .borrow_mut()
                .push_str(&format!("D{},", self.value));
        }
    }
}
pub struct GrowthSpec {
    pub name: &'static str,
    pub capacity: usize,
    pub count: usize,
    pub minimum: usize,
    pub moving: bool,
    pub fail: usize,
    pub operation: char,
}
pub const GROWTH_CASES: &[GrowthSpec] = &[
    GrowthSpec {
        name: "empty-noop",
        capacity: 0,
        count: 0,
        minimum: 0,
        moving: false,
        fail: 0,
        operation: 'R',
    },
    GrowthSpec {
        name: "empty-grow",
        capacity: 0,
        count: 0,
        minimum: 1,
        moving: false,
        fail: 0,
        operation: 'R',
    },
    GrowthSpec {
        name: "retained-noop",
        capacity: 4,
        count: 3,
        minimum: 4,
        moving: false,
        fail: 0,
        operation: 'R',
    },
    GrowthSpec {
        name: "copy-double",
        capacity: 4,
        count: 3,
        minimum: 5,
        moving: false,
        fail: 0,
        operation: 'R',
    },
    GrowthSpec {
        name: "copy-minimum",
        capacity: 4,
        count: 3,
        minimum: 20,
        moving: false,
        fail: 0,
        operation: 'R',
    },
    GrowthSpec {
        name: "copy-fail-1",
        capacity: 4,
        count: 3,
        minimum: 5,
        moving: false,
        fail: 1,
        operation: 'R',
    },
    GrowthSpec {
        name: "copy-fail-2",
        capacity: 4,
        count: 3,
        minimum: 5,
        moving: false,
        fail: 2,
        operation: 'R',
    },
    GrowthSpec {
        name: "copy-fail-3",
        capacity: 4,
        count: 3,
        minimum: 5,
        moving: false,
        fail: 3,
        operation: 'R',
    },
    GrowthSpec {
        name: "copy-A-fail-1",
        capacity: 2,
        count: 2,
        minimum: 3,
        moving: false,
        fail: 1,
        operation: 'A',
    },
    GrowthSpec {
        name: "copy-A-fail-2",
        capacity: 2,
        count: 2,
        minimum: 3,
        moving: false,
        fail: 2,
        operation: 'A',
    },
    GrowthSpec {
        name: "copy-Z-fail-1",
        capacity: 2,
        count: 2,
        minimum: 5,
        moving: false,
        fail: 1,
        operation: 'Z',
    },
    GrowthSpec {
        name: "copy-Z-fail-2",
        capacity: 2,
        count: 2,
        minimum: 5,
        moving: false,
        fail: 2,
        operation: 'Z',
    },
    GrowthSpec {
        name: "copy-S-fail-1",
        capacity: 2,
        count: 2,
        minimum: 5,
        moving: false,
        fail: 1,
        operation: 'S',
    },
    GrowthSpec {
        name: "copy-S-fail-2",
        capacity: 2,
        count: 2,
        minimum: 5,
        moving: false,
        fail: 2,
        operation: 'S',
    },
    GrowthSpec {
        name: "move-double",
        capacity: 4,
        count: 3,
        minimum: 5,
        moving: true,
        fail: 0,
        operation: 'R',
    },
    GrowthSpec {
        name: "move-minimum",
        capacity: 4,
        count: 3,
        minimum: 20,
        moving: true,
        fail: 0,
        operation: 'R',
    },
    GrowthSpec {
        name: "move-fail-1",
        capacity: 4,
        count: 3,
        minimum: 5,
        moving: true,
        fail: 1,
        operation: 'R',
    },
    GrowthSpec {
        name: "move-fail-2",
        capacity: 4,
        count: 3,
        minimum: 5,
        moving: true,
        fail: 2,
        operation: 'R',
    },
    GrowthSpec {
        name: "move-fail-3",
        capacity: 4,
        count: 3,
        minimum: 5,
        moving: true,
        fail: 3,
        operation: 'R',
    },
    GrowthSpec {
        name: "move-A-fail-1",
        capacity: 2,
        count: 2,
        minimum: 3,
        moving: true,
        fail: 1,
        operation: 'A',
    },
    GrowthSpec {
        name: "move-A-fail-2",
        capacity: 2,
        count: 2,
        minimum: 3,
        moving: true,
        fail: 2,
        operation: 'A',
    },
    GrowthSpec {
        name: "move-Z-fail-1",
        capacity: 2,
        count: 2,
        minimum: 5,
        moving: true,
        fail: 1,
        operation: 'Z',
    },
    GrowthSpec {
        name: "move-Z-fail-2",
        capacity: 2,
        count: 2,
        minimum: 5,
        moving: true,
        fail: 2,
        operation: 'Z',
    },
    GrowthSpec {
        name: "move-S-fail-1",
        capacity: 2,
        count: 2,
        minimum: 5,
        moving: true,
        fail: 1,
        operation: 'S',
    },
    GrowthSpec {
        name: "move-S-fail-2",
        capacity: 2,
        count: 2,
        minimum: 5,
        moving: true,
        fail: 2,
        operation: 'S',
    },
];
pub fn filled<'a>(spec: &GrowthSpec, trace: &'a GrowthTrace) -> LinearVector<GrowthItem<'a>> {
    let mut vector = LinearVector::with_capacity(spec.capacity);
    for index in 0..spec.count {
        vector.append(GrowthItem::new(10 + i32::try_from(index).unwrap(), trace));
    }
    trace.failing.set(spec.fail);
    trace.active.set(true);
    vector
}
pub fn row(
    phase: &str,
    spec: &GrowthSpec,
    vector: &LinearVector<GrowthItem<'_>>,
    before: *const GrowthItem<'_>,
    trace: &GrowthTrace,
    success: bool,
) -> String {
    let values: String = vector.iter().map(GrowthItem::encoded).collect();
    format!(
        "{phase}:{}:{}:{}:{}:{}:{}:{values}:{}",
        spec.name,
        if success { "ok" } else { "throw" },
        vector.len(),
        vector.capacity(),
        u8::from(before == vector.data_ptr()),
        trace.events.borrow(),
        trace.resources.get()
    )
}
#[cfg(not(test))]
pub fn operation<'a>(
    spec: &GrowthSpec,
    vector: &mut LinearVector<GrowthItem<'a>>,
    trace: &'a GrowthTrace,
) {
    let relocate = |item: &mut GrowthItem<'a>| item.relocate(spec.moving);
    match spec.operation {
        'R' => vector.reserve_with_relocation(spec.minimum, relocate),
        'A' => {
            vector.emplace_append_with_relocation(relocate, || GrowthItem::new(99, trace));
        }
        'Z' => vector.resize_with_relocation(spec.minimum, relocate, || GrowthItem::new(99, trace)),
        'S' => {
            let donor = GrowthItem::new(99, trace);
            vector.set_value_with_policies(
                spec.minimum - 1,
                relocate,
                || GrowthItem::new(99, trace),
                |item| item.value = donor.value,
            );
        }
        _ => unreachable!(),
    }
}
