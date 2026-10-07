use rustycad_tkernel::ncollection::{DoubleMap, HashPolicy};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
#[derive(Clone)]
pub struct CallbackTrace(pub Rc<TraceState>);
pub struct TraceState {
    pub events: RefCell<String>,
    pub recording: Cell<&'static str>,
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
            recording: Cell::new(""),
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
    pub fn map(&self) -> ObservedMap {
        DoubleMap::with_allocator(
            CallbackHash(self.clone()),
            CallbackHash(self.clone()),
            1,
            Default::default(),
        )
    }
    /// Starts recording the named callbacks, copy (C), hash (H) and destructor (D), from an
    /// empty trace; `trace.watch("C")` records copies only.
    pub fn watch(&self, events: &'static str) {
        self.0.recording.set(events);
        self.0.events.borrow_mut().clear();
    }
    pub fn stop(&self) {
        self.0.recording.set("");
    }
    pub fn events(&self) -> String {
        self.0.events.borrow().clone()
    }
    pub fn record(&self, event: char, kind: char) {
        if self.0.recording.get().contains(event) {
            self.0
                .events
                .borrow_mut()
                .push_str(&format!("{event}{kind},"));
        }
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
/// Everything public about the map: size, modulus, table identity, the first-key chains in
/// native order and the second-key index of every pair.
pub fn state(map: &ObservedMap) -> String {
    let mut text = format!("{} {} ", map.len(), map.bucket_count());
    let Some(table) = map.buckets() else {
        return text + "null";
    };
    text += &format!("{:p} ", table.identity());
    for bucket in 0..table.len() {
        for (first, second) in table.chain(bucket).unwrap() {
            let back = map.seek2(second).map(|key| key.value);
            text += &format!("{bucket}:{}>{}<{back:?},", first.value, second.value);
        }
    }
    text
}
/// True when the allocated first table holds no chain.
pub fn heads_empty(map: &ObservedMap) -> bool {
    let table = map.buckets().unwrap();
    (0..table.len()).all(|bucket| table.chain(bucket).unwrap().next().is_none())
}
pub fn caught(call: impl FnOnce()) -> bool {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(call)).is_err()
}
