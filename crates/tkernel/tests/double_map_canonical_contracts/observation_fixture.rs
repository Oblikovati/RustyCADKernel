use rustycad_tkernel::ncollection::{BaseAllocator, DoubleMap, DoubleMapObserver, HashPolicy};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
#[derive(Clone)]
pub struct CallbackTrace(pub Rc<TraceState>);
pub struct TraceState {
    pub events: RefCell<String>,
    pub mode: Cell<u8>,
    pub arena: BaseAllocator,
    pub observer: RefCell<Option<DoubleMapObserver>>,
    pub reject_clone: Cell<char>,
    pub reject_drop: Cell<char>,
    pub reject_hash: Cell<char>,
    pub reject_equal: Cell<bool>,
    pub hash_bias: Cell<usize>,
}
impl CallbackTrace {
    pub fn new() -> Self {
        Self(Rc::new(TraceState {
            events: RefCell::new(String::new()),
            mode: Cell::new(0),
            arena: BaseAllocator::incremental(512),
            observer: RefCell::new(None),
            reject_clone: Cell::new('\0'),
            reject_drop: Cell::new('\0'),
            reject_hash: Cell::new('\0'),
            reject_equal: Cell::new(false),
            hash_bias: Cell::new(0),
        }))
    }
    pub fn key(&self, kind: char, value: i32) -> CallbackKey {
        CallbackKey {
            kind,
            value,
            trace: self.clone(),
        }
    }
    pub fn map(&self, arena: BaseAllocator) -> ObservedMap {
        DoubleMap::with_allocator(
            CallbackHash(self.clone()),
            CallbackHash(self.clone()),
            1,
            arena,
        )
    }
    pub fn watch(&self, map: &ObservedMap, mode: u8) {
        *self.0.observer.borrow_mut() = Some(map.observer());
        self.0.mode.set(mode);
        self.0.events.borrow_mut().clear();
    }
    pub fn events(&self) -> String {
        self.0.events.borrow().clone()
    }
    pub fn live(&self) -> bool {
        self.0.arena.reset(false).is_err()
    }
    pub fn record(&self, event: char, kind: char) {
        let suffix = match self.0.mode.get() {
            1 if event == 'C' => u8::from(self.live()).to_string(),
            2 if event == 'D' || event == 'H' => {
                snapshot(&self.0.observer.borrow().as_ref().unwrap().snapshot())
            }
            3 => u8::from(self.live()).to_string(),
            _ => return,
        };
        self.0
            .events
            .borrow_mut()
            .push_str(&format!("{event}{kind}@{suffix},"));
    }
}
pub struct CallbackKey {
    pub kind: char,
    pub value: i32,
    pub trace: CallbackTrace,
}
impl std::fmt::Debug for CallbackKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}
impl Clone for CallbackKey {
    fn clone(&self) -> Self {
        self.trace.record('C', self.kind);
        assert_ne!(
            self.trace.0.reject_clone.get(),
            self.kind,
            "key {} copy rejected; expected permitted kind",
            self.kind
        );
        self.trace.key(self.kind, self.value)
    }
}
impl Drop for CallbackKey {
    fn drop(&mut self) {
        self.trace.record('D', self.kind);
        if self.trace.0.reject_drop.get() == self.kind {
            self.trace.0.reject_drop.set('\0');
            panic!("key {} drop rejected; expected permitted kind", self.kind);
        }
    }
}
#[derive(Clone)]
pub struct CallbackHash(pub CallbackTrace);
impl HashPolicy<CallbackKey> for CallbackHash {
    fn hash(&self, key: &CallbackKey) -> usize {
        self.0.record('H', key.kind);
        assert_ne!(
            self.0.0.reject_hash.get(),
            key.kind,
            "key {} hash rejected; expected permitted kind",
            key.kind
        );
        if key.kind == 'V' {
            self.0.0.hash_bias.get()
        } else {
            0
        }
    }
    fn equal(&self, first: &CallbackKey, second: &CallbackKey) -> bool {
        assert!(
            !self.0.0.reject_equal.get(),
            "key {} equality rejected; expected permitted comparison",
            second.value
        );
        first.value == second.value
    }
}
pub type ObservedMap = DoubleMap<CallbackKey, CallbackKey, CallbackHash, CallbackHash>;
pub fn snapshot(s: &rustycad_tkernel::ncollection::DoubleMapObservation) -> String {
    format!(
        "{}:{}:{}",
        s.size,
        u8::from(s.first_buckets.get(1).is_some_and(|p| *p != 0)),
        u8::from(s.second_buckets.get(1).is_some_and(|p| *p != 0))
    )
}
pub fn caught(call: impl FnOnce()) -> bool {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(call)).is_err()
}
