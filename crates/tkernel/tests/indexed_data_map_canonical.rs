//! Single native indexed key/value nodes, allocator callbacks and disjoint borrowed views.
#[path = "indexed_data_map_canonical/borrowing.rs"]
mod borrowing;
#[path = "indexed_data_map_canonical/lifetime.rs"]
mod lifetime;
#[path = "indexed_data_map_canonical/observers.rs"]
mod observers;
#[path = "indexed_data_map_canonical/route_changes.rs"]
mod route_changes;
#[path = "indexed_data_map_canonical/topology.rs"]
mod topology;
fn native_row(label: &str, actual: &str) {
    static ROWS: std::sync::OnceLock<std::collections::BTreeMap<&str, &str>> =
        std::sync::OnceLock::new();
    let rows = ROWS.get_or_init(|| {
        include_str!("../../../tools/oracle/tkernel_indexed_data_map_canonical.expected")
            .lines()
            .map(|row| (row.split_once(' ').unwrap().0, row))
            .collect()
    });
    assert_eq!(rows.get(label).copied(), Some(actual));
}
