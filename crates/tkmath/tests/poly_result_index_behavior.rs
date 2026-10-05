//! Result indexing addresses physical root storage independently of the reported root count.
use rustycad_tkmath::{math_poly, math_utils};

const ROOT_BITS: [u64; 4] = [
    0,
    0x8000_0000_0000_0000,
    0x7ff0_0000_0000_0000,
    0x7ff8_0000_0000_1234,
];

#[test]
fn math_poly_index_preserves_all_root_slots_with_zero_root_count() {
    let mut result = math_poly::PolyResult::default();
    for (index, root) in result.roots.iter_mut().enumerate() {
        *root = f64::from_bits(ROOT_BITS[index % ROOT_BITS.len()]);
    }
    assert_eq!(result.nb_roots, 0);
    for index in 0..math_poly::MAX_POLY_DEGREE {
        assert_eq!(result[index].to_bits(), ROOT_BITS[index % ROOT_BITS.len()]);
    }
}

#[test]
fn math_utils_index_preserves_all_root_slots_with_zero_root_count() {
    let mut result = math_utils::PolyResult::default();
    for (root, bits) in result.roots.iter_mut().zip(ROOT_BITS) {
        *root = f64::from_bits(bits);
    }
    assert_eq!(result.number_of_roots, 0);
    for (index, bits) in ROOT_BITS.into_iter().enumerate() {
        assert_eq!(result[index].to_bits(), bits);
    }
}
