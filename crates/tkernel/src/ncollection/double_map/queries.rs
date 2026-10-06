//! `AreBound`, the output-parameter finds and the allocator-replacing `Clear`.
use super::super::BaseAllocator;
use super::{Debug, DoubleMap, HashPolicy};

impl<A: Debug, B: Debug, H1: HashPolicy<A>, H2: HashPolicy<B>> DoubleMap<A, B, H1, H2> {
    /// `AreBound`: true when both keys are bound to the same pair, as the native walk finds it.
    ///
    /// Upstream quirk: after locating the first key, the native code scans the second key's
    /// bucket by following each node's *first-key* chain link (`Next()`, not `Next2()`), so a
    /// bound pair that is not reachable that way reports false. The walk is reproduced exactly.
    ///
    /// ```
    /// # use rustycad_tkernel::ncollection::DoubleMap;
    /// let mut map = DoubleMap::<i32, i32>::default();
    /// map.bind(1, 5).unwrap();
    /// map.bind(2, 106).unwrap(); // 106 shares 5's second-key bucket among 101 buckets
    /// assert!(map.are_bound(&2, &106));
    /// assert!(!map.are_bound(&1, &5)); // native answer, although (1, 5) is bound
    /// ```
    pub fn are_bound(&self, first: &A, second: &B) -> bool {
        if self.is_empty() {
            return false;
        }
        let one = self.first_policy.hash(first) % self.bucket_count() + 1;
        let two = self.second_policy.hash(second) % self.bucket_count() + 1;
        let Some(found) = self.match_first(one, first) else {
            return false;
        };
        self.match_second_through_primary(two, second) == Some(found)
    }
    fn match_second_through_primary(
        &self,
        bucket: usize,
        second: &B,
    ) -> Option<*mut super::MapLinkNode> {
        let mut node = self.storage.head(bucket, true);
        while !node.is_null() {
            if self
                .second_policy
                .equal(unsafe { self.storage.node(node) }.key2(), second)
            {
                return Some(node);
            }
            node = unsafe { (*node).next() };
        }
        None
    }

    /// `Find1(key1, key2&)`: copies the partner of a bound first key into `output`.
    ///
    /// ```
    /// # use rustycad_tkernel::ncollection::DoubleMap;
    /// let mut map = DoubleMap::<i32, i32>::default();
    /// map.bind(1, 5).unwrap();
    /// let mut out = 0;
    /// assert!(map.find1_into(&1, &mut out) && out == 5);
    /// ```
    pub fn find1_into(&self, key: &A, output: &mut B) -> bool
    where
        B: Clone,
    {
        let Some(found) = self.seek1(key) else {
            return false;
        };
        output.clone_from(found);
        true
    }

    /// `Find2(key2, key1&)`: copies the partner of a bound second key into `output`.
    ///
    /// ```
    /// # use rustycad_tkernel::ncollection::DoubleMap;
    /// let mut map = DoubleMap::<i32, i32>::default();
    /// map.bind(1, 5).unwrap();
    /// let mut out = 0;
    /// assert!(map.find2_into(&5, &mut out) && out == 1);
    /// ```
    pub fn find2_into(&self, key: &B, output: &mut A) -> bool
    where
        A: Clone,
    {
        let Some(found) = self.seek2(key) else {
            return false;
        };
        output.clone_from(found);
        true
    }

    /// `Clear(const Handle(NCollection_BaseAllocator)&)`: unlike the other maps, DoubleMap
    /// always releases its buckets; `None` (a null handle) selects the common allocator.
    ///
    /// ```
    /// # use rustycad_tkernel::ncollection::{BaseAllocator, DoubleMap};
    /// let mut map = DoubleMap::<i32, i32>::default();
    /// map.bind(1, 5).unwrap();
    /// map.clear_with_allocator(None);
    /// assert!(map.is_empty() && map.allocator() == &BaseAllocator::common());
    /// ```
    pub fn clear_with_allocator(&mut self, resource: Option<BaseAllocator>) {
        self.storage.clear_with_allocator(resource);
    }
}
