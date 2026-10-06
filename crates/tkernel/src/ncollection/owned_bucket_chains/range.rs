//! Shared ranges borrow the sole graph owner and lend immutable concrete nodes in native order.
use super::{BucketChainPosition, CanonicalMapNode, OwnedBucketChains};
pub(in crate::ncollection) struct BucketNodeRange<'a, N: CanonicalMapNode> {
    source: Option<&'a OwnedBucketChains<N>>,
    position: BucketChainPosition,
}
impl<'a, N: CanonicalMapNode> BucketNodeRange<'a, N> {
    pub(in crate::ncollection) fn new(source: &'a OwnedBucketChains<N>) -> Self {
        Self::from_position(Some(source), BucketChainPosition::new(&source.base))
    }
    pub(in crate::ncollection) fn from_position(
        source: Option<&'a OwnedBucketChains<N>>,
        position: BucketChainPosition,
    ) -> Self {
        Self { source, position }
    }
    pub(in crate::ncollection) fn current(&self) -> Option<&'a N> {
        if self.position.node.is_null() {
            return None;
        }
        Some(unsafe { self.source?.node(self.position.node) })
    }
    pub(in crate::ncollection) fn reset(&mut self) {
        self.position = self
            .source
            .map_or_else(BucketChainPosition::default, |source| {
                BucketChainPosition::new(&source.base)
            });
    }
}
// SAFETY: each shared range borrows a graph that cannot mutate; both payloads must be Sync.
unsafe impl<N: CanonicalMapNode> Send for BucketNodeRange<'_, N>
where
    N::Key: Sync,
    N::Value: Sync,
{
}
unsafe impl<N: CanonicalMapNode> Sync for BucketNodeRange<'_, N>
where
    N::Key: Sync,
    N::Value: Sync,
{
}
impl<'a, N: CanonicalMapNode> Iterator for BucketNodeRange<'a, N> {
    type Item = &'a N;
    fn next(&mut self) -> Option<Self::Item> {
        let node = self.current()?;
        self.position.advance(&self.source?.base);
        Some(node)
    }
}
impl<N: CanonicalMapNode> Default for BucketNodeRange<'_, N> {
    fn default() -> Self {
        Self {
            source: None,
            position: BucketChainPosition::default(),
        }
    }
}
impl<N: CanonicalMapNode> Clone for BucketNodeRange<'_, N> {
    fn clone(&self) -> Self {
        Self {
            source: self.source,
            position: self.position,
        }
    }
}
impl<N: CanonicalMapNode> PartialEq for BucketNodeRange<'_, N> {
    fn eq(&self, other: &Self) -> bool {
        self.position.node == other.position.node
    }
}
impl<N: CanonicalMapNode> Eq for BucketNodeRange<'_, N> {}
