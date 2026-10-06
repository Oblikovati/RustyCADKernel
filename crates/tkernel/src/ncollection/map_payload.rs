//! Concrete flat map nodes share native value-before-key construction and reverse destruction.
#![allow(unsafe_code)]
use std::mem::ManuallyDrop;

pub(super) fn construct_value_then_key<V, K>(
    construct_value: impl FnOnce() -> V,
    construct_key: impl FnOnce() -> K,
) -> (V, K) {
    // Keep the value normally owned until key construction succeeds so a key panic drops it.
    let value = construct_value();
    let key = construct_key();
    (value, key)
}
struct MapValueCleanup<'a, V>(&'a mut ManuallyDrop<V>);
impl<V> Drop for MapValueCleanup<'_, V> {
    fn drop(&mut self) {
        // SAFETY: this guard owns cleanup of one still-initialized payload, with no base padding.
        unsafe { ManuallyDrop::drop(self.0) };
    }
}
/// # Safety
/// Both distinct payloads must be initialized and exclusively owned by this one destruction call.
/// The value remains initialized until its guard runs, including while key destruction unwinds.
pub(super) unsafe fn drop_key_then_value<K, V>(
    key: &mut ManuallyDrop<K>,
    value: &mut ManuallyDrop<V>,
) {
    let _value_cleanup = MapValueCleanup(value);
    // SAFETY: caller transfers sole key cleanup; the guard always destroys the value afterwards.
    unsafe { ManuallyDrop::drop(key) };
}
