use super::fixture::{check_row, constructor_keys, snapshot};
use rustycad_tkernel::ncollection::{PackedInteger, PackedMapNode, PackedWord};

pub fn constructors<T: PackedInteger>(name: &str) {
    let mut tail = PackedMapNode::<T>::default();
    let tail_pointer = std::ptr::from_mut(&mut tail);
    let mut head = tail_pointer;
    let empty = PackedMapNode::new(head);
    snapshot(name, "constructor", 0, &empty, empty.next() == head);
    for (index, key) in constructor_keys::<T>().into_iter().enumerate() {
        let node = PackedMapNode::from_value(key, &mut head);
        snapshot(
            name,
            "constructor",
            index + 1,
            &node,
            node.next() == tail_pointer && head == tail_pointer,
        );
    }
    raw_constructor::<T>(name, head);
}

fn raw_constructor<T: PackedInteger>(name: &str, head: *mut PackedMapNode<T>) {
    let mut raw =
        PackedMapNode::<T>::from_parts(T::Word::from_bits(u64::MAX), T::Word::from_bits(0), head);
    snapshot(name, "constructor", 8, &raw, raw.next() == head);
    *raw.change_mask() = T::Word::from_bits(32);
    *raw.change_bits() = T::Word::from_bits(3);
    raw.set_next(std::ptr::null_mut());
    snapshot(name, "constructor", 9, &raw, raw.next().is_null());
}

pub fn layout<T: PackedInteger>(name: &str) {
    let mut node = PackedMapNode::<T>::default();
    let base = std::ptr::from_ref(&node).addr();
    let mask_offset = std::ptr::from_mut(node.change_mask()).addr() - base;
    let bits_offset = std::ptr::from_mut(node.change_bits()).addr() - base;
    let actual = [
        size_of::<PackedMapNode<T>>() as u64,
        align_of::<PackedMapNode<T>>() as u64,
        0,
        mask_offset as u64,
        bits_offset as u64,
    ];
    check_row(&format!("{name}:layout"), &actual);
}
