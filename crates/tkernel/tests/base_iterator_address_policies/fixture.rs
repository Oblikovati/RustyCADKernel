use rustycad_tkernel::ncollection::{
    BasicListNode, BasicListNodeMut, BasicSequenceNode, BasicSequenceNodeMut, ListLinkNode, SeqNode,
};
use std::{cell::RefCell, ptr::null_mut, rc::Rc};
pub type Trace = Rc<RefCell<String>>;

pub struct AddressItem {
    pub number: i32,
    pub alternate: *mut Self,
    calls: Trace,
}
impl AddressItem {
    /// Uses the native readonly address overload; `item.const_address()`.
    pub fn const_address(&self) -> *const Self {
        self.calls.borrow_mut().push('a');
        self.alternate.cast_const()
    }
    /// Uses the mutable overload's mutation and redirection; `item.mutable_address()`.
    pub fn mutable_address(&mut self) -> *mut Self {
        self.calls.borrow_mut().push('A');
        self.number += 1;
        self.alternate
    }
}
#[repr(C)]
pub struct AddressNode<L> {
    padding: [u64; 4],
    pub link: L,
    pub item: AddressItem,
}
impl<L> AddressNode<L> {
    fn readonly_item(&self) -> &AddressItem {
        self.item.calls.borrow_mut().push('v');
        &self.item
    }
    fn writable_item(&mut self) -> &mut AddressItem {
        self.item.calls.borrow_mut().push('V');
        &mut self.item
    }
}
// SAFETY: projection reverses the repr(C) link offset with whole-allocation provenance.
unsafe impl BasicListNode for AddressNode<ListLinkNode> {
    type Item = AddressItem;
    unsafe fn from_list_link(link: *mut ListLinkNode) -> *mut Self {
        unsafe { link.byte_sub(std::mem::offset_of!(Self, link)).cast() }
    }
    fn list_value(&self) -> &AddressItem {
        self.readonly_item()
    }
}
impl BasicListNodeMut for AddressNode<ListLinkNode> {
    fn list_change_value(&mut self) -> &mut AddressItem {
        self.writable_item()
    }
}
// SAFETY: projection reverses the repr(C) link offset with whole-allocation provenance.
unsafe impl BasicSequenceNode for AddressNode<SeqNode> {
    type Item = AddressItem;
    unsafe fn from_sequence_link(link: *mut SeqNode) -> *mut Self {
        unsafe { link.byte_sub(std::mem::offset_of!(Self, link)).cast() }
    }
    fn sequence_value(&self) -> &AddressItem {
        self.readonly_item()
    }
}
impl BasicSequenceNodeMut for AddressNode<SeqNode> {
    fn sequence_change_value(&mut self) -> &mut AddressItem {
        self.writable_item()
    }
}

pub struct NodeStorage<L> {
    pub node: *mut AddressNode<L>,
    pub target: *mut AddressItem,
    pub calls: Trace,
}
impl<L> NodeStorage<L> {
    /// Owns both allocations until fixture drop; `NodeStorage::new(link, 10, 30)`.
    pub fn new(link: L, number: i32, target_number: i32) -> Self {
        let calls = Rc::new(RefCell::new(String::new()));
        let target = Self::allocate_target(target_number, &calls);
        let item = AddressItem {
            number,
            alternate: target,
            calls: Rc::clone(&calls),
        };
        let node = Box::into_raw(Box::new(AddressNode {
            padding: [0; 4],
            link,
            item,
        }));
        Self {
            node,
            target,
            calls,
        }
    }
    fn allocate_target(number: i32, calls: &Trace) -> *mut AddressItem {
        Box::into_raw(Box::new(AddressItem {
            number,
            alternate: null_mut(),
            calls: Rc::clone(calls),
        }))
    }
    /// Preserves whole-node provenance for native projection; `storage.link()`.
    pub fn link(&self) -> *mut L {
        unsafe { std::ptr::addr_of_mut!((*self.node).link) }
    }
    /// Reads a source after all temporary getter borrows ended; `storage.source_number()`.
    pub fn source_number(&self) -> i32 {
        unsafe { (*self.node).item.number }
    }
    /// Returns original item identity without invoking address overload; `storage.item_pointer()`.
    pub fn item_pointer(&self) -> *mut AddressItem {
        unsafe { std::ptr::addr_of_mut!((*self.node).item) }
    }
    /// Takes getter/address events in order; `storage.take_calls()`.
    pub fn take_calls(&self) -> String {
        std::mem::take(&mut *self.calls.borrow_mut())
    }
}
impl<L> Drop for NodeStorage<L> {
    fn drop(&mut self) {
        // SAFETY: this fixture alone owns the two original Box allocations.
        unsafe {
            drop(Box::from_raw(self.node));
            drop(Box::from_raw(self.target));
        }
    }
}
