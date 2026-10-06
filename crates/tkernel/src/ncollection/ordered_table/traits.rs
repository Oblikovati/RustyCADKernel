//! Heap-owned node payloads determine container auto traits, never their non-owning links.
use super::*;
// SAFETY: exclusive table ownership transfers every payload and its graph together.
unsafe impl<K: Send, V: Send, H: Send, N: StorageNode<K, V>> Send for OrderedTable<K, V, H, N> {}
// SAFETY: shared table methods only read links and expose shared payload references.
unsafe impl<K: Sync, V: Sync, H: Sync, N: StorageNode<K, V>> Sync for OrderedTable<K, V, H, N> {}
impl<
    K: std::panic::UnwindSafe,
    V: std::panic::UnwindSafe,
    H: std::panic::UnwindSafe,
    N: StorageNode<K, V>,
> std::panic::UnwindSafe for OrderedTable<K, V, H, N>
{
}
impl<
    K: std::panic::RefUnwindSafe,
    V: std::panic::RefUnwindSafe,
    H: std::panic::RefUnwindSafe,
    N: StorageNode<K, V>,
> std::panic::RefUnwindSafe for OrderedTable<K, V, H, N>
{
}
