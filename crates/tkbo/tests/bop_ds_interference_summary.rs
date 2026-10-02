//! Regression coverage for the BOPDS unordered interference summary table.
use rustycad_tkbo::bop_ds::{DataStructure, Pair};

#[test]
fn interference_insertions_deduplicate_reversed_pairs_and_mark_both_shapes() {
    let mut structure = DataStructure::default();
    assert!(structure.interferences().is_empty());
    assert!(!structure.has_interference_for(4));
    assert!(!structure.has_interference_between(4, 9));

    assert!(structure.add_interference(4, 9));
    assert!(structure.add_interference(7, 2));
    assert!(!structure.add_interference(9, 4));
    assert!(structure.add_interference(4, 10));
    assert!(structure.add_interference(7, 7));
    assert_queries(&structure);
    assert_eq!(structure.interferences().len(), 4);
}

#[test]
fn interference_iteration_matches_native_bucket_order() {
    let structure = populated_summary();
    let pairs: Vec<_> = structure
        .interferences()
        .iter()
        .map(Pair::indices)
        .collect();
    assert_eq!(pairs, [(7, 7), (4, 9), (4, 10), (7, 2)]);
}

fn populated_summary() -> DataStructure {
    let mut structure = DataStructure::default();
    structure.add_interference(4, 9);
    structure.add_interference(7, 2);
    structure.add_interference(9, 4);
    structure.add_interference(4, 10);
    structure.add_interference(7, 7);
    structure
}

fn assert_queries(structure: &DataStructure) {
    for index in [2, 4, 7, 9, 10] {
        assert!(structure.has_interference_for(index));
    }
    assert!(!structure.has_interference_for(11));
    for pair in [(4, 9), (9, 4), (2, 7), (7, 2), (7, 7)] {
        assert!(structure.has_interference_between(pair.0, pair.1));
    }
    assert!(!structure.has_interference_between(4, 2));
    assert!(!structure.has_interference_between(7, 8));
}
