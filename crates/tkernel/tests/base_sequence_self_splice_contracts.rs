//! Native raw self-splice intentionally creates cycles then forgets the topology.
#![allow(unsafe_code)]
use rustycad_tkernel::ncollection::{BaseSequence, SeqNode};

struct RawSelfSequence {
    base: BaseSequence,
    nodes: [*mut SeqNode; 3],
}
impl RawSelfSequence {
    fn new() -> Self {
        let nodes = [(); 3].map(|()| Box::into_raw(Box::new(SeqNode::new())));
        let mut sequence = Self {
            base: BaseSequence::default(),
            nodes,
        };
        for node in nodes {
            unsafe { sequence.base.append_node(node) };
        }
        sequence
    }
    fn id(&self, pointer: *mut SeqNode) -> usize {
        self.nodes
            .iter()
            .position(|node| *node == pointer)
            .map_or(0, |index| index + 1)
    }
    fn snapshot(&self, name: &str) -> String {
        let links = self
            .nodes
            .iter()
            .map(|node| unsafe {
                format!(
                    " {} {}",
                    self.id((**node).previous()),
                    self.id((**node).next())
                )
            })
            .collect::<String>();
        format!("{name} {}{links}", self.base.size())
    }
    fn assert_null_metadata(&self) {
        assert!(self.base.is_empty());
        assert!(self.base.first_node().is_null());
        assert!(self.base.last_node().is_null());
        assert!(self.base.current_node().is_null());
        assert_eq!(self.base.current_index(), 0);
    }
}
impl Drop for RawSelfSequence {
    fn drop(&mut self) {
        // Cyclic links are nonowning: dispose each retained allocation exactly once.
        for node in self.nodes {
            unsafe { drop(Box::from_raw(node)) };
        }
    }
}
#[test]
fn raw_sequence_self_splice_matches_native_cycles_and_metadata() {
    let mut rows = Vec::new();
    let mut append = RawSelfSequence::new();
    unsafe { append.base.append_self() };
    rows.push(append.snapshot("append_self"));
    append.assert_null_metadata();
    let mut prepend = RawSelfSequence::new();
    unsafe { prepend.base.prepend_self() };
    rows.push(prepend.snapshot("prepend_self"));
    prepend.assert_null_metadata();
    for index in 0..=3 {
        let mut insert = RawSelfSequence::new();
        unsafe { insert.base.insert_self_after(index).unwrap() };
        rows.push(insert.snapshot("insert_self"));
        insert.assert_null_metadata();
    }
    assert_eq!(
        rows.join("\n") + "\n",
        include_str!("../../../tools/oracle/tkernel_base_sequence_self_splice_contracts.expected")
    );
}
#[test]
fn raw_self_splice_rejects_invalid_indices_before_mutating_any_link() {
    let mut sequence = RawSelfSequence::new();
    let expected = sequence.snapshot("before");
    let error = unsafe { sequence.base.insert_self_after(4) }.unwrap_err();
    assert!(error.message().contains("4"));
    assert!(error.message().contains("0..=3"));
    assert_eq!(sequence.snapshot("before"), expected);
}
#[test]
fn raw_self_splice_handles_empty_and_singleton_sequences() {
    let mut base = BaseSequence::default();
    unsafe {
        base.append_self();
        base.prepend_self();
        base.insert_self_after(0).unwrap();
    }
    assert!(unsafe { base.insert_self_after(1) }.is_err());
    let node = Box::into_raw(Box::new(SeqNode::new()));
    unsafe {
        base.append_node(node);
        base.insert_self_after(1).unwrap();
    }
    assert!(base.is_empty());
    assert_eq!(unsafe { (*node).next() }, node);
    assert_eq!(unsafe { (*node).previous() }, node);
    unsafe { drop(Box::from_raw(node)) };
}
