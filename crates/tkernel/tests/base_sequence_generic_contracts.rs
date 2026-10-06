//! Native generic BasicIterator dispatch, base-offset projection and raw metadata composition.
#![allow(unsafe_code)]
use rustycad_tkernel::ncollection::{
    BaseSequence, BaseSequenceBasicIterator, BaseSequenceRawIterator, BasicSequenceNode, SeqNode,
    SequenceNode,
};
#[path = "base_sequence_generic_contracts/fixture.rs"]
mod fixture;
use fixture::{CustomSequence, CustomSequenceNode};
type Cursor<const M: bool> = BaseSequenceBasicIterator<CustomSequenceNode, M>;

fn converted_rows(sequence: &CustomSequence, rows: &mut Vec<String>) {
    let mut writable = Cursor::<true>::new(&sequence.base, true);
    let previous = unsafe { writable.post_advance() };
    let readonly = writable.convert::<false>();
    let mut legacy = readonly.convert::<true>();
    let mut assigned = Cursor::<true>::default();
    assigned.assign(&readonly);
    let mut raw = BaseSequenceRawIterator::default();
    readonly.copy_to(&mut raw);
    let restored = Cursor::<false>::from_raw(raw);
    rows.push(format!(
        "converted {} {} {} {} {}",
        u8::from(writable == readonly),
        u8::from(legacy == readonly),
        u8::from(assigned == readonly),
        u8::from(restored == readonly),
        u8::from(raw.previous_node() == previous.node())
    ));
    dispatch_rows(sequence, &mut legacy, &readonly, rows);
}
fn dispatch_rows(
    sequence: &CustomSequence,
    legacy: &mut Cursor<true>,
    readonly: &Cursor<false>,
    rows: &mut Vec<String>,
) {
    unsafe {
        *legacy.dereference() = 8;
        *legacy.arrow() = 9;
        *legacy.change_value() = 10;
    }
    let snapshot = unsafe { *legacy.value() };
    let read = unsafe { *readonly.dereference() + *readonly.arrow() };
    let node = sequence.inspect(1);
    rows.push(format!(
        "dispatch {} {} {snapshot} {read}",
        node.reads.get(),
        node.writes
    ));
}
fn movement_rows(sequence: &CustomSequence, rows: &mut Vec<String>) {
    let mut end = Cursor::<true>::new(&sequence.base, false);
    let old = unsafe { end.post_retreat() };
    let at_second = end.node() == sequence.link(1);
    let previous_first = end.mutable_previous_node() == sequence.link(0);
    let address = std::ptr::from_mut(&mut end);
    let returns_self = std::ptr::eq(unsafe { end.retreat() }, address);
    rows.push(format!(
        "movement {} {} {} {} {}",
        u8::from(old.more()),
        u8::from(at_second),
        u8::from(previous_first),
        u8::from(returns_self),
        u8::from(end.node() == sequence.link(0))
    ));
    let other = CustomSequence::new();
    let other_end = Cursor::<false>::new(&other.base, false);
    end.initialize(&sequence.base, false);
    rows.push(format!(
        "ends {} {} {}",
        u8::from(end == other_end),
        u8::from(end != other_end),
        u8::from(end.current_node().is_null())
    ));
}
fn metadata_rows(sequence: &mut CustomSequence, rows: &mut Vec<String>) {
    rows.push(format!(
        "adjusted_base {} {}",
        u8::from(sequence.link(0).cast::<u8>() != sequence.node(0).cast::<u8>()),
        sequence.inspect(0).marker
    ));
    sequence.base.size = 9;
    sequence.base.current_index = 7;
    sequence.base.current = sequence.link(1);
    rows.push(format!(
        "raw_fields {} {} {} {} {}",
        sequence.base.size(),
        sequence.base.length(),
        u8::from(sequence.base.is_empty()),
        sequence.base.current_index(),
        u8::from(sequence.base.current_node() == sequence.link(1))
    ));
}
#[test]
fn custom_sequence_iterator_matches_native_rows() {
    let mut sequence = CustomSequence::new();
    let mut rows = Vec::new();
    converted_rows(&sequence, &mut rows);
    movement_rows(&sequence, &mut rows);
    metadata_rows(&mut sequence, &mut rows);
    assert_eq!(
        rows.join("\n") + "\n",
        include_str!(
            "../../../tools/oracle/tkernel_base_sequence_generic_iterator_contracts.expected"
        )
    );
}
#[test]
fn sequence_cursor_assignment_clone_and_both_conversion_directions_preserve_links() {
    let sequence = CustomSequence::new();
    let mut writable = Cursor::<true>::new(&sequence.base, false);
    unsafe { writable.previous() };
    let readonly = writable.convert::<false>();
    let mut destination = Cursor::<false>::default();
    let identity = std::ptr::from_mut(&mut destination);
    assert!(std::ptr::eq(destination.assign(&writable), identity));
    assert!(std::ptr::eq(destination.assign_self(), identity));
    assert!(destination == readonly);
    let copied = Clone::clone(&destination);
    assert!(copied == writable);
    assert_eq!(copied.previous_node(), sequence.link(0));
    assert_eq!(writable.mutable_node(), sequence.link(1));
    assert_eq!(readonly.node(), sequence.link(1));
}
#[test]
fn sequence_cursor_movement_handles_beginning_and_repeated_end_steps() {
    let sequence = CustomSequence::new();
    let mut cursor = Cursor::<false>::default();
    unsafe {
        cursor.next();
        cursor.previous();
    }
    assert!(!cursor.more());
    cursor.init(&sequence.base, true);
    let identity = std::ptr::from_mut(&mut cursor);
    assert!(std::ptr::eq(unsafe { cursor.advance() }, identity));
    assert_eq!(cursor.node(), sequence.link(1));
    unsafe {
        cursor.next();
        cursor.next();
    }
    assert!(!cursor.more());
    assert_eq!(cursor.previous_node(), sequence.link(1));
    unsafe {
        cursor.previous();
        cursor.previous();
        cursor.previous();
        cursor.previous();
    }
    assert!(!cursor.more());
    assert!(cursor.previous_node().is_null());
}
#[test]
fn sequence_cursor_value_is_readonly_even_in_writable_specialization() {
    let sequence = CustomSequence::new();
    let mut cursor = Cursor::<true>::new(&sequence.base, true);
    assert_eq!(unsafe { *cursor.value_ptr() }, 1);
    assert_eq!(sequence.inspect(0).reads.get(), 1);
    assert_eq!(sequence.inspect(0).writes, 0);
    unsafe {
        *cursor.dereference() = 19;
    }
    assert_eq!(sequence.inspect(0).writes, 1);
    assert_eq!(unsafe { *cursor.value() }, 19);
}
#[test]
fn sequence_cursor_raw_copy_and_custom_projection_preserve_nonzero_offsets() {
    let sequence = CustomSequence::new();
    let raw = BaseSequenceRawIterator {
        current: sequence.link(1),
        previous: sequence.link(0),
    };
    let cursor = Cursor::<false>::from_raw(raw);
    let projected = unsafe { CustomSequenceNode::from_sequence_link(raw.current) };
    assert_eq!(projected, sequence.node(1));
    assert_eq!(unsafe { *cursor.dereference() }, 2);
    let mut destination = BaseSequenceRawIterator::default();
    cursor.copy_to(&mut destination);
    assert_eq!(destination.current, raw.current);
    assert_eq!(destination.previous, raw.previous);
}
#[test]
fn built_in_sequence_node_uses_same_generic_dispatch_contract() {
    let node = Box::into_raw(Box::new(SequenceNode::new(8_i32)));
    let link = node.cast::<SeqNode>();
    let mut base = BaseSequence::default();
    unsafe { base.append_node(link) };
    let mut writable = BaseSequenceBasicIterator::<SequenceNode<i32>, true>::new(&base, true);
    assert_eq!(unsafe { *writable.value() }, 8);
    unsafe { *writable.change_value() = 9 };
    assert_eq!(unsafe { *writable.convert::<false>().value() }, 9);
    assert_eq!(
        unsafe { SequenceNode::<i32>::from_sequence_link(link) },
        node
    );
    unsafe { drop(Box::from_raw(node)) };
}

#[repr(C)]
struct ReadonlySequenceNode {
    link: SeqNode,
    value: i32,
}
// SAFETY: repr(C) guarantees the embedded SeqNode is at offset zero.
unsafe impl BasicSequenceNode for ReadonlySequenceNode {
    type Item = i32;
    unsafe fn from_sequence_link(link: *mut SeqNode) -> *mut Self {
        link.cast()
    }
    fn sequence_value(&self) -> &i32 {
        &self.value
    }
}
#[test]
fn readonly_custom_nodes_need_no_change_value_implementation() {
    let node = Box::into_raw(Box::new(ReadonlySequenceNode {
        link: SeqNode::new(),
        value: 42,
    }));
    let mut base = BaseSequence::default();
    unsafe { base.append_node(node.cast()) };
    let cursor = BaseSequenceBasicIterator::<ReadonlySequenceNode>::new(&base, true);
    let first = unsafe { *cursor.dereference() };
    let second = unsafe { *cursor.arrow() };
    let legacy = cursor.convert::<true>();
    let third = unsafe { *legacy.value() };
    assert_eq!(
        format!("readonly_only {first} {second} {third}\n"),
        include_str!(
            "../../../tools/oracle/tkernel_base_sequence_readonly_node_contracts.expected"
        )
    );
    unsafe { drop(Box::from_raw(node)) };
}
