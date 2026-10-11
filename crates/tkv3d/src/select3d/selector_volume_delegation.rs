//! Named selection fakes delegate unrelated operations to the real volume implementation.
// Delegation keeps unrelated selector operations on the real implementation.
macro_rules! delegate_volume_operation {
    ($name:ident($($arg:ident: $ty:ty),* $(,)?) -> $ret:ty) => {
        fn $name(&self, $($arg: $ty),*) -> $ret {
            // Traversal overrides point/policy callbacks; quadric fakes override analytic callbacks.
            self.inner.$name($($arg),*)
        }
    };
    ($name:ident($($arg:ident: $ty:ty),* $(,)?)) => {
        fn $name(&self, $($arg: $ty),*) {
            // Mutable-output operations use the same selector implementation.
            self.inner.$name($($arg),*)
        }
    };
}
pub(super) use delegate_volume_operation;
