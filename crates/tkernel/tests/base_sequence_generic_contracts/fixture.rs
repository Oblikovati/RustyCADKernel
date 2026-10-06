use rustycad_tkernel::ncollection::{
    BaseSequence, BasicSequenceNode, BasicSequenceNodeMut, SeqNode,
};
use std::cell::Cell;

#[repr(C)]
pub struct CustomSequenceNode {
    pub marker: u64,
    pub links: SeqNode,
    pub number: i32,
    pub reads: Cell<usize>,
    pub writes: usize,
}
impl CustomSequenceNode {
    fn new(number: i32) -> Self {
        Self {
            marker: 42,
            links: SeqNode::new(),
            number,
            reads: Cell::new(0),
            writes: 0,
        }
    }
}
// SAFETY: repr(C) fixes the link offset, and subtraction retains original allocation provenance.
unsafe impl BasicSequenceNode for CustomSequenceNode {
    type Item = i32;
    unsafe fn from_sequence_link(link: *mut SeqNode) -> *mut Self {
        unsafe {
            link.cast::<u8>()
                .sub(std::mem::offset_of!(Self, links))
                .cast()
        }
    }
    fn sequence_value(&self) -> &i32 {
        self.reads.set(self.reads.get() + 1);
        &self.number
    }
}
impl BasicSequenceNodeMut for CustomSequenceNode {
    fn sequence_change_value(&mut self) -> &mut i32 {
        self.writes += 1;
        &mut self.number
    }
}
pub struct CustomSequence {
    pub base: BaseSequence,
    nodes: [*mut CustomSequenceNode; 2],
}
impl CustomSequence {
    pub fn new() -> Self {
        let nodes = [1, 2].map(|number| Box::into_raw(Box::new(CustomSequenceNode::new(number))));
        let mut sequence = Self {
            base: BaseSequence::default(),
            nodes,
        };
        for index in 0..2 {
            unsafe { sequence.base.append_node(sequence.link(index)) };
        }
        sequence
    }
    pub fn link(&self, index: usize) -> *mut SeqNode {
        unsafe { std::ptr::addr_of_mut!((*self.nodes[index]).links) }
    }
    pub fn inspect(&self, index: usize) -> &CustomSequenceNode {
        unsafe { &*self.nodes[index] }
    }
    pub fn node(&self, index: usize) -> *mut CustomSequenceNode {
        self.nodes[index]
    }
}
impl Drop for CustomSequence {
    fn drop(&mut self) {
        for node in self.nodes {
            unsafe { drop(Box::from_raw(node)) };
        }
    }
}
