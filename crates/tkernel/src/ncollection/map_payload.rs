//! Native map nodes construct their value (the `TListNode` base) before their key member.

pub(super) fn construct_value_then_key<V, K>(
    construct_value: impl FnOnce() -> V,
    construct_key: impl FnOnce() -> K,
) -> (V, K) {
    // Keep the value normally owned until key construction succeeds so a key panic drops it.
    let value = construct_value();
    let key = construct_key();
    (value, key)
}
