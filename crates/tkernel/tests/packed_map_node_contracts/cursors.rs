use super::fixture::check_row;
use rustycad_tkernel::ncollection::{PackedInteger, PackedMap, PackedMapNode, PackedWord};

pub fn cursors<T: PackedInteger>(name: &str) {
    let full = T::Word::from_bits(u64::MAX).bits();
    let high = 1_u64 << (PackedMap::<T>::BITS_PER_BLOCK - 1);
    let bases = [0, T::MIN.bits(), T::MAX.bits()];
    let bits = [
        0,
        1,
        high,
        full,
        0xaaaaaaaaaaaaaaaa,
        0x5555555555555555,
        0x80000001,
    ];
    let masks = [0, full, 1, high, full ^ 1];
    for index in 0..105 {
        cursor_row::<T>(
            name,
            "cursor",
            index,
            bases[index / 35],
            bits[index / 5 % 7],
            masks[index % 5],
        );
    }
}

pub fn one_bit<T: PackedInteger>(name: &str) {
    for index in 0..PackedMap::<T>::BITS_PER_BLOCK as usize {
        cursor_row::<T>(name, "one_bit", index, 32, 1 << index, u64::MAX);
    }
}

fn cursor_row<T: PackedInteger>(
    name: &str,
    kind: &str,
    index: usize,
    base: u64,
    bits: u64,
    search: u64,
) {
    let node = PackedMapNode::<T>::from_parts(
        T::Word::from_bits(base),
        T::Word::from_bits(bits),
        std::ptr::null_mut(),
    );
    let mut next_mask = T::Word::from_bits(search);
    let mut prev_mask = next_mask;
    let next = PackedMap::find_next(&node, &mut next_mask);
    let prev = PackedMap::find_prev(&node, &mut prev_mask);
    check_row(
        &format!("{name}:{kind}:{index}"),
        &[next.bits(), next_mask.bits(), prev.bits(), prev_mask.bits()],
    );
}
