#![allow(unsafe_code)]
//! Address overloads run after native getters and may select another valid allocation.
#[path = "base_iterator_address_policies/fixture.rs"]
mod fixture;
use fixture::{AddressItem, AddressNode, NodeStorage};
use rustycad_tkernel::ncollection::{
    BaseListBasicIterator, BaseListRawIterator, BaseSequenceBasicIterator, BaseSequenceRawIterator,
    ListLinkNode, SeqNode,
};
use std::ptr::null_mut;
type ListCursor<const M: bool> = BaseListBasicIterator<AddressNode<ListLinkNode>, M>;
type SequenceCursor<const M: bool> = BaseSequenceBasicIterator<AddressNode<SeqNode>, M>;

fn row<L>(
    name: &str,
    storage: &NodeStorage<L>,
    pointer: *const AddressItem,
    custom: bool,
) -> String {
    let expected = if custom {
        storage.target
    } else {
        storage.item_pointer()
    };
    assert_eq!(pointer, expected.cast_const());
    // SAFETY: pointer is one of the fixture's two live allocations, with borrows ended.
    let number = unsafe { (*pointer).number };
    let calls = storage.take_calls();
    if custom {
        return format!("{name} 1 {number} {} {calls}\n", storage.source_number());
    }
    format!("{name} 1 {number} {calls}\n")
}
fn list_rows() -> String {
    let storage = NodeStorage::new(ListLinkNode::new(null_mut()), 10, 30);
    let raw = BaseListRawIterator {
        current: storage.link(),
        previous: null_mut(),
    };
    let readonly = ListCursor::<false>::from_raw(raw);
    let mut writable = ListCursor::<true>::from_raw(raw);
    // SAFETY: each exclusive/shared access ends before the next call; storage owns both nodes.
    unsafe {
        let pointer = readonly.arrow_with(AddressItem::const_address);
        let mut rows = row("list_const", &storage, pointer, true);
        let pointer = writable.arrow_with(AddressItem::mutable_address);
        rows += &row("list_mut", &storage, pointer, true);
        rows += &row("list_builtin_const", &storage, readonly.arrow(), false);
        rows += &row("list_builtin_mut", &storage, writable.arrow(), false);
        rows
    }
}
fn sequence_rows() -> String {
    let storage = NodeStorage::new(SeqNode::new(), 50, 70);
    let raw = BaseSequenceRawIterator {
        current: storage.link(),
        previous: null_mut(),
    };
    let readonly = SequenceCursor::<false>::from_raw(raw);
    let mut writable = SequenceCursor::<true>::from_raw(raw);
    // SAFETY: each exclusive/shared access ends before the next call; storage owns both nodes.
    unsafe {
        let pointer = readonly.arrow_with(AddressItem::const_address);
        let mut rows = row("sequence_const", &storage, pointer, true);
        let pointer = writable.arrow_with(AddressItem::mutable_address);
        rows += &row("sequence_mut", &storage, pointer, true);
        rows += &row("sequence_builtin_const", &storage, readonly.arrow(), false);
        rows += &row("sequence_builtin_mut", &storage, writable.arrow(), false);
        rows
    }
}
#[test]
fn all_native_address_policy_rows_match_getter_order_and_pointer_selection() {
    let actual = list_rows() + &sequence_rows();
    assert_eq!(
        actual,
        include_str!("../../../tools/oracle/tkernel_base_iterator_address_policies.expected")
    );
}
#[test]
fn writable_address_result_can_modify_the_redirected_allocation() {
    let storage = NodeStorage::new(ListLinkNode::new(null_mut()), 10, 30);
    let raw = BaseListRawIterator {
        current: storage.link(),
        previous: null_mut(),
    };
    let mut writable = ListCursor::<true>::from_raw(raw);
    // SAFETY: returned pointer names the separately owned, exclusively accessible live target.
    unsafe {
        (*writable.arrow_with(AddressItem::mutable_address)).number = 99;
    }
    assert_eq!(storage.source_number(), 11);
    assert_eq!(unsafe { (*storage.target).number }, 99);
    assert_eq!(storage.take_calls(), "VA");
}
#[test]
fn readonly_policy_accepts_an_owned_once_capture() {
    let storage = NodeStorage::new(SeqNode::new(), 50, 70);
    let raw = BaseSequenceRawIterator {
        current: storage.link(),
        previous: null_mut(),
    };
    let readonly = SequenceCursor::<false>::from_raw(raw);
    let owned = String::from("address policy");
    // SAFETY: fixture remains live/readable and the policy retains no references.
    let pointer = unsafe {
        readonly.arrow_with(|item| {
            drop(owned);
            item.const_address()
        })
    };
    assert_eq!(pointer, storage.target.cast_const());
    assert_eq!(storage.take_calls(), "va");
}
