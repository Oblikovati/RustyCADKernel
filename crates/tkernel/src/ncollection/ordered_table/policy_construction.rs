//! Native constructor policy callbacks run after the empty base and before graph transfer.
use super::*;
impl<K, V, H: HashPolicy<K>, N: StorageNode<K, V>> OrderedTable<K, V, H, N> {
    pub fn move_from_with(source: &mut Self, take_policy: impl FnOnce(&mut H) -> H) -> Self {
        let buckets = source.bucket_count;
        let resource = source.resource.clone();
        let mut moved =
            Self::new_with_policy(|| take_policy(&mut source.hasher), buckets, resource);
        moved.exchange_graph(source);
        moved
    }
    pub(super) fn exchange_graph(&mut self, other: &mut Self) {
        // OCCT 3d097a0 moves its policy first; swapping only graph ownership keeps the donor policy.
        std::mem::swap(&mut self.slots, &mut other.slots);
        std::mem::swap(&mut self.free, &mut other.free);
        std::mem::swap(&mut self.identities, &mut other.identities);
        std::mem::swap(&mut self.base, &mut other.base);
        std::mem::swap(&mut self.count, &mut other.count);
        std::mem::swap(&mut self.bucket_count, &mut other.bucket_count);
        std::mem::swap(&mut self.first, &mut other.first);
        std::mem::swap(&mut self.last, &mut other.last);
        std::mem::swap(&mut self.resource, &mut other.resource);
    }
}
macro_rules! ordered_policy_construction {
    ($map:ident<$key:ident $(,$parameter:ident)*>, $hasher:ident) => {
        impl<$key,$($parameter),*> $map<$key,$($parameter),*> where $hasher:crate::ncollection::HashPolicy<$key> {
            /// Copies a borrowed policy after constructing the empty base; `Map::with_borrowed_hasher(&policy, 1, arena)`.
            pub fn with_borrowed_hasher(hasher:&$hasher,buckets:usize,resource:crate::ncollection::BaseAllocator)->Self where $hasher:Clone {
                Self { table: OrderedTable::new_with_policy(||hasher.clone(),buckets,resource) }
            }
            /// Validates the signed count before copying the policy; `Map::with_borrowed_hasher_int(&policy, -1, arena)`.
            pub fn with_borrowed_hasher_int(hasher:&$hasher,buckets:i32,resource:crate::ncollection::BaseAllocator)->Result<Self,crate::standard::Failure> where $hasher:Clone {
                Ok(Self::with_borrowed_hasher(hasher,buckets_from_int(buckets)?,resource))
            }
            /// Runs an injected native move constructor without requiring Clone or Default; `Map::with_moved_hasher(&mut policy, 1, arena, take_policy)`.
            pub fn with_moved_hasher(hasher:&mut $hasher,buckets:usize,resource:crate::ncollection::BaseAllocator,take_policy:impl FnOnce(&mut $hasher)->$hasher)->Self {
                Self { table: OrderedTable::new_with_policy(||take_policy(hasher),buckets,resource) }
            }
            /// Validates before moving, retaining the donor on negative counts; `Map::with_moved_hasher_int(&mut policy, -1, arena, take_policy)`.
            pub fn with_moved_hasher_int(hasher:&mut $hasher,buckets:i32,resource:crate::ncollection::BaseAllocator,take_policy:impl FnOnce(&mut $hasher)->$hasher)->Result<Self,crate::standard::Failure> {
                Ok(Self::with_moved_hasher(hasher,buckets_from_int(buckets)?,resource,take_policy))
            }
            /// Moves the policy then transfers existing nodes and buckets; `Map::move_from_with(&mut source, take_policy)`.
            /// The callback defines native moved-from state; if it panics, the source graph remains owned.
            pub fn move_from_with(source:&mut Self,take_policy:impl FnOnce(&mut $hasher)->$hasher)->Self {
                Self { table: OrderedTable::move_from_with(&mut source.table,take_policy) }
            }
        }
    };
}
pub(in crate::ncollection) use ordered_policy_construction;
