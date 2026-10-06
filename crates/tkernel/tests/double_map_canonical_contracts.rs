//! Real dual chains and callback phases checked against pinned native DoubleMap behavior.
#![allow(unsafe_code)]
mod double_map_canonical_contracts {
    pub mod hash_order;
    pub mod observation_fixture;
    pub mod observations;
    pub mod panic_cleanup;
    pub mod topology;
    pub mod traits;
}
fn native_row(label: &str, actual: String) {
    let expected =
        include_str!("../../../tools/oracle/tkernel_double_map_canonical_contracts.expected");
    assert_eq!(
        actual,
        expected
            .lines()
            .find(|row| row.starts_with(&format!("{label} ")))
            .unwrap()
    );
}
