use super::{Debug, DoubleMap, DoubleMapItems, DoubleMapValues, HashPolicy};
impl<A: Debug, B: Debug, H1: HashPolicy<A>, H2: HashPolicy<B>> DoubleMap<A, B, H1, H2> {
    /// Borrows both keys in first-key bucket order; `for (first,second) in map.items() {}`.
    pub fn items(&self) -> DoubleMapItems<'_, A, B> {
        DoubleMapItems {
            source: Some(&self.storage),
            position: super::super::owned_bucket_chains::BucketChainPosition::new(
                self.storage.base().base(),
            ),
        }
    }

    /// Borrows second keys in first-key bucket order; `map.iter().count()`.
    pub fn iter(&self) -> DoubleMapValues<'_, A, B> {
        DoubleMapValues {
            pairs: self.items(),
        }
    }
    /// Returns an exhausted value iterator; `map.iter()!=map.end()`.
    pub fn end(&self) -> DoubleMapValues<'_, A, B> {
        DoubleMapValues {
            pairs: DoubleMapItems::default(),
        }
    }
}

impl<'a, A, B> DoubleMapItems<'a, A, B> {
    pub(super) fn current(&self) -> Option<&'a super::DoubleMapNode<A, B>> {
        if self.position.node.is_null() {
            return None;
        }
        Some(unsafe { self.source?.node(self.position.node) })
    }
    pub(super) fn reset(&mut self) {
        self.position = self.source.map_or_else(Default::default, |source| {
            super::super::owned_bucket_chains::BucketChainPosition::new(source.base().base())
        });
    }
}
impl<'a, A, B> Iterator for DoubleMapItems<'a, A, B> {
    type Item = (&'a A, &'a B);
    fn next(&mut self) -> Option<Self::Item> {
        let node = self.current()?;
        self.position.advance(self.source?.base().base());
        Some((node.key1(), node.key2()))
    }
}
impl<A, B> Clone for DoubleMapItems<'_, A, B> {
    fn clone(&self) -> Self {
        Self {
            source: self.source,
            position: self.position,
        }
    }
}
impl<A, B> Default for DoubleMapItems<'_, A, B> {
    fn default() -> Self {
        Self {
            source: None,
            position: Default::default(),
        }
    }
}
// SAFETY: these readonly ranges borrow an immutable owner and expose only shared payload references.
unsafe impl<A: Sync, B: Sync> Send for DoubleMapItems<'_, A, B> {}
unsafe impl<A: Sync, B: Sync> Sync for DoubleMapItems<'_, A, B> {}
impl<A, B> PartialEq for DoubleMapItems<'_, A, B> {
    fn eq(&self, other: &Self) -> bool {
        self.position.node == other.position.node
    }
}
impl<A, B> Eq for DoubleMapItems<'_, A, B> {}
impl<'a, A, B> Iterator for DoubleMapValues<'a, A, B> {
    type Item = &'a B;
    fn next(&mut self) -> Option<Self::Item> {
        self.pairs.next().map(|(_, second)| second)
    }
}
impl<A, B> Clone for DoubleMapValues<'_, A, B> {
    fn clone(&self) -> Self {
        Self {
            pairs: self.pairs.clone(),
        }
    }
}
impl<A, B> PartialEq for DoubleMapValues<'_, A, B> {
    fn eq(&self, other: &Self) -> bool {
        self.pairs == other.pairs
    }
}
impl<A, B> Eq for DoubleMapValues<'_, A, B> {}
impl<'a, A: Debug, B: Debug, H1: HashPolicy<A>, H2: HashPolicy<B>> IntoIterator
    for &'a DoubleMap<A, B, H1, H2>
{
    type Item = &'a B;
    type IntoIter = DoubleMapValues<'a, A, B>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
