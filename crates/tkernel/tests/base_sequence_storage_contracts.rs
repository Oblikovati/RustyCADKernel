//! Raw protected BaseSequence methods checked against a native derived-class fixture.
#![allow(unsafe_code)]
use rustycad_tkernel::ncollection::{
    BaseAllocator, BaseSequence, BaseSequenceRawIterator, SeqNode, Sequence, SequenceNode,
};

#[derive(Default)]
struct RawSequence(BaseSequence);
impl Drop for RawSequence {
    fn drop(&mut self) {
        // SAFETY: fixture nodes all have the exact i32 type and common resource unless explicitly cleared.
        unsafe {
            self.0
                .clear_seq(|node, arena| Sequence::<i32>::del_node(node, arena))
        };
    }
}
impl RawSequence {
    fn allocate(&self, value: i32) -> *mut SeqNode {
        SequenceNode::into_raw(SequenceNode::allocate(self.0.allocator(), value)).0
    }
    fn append(&mut self, value: i32) {
        let node = self.allocate(value);
        unsafe { self.0.append_node(node) };
    }
    fn prepend(&mut self, value: i32) {
        let node = self.allocate(value);
        unsafe { self.0.prepend_node(node) };
    }
    fn after(&mut self, index: usize, value: i32) {
        let node = self.allocate(value);
        unsafe { self.0.insert_after(index, node).unwrap() };
    }
    fn cache(&mut self, index: usize) {
        unsafe { self.0.cache_position(index).unwrap() };
    }
    fn values(&self) -> Vec<i32> {
        (1..=self.0.size())
            .map(|index| unsafe { node_value(self.0.find(index)) })
            .collect()
    }
    fn snapshot(&self, name: &str) -> String {
        let current = self.0.current_node();
        let current_value = if current.is_null() {
            -1
        } else {
            unsafe { node_value(current) }
        };
        let values = self
            .values()
            .iter()
            .map(|v| format!(" {v}"))
            .collect::<String>();
        format!(
            "{name} {} {} {current_value} :{values}",
            self.0.size(),
            self.0.current_index()
        )
    }
}
unsafe fn node_value(node: *mut SeqNode) -> i32 {
    // SAFETY: fixtures insert only live SequenceNode<i32> allocations with prefix-first layout.
    unsafe { *(*node.cast::<SequenceNode<i32>>()).value() }
}
fn delete(node: *mut SeqNode, arena: &mut BaseAllocator) {
    // SAFETY: all fixtures pass their uniquely owned i32 node and retained matching resource.
    unsafe { Sequence::<i32>::del_node(node, arena) };
}
fn edit_rows(rows: &mut Vec<String>) {
    let mut sequence = RawSequence::default();
    rows.push(sequence.snapshot("empty"));
    for value in 1..=3 {
        sequence.append(value);
    }
    sequence.cache(2);
    sequence.prepend(0);
    rows.push(sequence.snapshot("prepend_cache"));
    sequence.after(1, 8);
    rows.push(sequence.snapshot("index_insert_cache"));
    let cursor = BaseSequenceRawIterator::new(&sequence.0, true);
    unsafe {
        sequence
            .0
            .insert_after_cursor(&cursor, sequence.allocate(9))
    };
    rows.push(sequence.snapshot("cursor_insert_cache"));
    reorder_rows(&mut sequence, rows);
}
fn reorder_rows(sequence: &mut RawSequence, rows: &mut Vec<String>) {
    sequence.cache(4);
    unsafe { sequence.0.reverse() };
    rows.push(sequence.snapshot("reverse_cache"));
    unsafe { sequence.0.exchange(1, 6).unwrap() };
    rows.push(sequence.snapshot("exchange_edges"));
    unsafe { sequence.0.exchange(2, 3).unwrap() };
    rows.push(sequence.snapshot("exchange_neighbors"));
    unsafe { sequence.0.exchange(3, 3).unwrap() };
    rows.push(sequence.snapshot("exchange_equal"));
}
fn transfer_rows(rows: &mut Vec<String>) {
    let mut first = RawSequence::default();
    let mut second = RawSequence::default();
    first.append(1);
    first.append(2);
    first.cache(2);
    second.append(7);
    second.append(8);
    unsafe { first.0.insert_sequence_after(1, &mut second.0).unwrap() };
    rows.push(first.snapshot("insert_sequence"));
    rows.push(second.snapshot("source_empty"));
    second.append(9);
    unsafe { first.0.prepend_sequence(&mut second.0) };
    rows.push(first.snapshot("prepend_sequence"));
    second.append(10);
    unsafe { first.0.append_sequence(&mut second.0) };
    rows.push(first.snapshot("append_sequence"));
    split_rows(&mut first, &mut second, rows);
}
fn split_rows(first: &mut RawSequence, second: &mut RawSequence, rows: &mut Vec<String>) {
    first.cache(4);
    unsafe { first.0.split(3, &mut second.0).unwrap() };
    rows.push(first.snapshot("split_head"));
    rows.push(second.snapshot("split_tail"));
    let mut empty = RawSequence::default();
    assert!(unsafe { empty.0.insert_sequence_after(1, &mut second.0) }.is_err());
    rows.push("invalid_insert 1".into());
    assert!(second.0.split_self(1).is_err());
    rows.push("self_split 1".into());
}
fn removal_rows(rows: &mut Vec<String>) {
    let mut sequence = RawSequence::default();
    for value in 1..=6 {
        sequence.append(value);
    }
    sequence.cache(4);
    unsafe { sequence.0.remove(2, delete).unwrap() };
    rows.push(sequence.snapshot("remove_before_cache"));
    unsafe { sequence.0.remove(3, delete).unwrap() };
    rows.push(sequence.snapshot("remove_at_cache"));
    unsafe { sequence.0.remove_range(2, 3, delete).unwrap() };
    rows.push(sequence.snapshot("remove_range_cache"));
    remove_cursor_rows(&mut sequence, rows);
}
fn remove_cursor_rows(sequence: &mut RawSequence, rows: &mut Vec<String>) {
    let mut cursor = BaseSequenceRawIterator::new(&sequence.0, false);
    unsafe { cursor.previous() };
    unsafe { sequence.0.remove_cursor(&mut cursor, delete) };
    rows.push(sequence.snapshot("remove_cursor_cache"));
    unsafe { sequence.0.remove_cursor(&mut cursor, delete) };
    rows.push(sequence.snapshot("remove_end_noop"));
    unsafe { sequence.0.clear_seq(delete) };
    rows.push(sequence.snapshot("clear_cache"));
}

#[test]
fn base_sequence_protected_engine_matches_native_subclass_rows() {
    let mut rows = Vec::new();
    edit_rows(&mut rows);
    transfer_rows(&mut rows);
    removal_rows(&mut rows);
    let expected =
        include_str!("../../../tools/oracle/tkernel_base_sequence_storage_contracts.expected");
    assert_eq!(rows.join("\n") + "\n", expected);
}

#[test]
fn base_sequence_find_uses_consistent_cache_for_every_search_direction() {
    let mut sequence = RawSequence::default();
    for value in 1..=10 {
        sequence.append(value);
    }
    for cache in 1..=10 {
        sequence.cache(cache);
        assert_eq!(sequence.values(), (1..=10).collect::<Vec<_>>());
        assert_eq!(sequence.0.current_index(), cache);
        assert_eq!(
            unsafe { node_value(sequence.0.current_node()) },
            cache as i32
        );
    }
    assert_eq!(sequence.0.length(), 10);
    assert_eq!(sequence.0.first_node(), unsafe { sequence.0.find(1) });
    assert_eq!(sequence.0.last_node(), unsafe { sequence.0.find(10) });
}

#[test]
fn base_sequence_raw_iterator_preserves_previous_when_removing_current() {
    let mut sequence = RawSequence::default();
    sequence.append(1);
    sequence.append(2);
    sequence.append(3);
    let mut cursor = BaseSequenceRawIterator::default();
    assert!(cursor.current_node().is_null());
    cursor.init(&sequence.0, true);
    unsafe { cursor.next() };
    let previous = cursor.previous_node();
    let mut copied = cursor;
    unsafe { sequence.0.remove_cursor(&mut cursor, delete) };
    assert_eq!(cursor.previous_node(), previous);
    assert_eq!(unsafe { node_value(cursor.current_node()) }, 3);
    unsafe { cursor.previous() };
    assert_eq!(unsafe { node_value(cursor.current_node()) }, 1);
    check_end_cursor(&sequence, &mut copied);
}
fn check_end_cursor(sequence: &RawSequence, copied: &mut BaseSequenceRawIterator) {
    copied.init(&sequence.0, false);
    unsafe { copied.previous() };
    assert_eq!(unsafe { node_value(copied.current_node()) }, 3);
    unsafe {
        copied.next();
        copied.next()
    };
    assert!(copied.current_node().is_null());
}

#[test]
fn base_sequence_invalid_indices_preserve_metadata_and_skip_deleter() {
    let mut sequence = RawSequence::default();
    sequence.append(1);
    let mut calls = 0;
    assert!(unsafe { sequence.0.remove(0, |_, _| calls += 1) }.is_err());
    assert!(unsafe { sequence.0.remove_range(2, 1, |_, _| calls += 1) }.is_err());
    assert!(unsafe { sequence.0.exchange(0, 1) }.is_err());
    assert!(unsafe { sequence.0.cache_position(2) }.is_err());
    let mut empty = RawSequence::default();
    assert!(unsafe { sequence.0.insert_sequence_after(2, &mut empty.0) }.is_err());
    assert!(unsafe { sequence.0.split(0, &mut empty.0) }.is_err());
    assert!(empty.0.split_self(1).is_err());
    assert_eq!(calls, 0);
    assert_eq!(sequence.values(), [1]);
}

#[test]
fn base_sequence_callback_can_replace_the_allocator_handle_between_deletions() {
    let mut sequence = RawSequence::default();
    sequence.append(1);
    sequence.append(2);
    let original = sequence.0.allocator().clone();
    let replacement = BaseAllocator::incremental(4096);
    let mut seen = Vec::new();
    unsafe {
        sequence.0.clear_seq(|node, arena| {
            seen.push((node_value(node), arena == &original));
            delete(node, &mut original.clone());
            *arena = replacement.clone();
        })
    };
    assert_eq!(seen, [(1, true), (2, false)]);
    assert_eq!(sequence.0.allocator(), &replacement);
    assert!(sequence.0.is_empty());
    assert_eq!(sequence.0.current_index(), 0);
    *sequence.0.change_allocator() = original;
}

#[test]
fn base_sequence_raw_transfers_do_not_replace_either_allocator() {
    let mut source = RawSequence::default();
    source.append(1);
    source.append(2);
    let other = BaseAllocator::incremental(4096);
    let mut target = BaseSequence::new(Some(other.clone()));
    unsafe { source.0.split(1, &mut target).unwrap() };
    assert!(source.0.is_empty());
    assert_eq!(target.allocator(), &other);
    let original = BaseAllocator::common();
    unsafe { target.clear_seq(|node, _| delete(node, &mut original.clone())) };
    assert!(target.is_empty());
    assert_eq!(target.allocator(), &other);
}

#[test]
fn base_sequence_empty_insert_and_prepend_append_transfer_branches() {
    let mut first = RawSequence::default();
    let mut second = RawSequence::default();
    unsafe {
        first.0.append_sequence(&mut second.0);
        first.0.prepend_sequence(&mut second.0);
        first.0.reverse()
    };
    second.append(2);
    unsafe { first.0.prepend_sequence(&mut second.0) };
    assert_eq!(first.values(), [2]);
    let end = BaseSequenceRawIterator::new(&first.0, false);
    unsafe { first.0.insert_after_cursor(&end, first.allocate(1)) };
    assert_eq!(first.values(), [1, 2]);
    second.append(0);
    unsafe { first.0.insert_sequence_after(0, &mut second.0).unwrap() };
    assert_eq!(first.values(), [0, 1, 2]);
}

#[test]
fn base_sequence_drop_leaves_node_deletion_to_the_subclass_callback() {
    let arena = BaseAllocator::incremental(4096);
    let owner = SequenceNode::allocate(&arena, 7_i32);
    let (node, mut allocator) = SequenceNode::into_raw(owner);
    let mut base = BaseSequence::new(Some(arena.clone()));
    unsafe { base.append_node(node) };
    drop(base);
    assert_eq!(unsafe { node_value(node) }, 7);
    assert!(arena.reset(false).is_err());
    let callback: rustycad_tkernel::ncollection::DelSeqNode = Sequence::<i32>::del_node;
    unsafe { callback(node, &mut allocator) };
    assert!(arena.reset(false).is_ok());
}
