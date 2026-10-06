use rustycad_tkernel::ncollection::{BoundingNode, BoundingVolume, ObjBnd};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
#[derive(Clone)]
pub struct PayloadTrace(Rc<TraceState>);
struct TraceState {
    events: RefCell<String>,
    reject: Cell<(char, char)>,
    object: Cell<usize>,
    bounds: Cell<usize>,
    stable: Cell<bool>,
}
thread_local! {
    // Native Default has no argument: a scoped per-thread fixture supplies its callback recorder.
    static DEFAULT_TRACE: RefCell<Option<PayloadTrace>> = const { RefCell::new(None) };
}
struct RestoreDefaultTrace(Option<PayloadTrace>);
impl Drop for RestoreDefaultTrace {
    fn drop(&mut self) {
        DEFAULT_TRACE.with(|slot| *slot.borrow_mut() = self.0.take());
    }
}
impl PayloadTrace {
    pub fn new() -> Self {
        Self(Rc::new(TraceState {
            events: RefCell::new(String::new()),
            reject: Cell::new(('\0', '\0')),
            object: Cell::new(0),
            bounds: Cell::new(0),
            stable: Cell::new(true),
        }))
    }
    pub fn item<const KIND: char>(&self, value: i32) -> Item<KIND> {
        Item {
            value,
            trace: self.clone(),
        }
    }
    pub fn clear(&self) {
        self.0.events.borrow_mut().clear();
        self.0.reject.set(('\0', '\0'));
        self.0.object.set(0);
        self.0.bounds.set(0);
        self.0.stable.set(true);
    }
    pub fn reject(&self, phase: char, kind: char) {
        self.0.reject.set((phase, kind));
    }
    pub fn events(&self) -> String {
        self.0.events.borrow().clone()
    }
    pub fn stable(&self) -> u8 {
        u8::from(self.0.stable.get())
    }
    pub fn record(&self, phase: char, kind: char, address: usize) {
        self.0
            .events
            .borrow_mut()
            .push_str(&format!("{phase}{kind},"));
        if phase == 'A' || phase == 'D' {
            self.observe(kind, address);
        }
        if self.0.reject.get() == (phase, kind) {
            self.0.reject.set(('\0', '\0'));
            panic!("payload {kind} phase {phase} rejected; expected permitted callback");
        }
    }
    fn observe(&self, kind: char, address: usize) {
        let expected = if kind == 'O' {
            self.0.object.get()
        } else {
            self.0.bounds.get()
        };
        if expected != 0 {
            self.0
                .stable
                .set(self.0.stable.get() && expected == address);
        }
    }
    pub fn watch_node(&self, node: &Node) {
        self.0.object.set(std::ptr::from_ref(node.object()).addr());
        self.0.bounds.set(std::ptr::from_ref(node.bounds()).addr());
    }
    pub fn watch_pair(&self, pair: &Pair) {
        self.0.object.set(std::ptr::from_ref(&pair.object).addr());
        self.0.bounds.set(std::ptr::from_ref(&pair.bounds).addr());
    }
    pub fn with_defaults<R>(&self, operation: impl FnOnce() -> R) -> R {
        let prior = DEFAULT_TRACE.with(|slot| slot.replace(Some(self.clone())));
        let _restore = RestoreDefaultTrace(prior);
        operation()
    }
}
pub struct Item<const KIND: char> {
    pub value: i32,
    trace: PayloadTrace,
}
impl<const KIND: char> Clone for Item<KIND> {
    fn clone(&self) -> Self {
        self.trace.record('C', KIND, 0);
        self.trace.item(self.value)
    }
    fn clone_from(&mut self, source: &Self) {
        self.value = source.value;
        self.trace
            .record('A', KIND, std::ptr::from_ref(self).addr());
    }
}
impl<const KIND: char> Default for Item<KIND> {
    fn default() -> Self {
        let trace = DEFAULT_TRACE
            .with(|slot| slot.borrow().clone())
            .expect("Default trace None; expected Some(PayloadTrace)");
        trace.record('N', KIND, 0);
        trace.item(0)
    }
}
impl<const KIND: char> Drop for Item<KIND> {
    fn drop(&mut self) {
        self.trace
            .record('D', KIND, std::ptr::from_ref(self).addr());
    }
}
impl BoundingVolume for Item<'B'> {
    fn add(&mut self, _: &Self) {}
    fn is_out(&self, _: &Self) -> bool {
        false
    }
    fn square_extent(&self) -> f64 {
        0.0
    }
}
pub type Node = BoundingNode<Item<'O'>, Item<'B'>>;
pub type Pair = ObjBnd<Item<'O'>, Item<'B'>>;
pub fn caught(operation: impl FnOnce()) -> bool {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation)).is_err()
}
