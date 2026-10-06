//! Actual native nodes, links, stored indices and callback-visible canonical metadata.
#[path = "indexed_map_canonical/lifetime.rs"]
mod lifetime;
#[path = "indexed_map_canonical/observers.rs"]
mod observers;
#[path = "indexed_map_canonical/topology.rs"]
mod topology;
#[path = "indexed_map_canonical/unwind.rs"]
mod unwind;
fn native_row(label: &str, actual: &str) {
    assert_eq!(Some(actual), native_rows().get(label).copied());
}
fn native_rows() -> &'static std::collections::BTreeMap<&'static str, &'static str> {
    static ROWS: std::sync::OnceLock<std::collections::BTreeMap<&str, &str>> =
        std::sync::OnceLock::new();
    ROWS.get_or_init(|| {
        let expected = [
            include_str!("../../../tools/oracle/tkernel_indexed_map_canonical_0.expected"),
            include_str!("../../../tools/oracle/tkernel_indexed_map_canonical_1.expected"),
            include_str!("../../../tools/oracle/tkernel_indexed_map_canonical_observer.expected"),
        ];
        expected
            .into_iter()
            .flat_map(str::lines)
            .map(|row| (row.split_once(' ').unwrap().0, row))
            .collect()
    })
}
