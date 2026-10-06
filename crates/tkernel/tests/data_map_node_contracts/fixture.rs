use rustycad_tkernel::ncollection::{BaseAllocator, DataMapNode};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
#[derive(Clone)]
pub struct NodeTrace {
    events: Rc<RefCell<String>>,
    pub reject_clone: Rc<Cell<char>>,
    pub reject_drop: Rc<Cell<char>>,
    pub value_live: Rc<Cell<bool>>,
    pub key_live: Rc<Cell<bool>>,
    pub arena: BaseAllocator,
}
impl NodeTrace {
    pub fn new() -> Self {
        Self {
            events: Rc::new(RefCell::new(String::new())),
            reject_clone: Rc::new(Cell::new('\0')),
            reject_drop: Rc::new(Cell::new('\0')),
            value_live: Rc::new(Cell::new(false)),
            key_live: Rc::new(Cell::new(false)),
            arena: BaseAllocator::incremental(512),
        }
    }
    pub fn item(&self, kind: char) -> NodeItem {
        NodeItem {
            kind,
            trace: self.clone(),
        }
    }
    pub fn events(&self) -> String {
        self.events.borrow().clone()
    }
    pub fn clear(&self) {
        self.events.borrow_mut().clear();
    }
    pub fn live(&self) -> bool {
        self.arena.reset(false).is_err()
    }
    pub fn borrowed_node(
        &self,
    ) -> rustycad_tkernel::ncollection::DataMapNodeOwner<NodeItem, NodeItem> {
        let key = self.item('K');
        let value = self.item('V');
        // Source cleanup follows this helper; callers reset tracing before observing node Drop.
        DataMapNode::allocate_ref(&self.arena, &key, &value, std::ptr::null_mut())
    }
}
pub struct NodeItem {
    pub kind: char,
    trace: NodeTrace,
}
impl Clone for NodeItem {
    fn clone(&self) -> Self {
        self.trace
            .events
            .borrow_mut()
            .push_str(&format!("C{},", self.kind));
        let observed = if self.kind == 'V' {
            &self.trace.value_live
        } else {
            &self.trace.key_live
        };
        observed.set(self.trace.live());
        assert_ne!(
            self.kind,
            self.trace.reject_clone.get(),
            "node {} clone rejected",
            self.kind
        );
        self.trace.item(self.kind)
    }
}
impl Drop for NodeItem {
    fn drop(&mut self) {
        self.trace
            .events
            .borrow_mut()
            .push_str(&format!("D{},", self.kind));
        if self.kind == self.trace.reject_drop.get() {
            self.trace.reject_drop.set('\0');
            panic!("node {} destruction rejected", self.kind);
        }
    }
}
pub fn native_node_row(prefix: &str, actual: String) {
    let expected =
        include_str!("../../../../tools/oracle/tkernel_data_map_node_contracts.expected");
    let row = expected
        .lines()
        .find(|row| row.starts_with(prefix))
        .unwrap();
    assert_eq!(actual, row);
}
pub fn node_caught(operation: impl FnOnce()) -> bool {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation)).is_err()
}
