//! Constructor progress retains native completed nodes when later construction fails.
use super::*;
impl<K: Clone, V: Clone, H: HashPolicy<K> + Clone, N: StorageNode<K, V>> OrderedTable<K, V, H, N> {
    pub fn clone_with(&self, replace_values: bool) -> Self {
        let mut copy = Self::new_with_policy(
            || self.hasher.clone(),
            self.bucket_count,
            self.resource().clone(),
        );
        copy.incomplete_copy = true;
        copy.assign(self, replace_values);
        copy.incomplete_copy = false;
        copy
    }
}
impl<K, V, H, N: StorageNode<K, V>> OrderedTable<K, V, H, N> {
    pub(super) fn abandon_incomplete_copy(&mut self) {
        for _node in self.slots.iter().flatten() {
            // SAFETY: completed nodes are uniquely owned here. Native constructor unwind
            // retains their allocation/payloads; no safe reference survives this failed map.
            unsafe {
                self.resource().relinquish_typed_to_raw();
            }
        }
        // Keep the bucket allocation until the base drops, after the hasher destructor.
        self.base.bucket_slots_mut().fill(std::ptr::null_mut());
        self.base.finish_destroy(false);
        self.count = 0;
        self.first = None;
        self.last = None;
    }
}
macro_rules! ordered_default_construction {
    ($map:ident<$key:ident $(,$parameter:ident)*>,$hasher:ident)=>{
        impl<$key,$($parameter),*> $map<$key,$($parameter),*> where $hasher:crate::ncollection::HashPolicy<$key>+Default {
            /// Creates the empty base before defaulting its policy; `Map::with_default_allocator(1, arena)`.
            pub fn with_default_allocator(buckets:usize,resource:crate::ncollection::BaseAllocator)->Self {
                Self {table:OrderedTable::new_with_policy($hasher::default,buckets,resource)}
            }
            /// Validates before constructing base or policy; `Map::with_default_allocator_int(-1, arena)`.
            pub fn with_default_allocator_int(buckets:i32,resource:crate::ncollection::BaseAllocator)->Result<Self,crate::standard::Failure> {
                Ok(Self::with_default_allocator(buckets_from_int(buckets)?,resource))
            }
            /// Uses the common allocator after signed validation; `Map::new_int(0)?`.
            pub fn new_int(buckets:i32)->Result<Self,crate::standard::Failure> {
                Ok(Self::new(buckets_from_int(buckets)?))
            }
        }
    };
}
pub(in crate::ncollection) use ordered_default_construction;
