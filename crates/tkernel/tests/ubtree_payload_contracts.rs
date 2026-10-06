//! Native TreeNode and ObjBnd member lifetime phases, with checked arena links retained.
mod ubtree_payload_contracts {
    pub mod construction;
    pub mod failures;
    pub mod fixture;
    pub mod node_default;
    pub mod traits;
}
fn native_row(label: &str, actual: String) {
    let expected = include_str!("../../../tools/oracle/tkernel_ubtree_payload_contracts.expected");
    assert_eq!(
        Some(actual.as_str()),
        expected
            .lines()
            .find(|row| row.starts_with(&format!("{label} ")))
    );
}
