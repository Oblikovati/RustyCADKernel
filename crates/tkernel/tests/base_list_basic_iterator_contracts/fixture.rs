use rustycad_tkernel::ncollection::{
    BaseListBasicIterator, BaseListStorage, BasicListNode, BasicListNodeMut, ListLinkNode,
};
use std::cell::Cell;
use std::ptr::{addr_of_mut, null_mut};

#[repr(C)]
pub struct OffsetNode {
    padding: [u64; 4],
    pub link: ListLinkNode,
    pub item: i32,
    pub reads: Cell<usize>,
    pub writes: usize,
}
// SAFETY: projection reverses exactly the embedded repr(C) link offset.
unsafe impl BasicListNode for OffsetNode {
    type Item = i32;
    unsafe fn from_list_link(link: *mut ListLinkNode) -> *mut Self {
        unsafe {
            link.cast::<u8>()
                .sub(std::mem::offset_of!(Self, link))
                .cast()
        }
    }
    fn list_value(&self) -> &i32 {
        self.reads.set(self.reads.get() + 1);
        &self.item
    }
}
impl BasicListNodeMut for OffsetNode {
    fn list_change_value(&mut self) -> &mut i32 {
        self.writes += 1;
        &mut self.item
    }
}
pub type Writable = BaseListBasicIterator<OffsetNode, true>;
pub type Readonly = BaseListBasicIterator<OffsetNode>;
pub struct NativeList {
    pub base: BaseListStorage,
    pub nodes: Vec<*mut OffsetNode>,
}
impl NativeList {
    pub fn new() -> Self {
        let mut list = Self {
            base: BaseListStorage::default(),
            nodes: Vec::new(),
        };
        for item in [7, 9] {
            let node = Box::into_raw(Box::new(OffsetNode {
                padding: [0; 4],
                link: ListLinkNode::new(null_mut()),
                item,
                reads: Cell::new(0),
                writes: 0,
            }));
            unsafe { list.base.append_node(addr_of_mut!((*node).link)) };
            list.nodes.push(node);
        }
        list
    }
    pub fn link(&self, index: usize) -> *mut ListLinkNode {
        let node = self.nodes[index];
        unsafe { addr_of_mut!((*node).link) }
    }
}
impl Drop for NativeList {
    fn drop(&mut self) {
        for node in &self.nodes {
            unsafe { drop(Box::from_raw(*node)) };
        }
    }
}
pub fn oracle(row: &str) {
    let native = include_str!(
        "../../../../tools/oracle/tkernel_base_list_basic_iterator_contracts.expected"
    );
    assert!(
        native.lines().any(|line| line == row),
        "unexpected native row {row}"
    );
}
