//! Native CellHasher reads the initialized integer representation without a scratch buffer.
use super::{CellHash, CellIndex};
use crate::ncollection::HashPolicy;

#[test]
fn cell_hash_reads_coordinate_storage_without_copying() {
    for count in [1, 2, 3, 10, 11, 24, 65] {
        let coordinates: Vec<i32> = (0..count).map(|axis| axis * 7919 - 104729).collect();
        let index = CellIndex::from_slice(&coordinates);
        let words = index.as_slice();
        let bytes = crate::standard::hashing::object_bytes(words, std::mem::size_of_val(words));
        assert_eq!(bytes.as_ptr(), words.as_ptr().cast::<u8>());
        let copied: Vec<u8> = coordinates
            .iter()
            .flat_map(|word| word.to_ne_bytes())
            .collect();
        assert_eq!(bytes, copied);
        assert_eq!(
            CellHash.hash(&index),
            crate::standard::hashing::hash_bytes(&copied)
        );
    }
}

#[test]
fn cell_hash_signed_extremes_match_native_rows() {
    let coordinates = [i32::MIN, -1, 0, 1, i32::MAX, -7919, 104729];
    let mut rows = String::new();
    for count in 1..=coordinates.len() {
        let index = CellIndex::from_slice(&coordinates[..count]);
        rows.push_str(&format!("{count} {}\n", CellHash.hash(&index)));
    }
    #[cfg(target_pointer_width = "64")]
    assert_eq!(
        rows,
        include_str!("../../../../../tools/oracle/tkernel_cell_hash_bytes.expected")
    );
}
