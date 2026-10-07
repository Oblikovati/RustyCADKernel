//! Native observation uses raw pointers only for slots confirmed live by token progress.
#![allow(unsafe_code)]
use super::{LinearVector, LinearVectorGrowth};
use std::cell::{Cell, RefCell};
#[derive(Default)]
pub struct DestructionTrace<'a> {
    pub active: Cell<bool>,
    pub resources: Cell<usize>,
    pub events: RefCell<String>,
    pub fail_drop: Cell<Option<usize>>,
    pub fail_repair: Cell<Option<i32>>,
    pub source: Cell<*const DestructionItem<'a>>,
    pub count: Cell<usize>,
    pub capacity: Cell<usize>,
    pub dead_end: Cell<usize>,
    pub repaired: Cell<usize>,
}
struct DestructionPayload<'a>(&'a DestructionTrace<'a>);
impl Drop for DestructionPayload<'_> {
    fn drop(&mut self) {
        self.0.resources.set(self.0.resources.get() - 1);
    }
}
pub struct DestructionItem<'a> {
    pub value: i32,
    payload: Option<Box<DestructionPayload<'a>>>,
    trace: &'a DestructionTrace<'a>,
}
impl<'a> DestructionItem<'a> {
    pub fn new(value: i32, trace: &'a DestructionTrace<'a>) -> Self {
        trace.resources.set(trace.resources.get() + 1);
        let payload = Some(Box::new(DestructionPayload(trace)));
        if trace.fail_repair.get() == Some(value) {
            std::panic::panic_any(value);
        }
        Self {
            value,
            payload,
            trace,
        }
    }
    pub fn relocate(&mut self, moving: bool) -> Self {
        let value = self.value;
        let trace = self.trace;
        let payload = if moving {
            self.value = -1;
            self.payload.take()
        } else {
            trace.resources.set(trace.resources.get() + 1);
            Some(Box::new(DestructionPayload(trace)))
        };
        trace
            .events
            .borrow_mut()
            .push_str(&format!("{}{value},", if moving { 'M' } else { 'C' }));
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
impl Drop for DestructionItem<'_> {
    fn drop(&mut self) {
        let trace = self.trace;
        if !trace.active.get() {
            return;
        }
        let index = (0..trace.count.get())
            .find(|&i| std::ptr::eq(std::ptr::from_ref(self), trace.source.get().wrapping_add(i)));
        let address = index.map_or_else(|| "X".to_string(), |i| i.to_string());
        // This snapshot describes the unpublished original during all traced destruction.
        // Native callback metadata is direct; Rust does not alias its exclusively borrowed token.
        trace.events.borrow_mut().push_str(&format!(
            "D{}@{address}({}/{}/1),",
            self.value,
            trace.count.get(),
            trace.capacity.get()
        ));
        if let Some(index) = index {
            trace.dead_end.set(index + 1);
            if trace.fail_drop.get() == Some(index) {
                std::panic::panic_any(index);
            }
        }
    }
}
pub struct DestructionSpec {
    pub name: &'static str,
    pub capacity: usize,
    pub count: usize,
    pub minimum: usize,
    pub moving: bool,
    pub fail: Option<usize>,
    pub repair_failure: Option<i32>,
}
pub const DESTRUCTION_CASES: &[DestructionSpec] = &[
    DestructionSpec {
        name: "empty-noop",
        capacity: 0,
        count: 0,
        minimum: 0,
        moving: false,
        fail: None,
        repair_failure: None,
    },
    DestructionSpec {
        name: "empty-grow",
        capacity: 0,
        count: 0,
        minimum: 1,
        moving: false,
        fail: None,
        repair_failure: None,
    },
    DestructionSpec {
        name: "retained-noop",
        capacity: 4,
        count: 3,
        minimum: 4,
        moving: false,
        fail: None,
        repair_failure: None,
    },
    DestructionSpec {
        name: "copy-success",
        capacity: 4,
        count: 3,
        minimum: 5,
        moving: false,
        fail: None,
        repair_failure: None,
    },
    DestructionSpec {
        name: "copy-first",
        capacity: 4,
        count: 3,
        minimum: 5,
        moving: false,
        fail: Some(0),
        repair_failure: None,
    },
    DestructionSpec {
        name: "copy-middle",
        capacity: 4,
        count: 3,
        minimum: 5,
        moving: false,
        fail: Some(1),
        repair_failure: None,
    },
    DestructionSpec {
        name: "copy-final",
        capacity: 4,
        count: 3,
        minimum: 5,
        moving: false,
        fail: Some(2),
        repair_failure: None,
    },
    DestructionSpec {
        name: "move-success",
        capacity: 4,
        count: 3,
        minimum: 5,
        moving: true,
        fail: None,
        repair_failure: None,
    },
    DestructionSpec {
        name: "move-first",
        capacity: 4,
        count: 3,
        minimum: 5,
        moving: true,
        fail: Some(0),
        repair_failure: None,
    },
    DestructionSpec {
        name: "move-middle",
        capacity: 4,
        count: 3,
        minimum: 5,
        moving: true,
        fail: Some(1),
        repair_failure: None,
    },
    DestructionSpec {
        name: "move-final",
        capacity: 4,
        count: 3,
        minimum: 5,
        moving: true,
        fail: Some(2),
        repair_failure: None,
    },
    DestructionSpec {
        name: "copy-repair-first",
        capacity: 4,
        count: 3,
        minimum: 5,
        moving: false,
        fail: Some(2),
        repair_failure: Some(100),
    },
    DestructionSpec {
        name: "copy-repair-late",
        capacity: 4,
        count: 3,
        minimum: 5,
        moving: false,
        fail: Some(2),
        repair_failure: Some(101),
    },
    DestructionSpec {
        name: "move-repair-first",
        capacity: 4,
        count: 3,
        minimum: 5,
        moving: true,
        fail: Some(2),
        repair_failure: Some(100),
    },
    DestructionSpec {
        name: "move-repair-late",
        capacity: 4,
        count: 3,
        minimum: 5,
        moving: true,
        fail: Some(2),
        repair_failure: Some(101),
    },
    DestructionSpec {
        name: "copy-repair-final",
        capacity: 4,
        count: 3,
        minimum: 5,
        moving: false,
        fail: Some(2),
        repair_failure: Some(102),
    },
    DestructionSpec {
        name: "move-repair-final",
        capacity: 4,
        count: 3,
        minimum: 5,
        moving: true,
        fail: Some(2),
        repair_failure: Some(102),
    },
];
pub fn filled<'a>(
    spec: &DestructionSpec,
    trace: &'a DestructionTrace<'a>,
) -> LinearVectorGrowth<DestructionItem<'a>> {
    let mut vector = LinearVector::with_capacity(spec.capacity);
    for i in 0..spec.count {
        vector.append(DestructionItem::new(10 + i32::try_from(i).unwrap(), trace));
    }
    trace.source.set(vector.data_ptr());
    trace.count.set(spec.count);
    trace.capacity.set(spec.capacity);
    trace.fail_drop.set(spec.fail);
    trace.fail_repair.set(spec.repair_failure);
    trace.active.set(true);
    vector.into_growth_token()
}
pub fn row<'a>(
    phase: &str,
    spec: &DestructionSpec,
    token: &LinearVectorGrowth<DestructionItem<'a>>,
    trace: &DestructionTrace<'a>,
    success: bool,
) -> String {
    let progress = token.progress();
    let mut values = String::new();
    for index in 0..progress.size {
        if progress.dead_slots.contains(&index) {
            values.push_str("X,");
            continue;
        }
        // SAFETY: the token's tracked liveness excludes all dead slots. Only shared access
        // occurs outside mutation/destruction; no reference escapes this observation.
        values.push_str(&unsafe { &*token.original_ptr().add(index) }.encoded());
    }
    format!(
        "{phase}:{}:{}:{}:{}:{}:{}:{}:{}:{values}:{}",
        spec.name,
        if success { "ok" } else { "throw" },
        progress.size,
        progress.capacity,
        u8::from(token.original_ptr() == trace.source.get()),
        trace.dead_end.get(),
        trace.repaired.get(),
        trace.events.borrow(),
        trace.resources.get()
    )
}
pub fn repaired_item<'a>(index: usize, trace: &'a DestructionTrace<'a>) -> DestructionItem<'a> {
    trace.events.borrow_mut().push_str(&format!("R{index},"));
    let item = DestructionItem::new(100 + i32::try_from(index).unwrap(), trace);
    trace.repaired.set(index + 1);
    item
}
