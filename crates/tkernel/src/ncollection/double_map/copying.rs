use super::{Debug, DoubleMap, HashPolicy};
impl<A: Debug + Clone, B: Debug + Clone, H1: HashPolicy<A>, H2: HashPolicy<B>>
    DoubleMap<A, B, H1, H2>
{
    /// Copies bindings retaining destination policies and allocator; `map.assign(&source)`.
    pub fn assign(&mut self, source: &Self) -> &mut Self {
        self.clear(false);
        if source.is_empty() {
            return self;
        }
        self.resize(source.len() - 1);
        for (first, second) in source.items() {
            self.copy_native_pair(first, second);
        }
        self
    }
    fn copy_native_pair(&mut self, first: &A, second: &B) {
        // OCCT Assign copies locals, then constructs the node base (key2) before key1.
        let local_first = first.clone();
        let local_second = second.clone();
        let node_second = local_second.clone();
        let node_first = local_first.clone();
        self.insert_pair(node_first, node_second);
    }
}
impl<A: Debug + Clone, B: Debug + Clone, H1: HashPolicy<A> + Clone, H2: HashPolicy<B> + Clone> Clone
    for DoubleMap<A, B, H1, H2>
{
    fn clone(&self) -> Self {
        let mut copy = Self::with_allocator(
            self.first_policy.clone(),
            self.second_policy.clone(),
            self.bucket_count,
            self.resource.clone(),
        );
        copy.assign(self);
        copy
    }
    fn clone_from(&mut self, source: &Self) {
        self.assign(source);
    }
}
