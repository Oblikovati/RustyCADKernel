//! Native same-domain chain, sentinel, and rebinding regressions.
use rustycad_tkbo::bop_ds::DataStructure;
use std::fmt::Write;

fn append_lookup_rows(ds: &DataStructure, stage: i32, rows: &mut String) {
    for index in [-4, 1, 2, 3, 4, 5] {
        let mut representative = 987;
        let mapped = ds.has_shape_sd_into(index, &mut representative);
        assert_eq!(ds.shape_sd(index), mapped.then_some(representative));
        writeln!(
            rows,
            "{stage} {index} {} {representative} {}",
            i32::from(mapped),
            ds.same_domain_index(index)
        )
        .unwrap();
    }
}

#[test]
fn same_domain_chains_match_native_lookup_and_rebinding() {
    let mut ds = DataStructure::default();
    let mut rows = String::new();
    ds.add_shape_sd(1, 1);
    assert_eq!(ds.shape_sd_items().count(), 0);
    append_lookup_rows(&ds, 0, &mut rows);
    for (source, target) in [(1, 2), (2, 3), (3, 4), (-4, 1)] {
        ds.add_shape_sd(source, target);
    }
    append_lookup_rows(&ds, 1, &mut rows);
    ds.add_shape_sd(1, 1);
    ds.add_shape_sd(2, 5);
    append_lookup_rows(&ds, 2, &mut rows);
    assert_eq!(rows.lines().count(), 18);
    assert_eq!(
        rows,
        include_str!("../../../tools/oracle/tkbo_same_domain.expected")
    );
    assert_eq!(
        ds.shape_sd_items().find(|(key, _)| **key == 1),
        Some((&1, &2))
    );
}
