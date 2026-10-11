//! Bounded native virtual-policy chunks preserve exact callback traces.
pub(super) const GROUP_POLICY_NATIVE: &str = concat!(
    include_str!("../../tests/fixtures/sensitive-group-policy-00.txt"),
    include_str!("../../tests/fixtures/sensitive-group-policy-01.txt"),
);
