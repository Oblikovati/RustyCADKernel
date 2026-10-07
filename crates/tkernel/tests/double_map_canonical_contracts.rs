//! Dual chains and callback order checked against pinned native DoubleMap behavior.
mod double_map_canonical_contracts {
    pub mod hash_order;
    pub mod observation_fixture;
    pub mod observations;
    pub mod panic_cleanup;
    pub mod topology;
    pub mod traits;
}
fn expected_row(label: &str) -> &'static str {
    let expected =
        include_str!("../../../tools/oracle/tkernel_double_map_canonical_contracts.expected");
    expected
        .lines()
        .find(|row| row.starts_with(&format!("{label} ")))
        .unwrap()
}
fn native_row(label: &str, actual: String) {
    assert_eq!(actual, expected_row(label));
}
/// Compares the callback order of a native row. The probe also printed, after `@`, whether the
/// node arena held a live allocation and the map metadata seen from inside the callback; both
/// are protected native state (plan D1 class B6) that slot storage no longer models, and the
/// trailing `after=` field is checked through the public API by each test.
fn native_order(label: &str, actual: String) {
    let row = expected_row(label);
    let events = row.split(",after=").next().unwrap();
    let order: Vec<&str> = events
        .split(',')
        .map(|event| event.split('@').next().unwrap())
        .collect();
    assert_eq!(actual, order.join(","));
}
