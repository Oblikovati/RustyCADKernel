use rustycad_tkernel::ncollection::{
    BaseMapStorage, BasicMapIteratorSource, BasicMapNode, BasicMapNodeMut, MapLinkNode,
};
use std::{cell::Cell, rc::Rc};

#[derive(Default)]
pub(super) struct Calls {
    pub reads: Cell<i32>,
    pub writes: Cell<i32>,
    pub keys: Cell<i32>,
    pub buckets: Cell<i32>,
    pub orders: Cell<i32>,
}
impl Calls {
    pub fn reset(&self) {
        for counter in [
            &self.reads,
            &self.writes,
            &self.keys,
            &self.buckets,
            &self.orders,
        ] {
            counter.set(0);
        }
    }
}
#[repr(C)]
pub(super) struct Node {
    link: MapLinkNode,
    key: i32,
    value: i32,
    ordered: *mut Self,
    calls: Rc<Calls>,
}
impl Node {
    fn new(key: i32, value: i32, calls: &Rc<Calls>) -> Self {
        Self {
            link: MapLinkNode::new(std::ptr::null_mut()),
            key,
            value,
            ordered: std::ptr::null_mut(),
            calls: calls.clone(),
        }
    }
}
impl BasicMapNode for Node {
    type Key = i32;
    type Value = i32;
    fn bucket_next(&self) -> *mut Self {
        self.calls.buckets.set(self.calls.buckets.get() + 1);
        self.link.next().cast()
    }
    fn order_next(&self) -> *mut Self {
        self.calls.orders.set(self.calls.orders.get() + 1);
        self.ordered
    }
    fn map_key(&self) -> &i32 {
        self.calls.keys.set(self.calls.keys.get() + 1);
        &self.key
    }
    fn map_value(&self) -> &i32 {
        self.calls.reads.set(self.calls.reads.get() + 1);
        &self.value
    }
}
impl BasicMapNodeMut for Node {
    fn map_value_mut(&mut self) -> &mut i32 {
        self.calls.writes.set(self.calls.writes.get() + 1);
        &mut self.value
    }
}

pub(super) struct ProbeMap {
    pub storage: BaseMapStorage,
    first: *mut Node,
}
impl ProbeMap {
    pub fn new(nodes: [*mut Node; 3]) -> Self {
        let mut storage = BaseMapStorage::new(1, None);
        let plan = storage.begin_resize(0).unwrap();
        storage.end_resize(plan);
        storage.bucket_slots_mut()[2] = nodes[0].cast();
        storage.bucket_slots_mut()[9] = nodes[2].cast();
        Self {
            storage,
            first: nodes[2],
        }
    }
    pub fn unallocated() -> Self {
        Self {
            storage: BaseMapStorage::new(1, None),
            first: std::ptr::null_mut(),
        }
    }
}
impl BasicMapIteratorSource for ProbeMap {
    type Node = Node;
    fn basic_bucket_slots(&self) -> Option<&[*mut MapLinkNode]> {
        (!self.storage.buckets().is_null()).then(|| self.storage.bucket_slots())
    }
    fn basic_first_node(&self) -> *mut Node {
        self.first
    }
}

pub(super) fn with_nodes(callback: impl FnOnce([*mut Node; 3], Rc<Calls>)) {
    let calls = Rc::new(Calls::default());
    let mut a = Node::new(1, 101, &calls);
    let mut b = Node::new(2, 202, &calls);
    let mut c = Node::new(3, 303, &calls);
    let b_pointer = &raw mut b;
    *a.link.next_mut() = b_pointer.cast();
    a.ordered = b_pointer;
    let a_pointer = &raw mut a;
    c.ordered = a_pointer;
    let nodes = [a_pointer, b_pointer, &raw mut c];
    callback(nodes, calls);
}

pub(super) unsafe fn value_address(node: *mut Node) -> *mut i32 {
    // SAFETY: every test calls inside with_nodes, while the requested node is live.
    unsafe { &raw mut (*node).value }
}
