#[path = "../packed_map_oracle/mod.rs"]
mod decoder;
use decoder::{NativeRows, decode_native_rows};
use rustycad_tkernel::ncollection::{PackedInteger, PackedMap, PackedMapNode, PackedWord};
use std::sync::OnceLock;

pub fn check_row(label: &str, actual: &[u64]) {
    static ROWS: OnceLock<NativeRows> = OnceLock::new();
    let rows = ROWS.get_or_init(|| {
        decode_native_rows(include_str!(
            "../../../../tools/oracle/tkernel_packed_map_node.expected"
        ))
    });
    assert_eq!(actual, rows[label].as_slice(), "{label}");
}

pub fn snapshot<T: PackedInteger>(
    name: &str,
    kind: &str,
    step: usize,
    node: &PackedMapNode<T>,
    changed: bool,
) {
    let actual = [
        node.mask().bits(),
        node.bits().bits(),
        node.key().bits(),
        node.nb_values() as u64,
        u64::from(node.has_values()),
        u64::from(changed),
        node.hash_code(101) as u64,
        u64::from(node.is_equal(PackedMap::<T>::packed_key_index(node.key()))),
    ];
    check_row(&format!("{name}:{kind}:{step}"), &actual);
}

pub fn constructor_keys<T: PackedInteger>() -> [T; 7] {
    [0, 1, u64::MAX, 31, 32, T::MIN.bits(), T::MAX.bits()].map(T::from_bits)
}
