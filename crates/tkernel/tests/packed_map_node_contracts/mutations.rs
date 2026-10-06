use super::fixture::snapshot;
use rustycad_tkernel::ncollection::{PackedInteger, PackedMap, PackedMapNode, PackedWord};

pub fn mutations<T: PackedInteger>(name: &str) {
    let mut node = PackedMapNode::<T>::default();
    mutate_keys(name, "add", &mut node, true);
    mutate_keys(name, "remove", &mut node, false);
    wrapped_mask::<T>(name);
}

fn mutate_keys<T: PackedInteger>(
    name: &str,
    kind: &str,
    node: &mut PackedMapNode<T>,
    inserting: bool,
) {
    let mutate: fn(&mut PackedMapNode<T>, T) -> bool = if inserting {
        PackedMapNode::add_value
    } else {
        PackedMapNode::del_value
    };
    for index in 0..PackedMap::<T>::BITS_PER_BLOCK as usize {
        let key = T::from_bits(index as u64);
        let changed = mutate(node, key);
        let repeated = mutate(node, key);
        let matched = changed && !repeated && node.has_value(key) == inserting;
        snapshot(name, kind, index, node, matched);
    }
}

fn wrapped_mask<T: PackedInteger>(name: &str) {
    let mut node = PackedMapNode::<T>::from_parts(
        T::Word::from_bits(u64::MAX),
        T::Word::from_bits(0),
        std::ptr::null_mut(),
    );
    let added = node.add_value(T::from_bits(0));
    snapshot(name, "wrap", 0, &node, added);
    let removed = node.del_value(T::from_bits(0));
    snapshot(name, "wrap", 1, &node, removed);
}
