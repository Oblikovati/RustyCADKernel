//! Constructor, exchange and clear variants shared by OrderedMap and OrderedDataMap as macros:
//! each native overload keeps its policy-callback order around the graph transfer.
use super::*;
impl<K, V, H> OrderedTable<K, V, H> {
    /// Native move constructor: the empty base, the moved policy, then the graph.
    pub fn move_from_with(source: &mut Self, take_policy: impl FnOnce(&mut H) -> H) -> Self {
        let buckets = source.bucket_count();
        let resource = source.resource().clone();
        let mut moved =
            Self::new_with_policy(|| take_policy(&mut source.hasher), buckets, resource);
        moved.exchange_graph(source);
        moved
    }
    /// Native `Exchange`: policies swap first, then the graphs.
    pub fn exchange_with(&mut self, other: &mut Self, swap_policy: impl FnOnce(&mut H, &mut H)) {
        swap_policy(&mut self.hasher, &mut other.hasher);
        self.exchange_graph(other);
    }
    /// Native `Clear(theAllocator)` with a borrowed handle: the allocator is adopted only after
    /// every entry is dropped (OCCT 3d097a0).
    pub fn clear_with_borrowed_allocator(&mut self, resource: Option<&BaseAllocator>) {
        self.reset_order();
        self.storage.clear_with_borrowed_allocator(resource);
    }
}
macro_rules! ordered_base_bridge {
    ($map:ident<$($parameter:ident),+>) => {
        impl<$($parameter),+> crate::ncollection::BaseMap for $map<$($parameter),+> {
            fn nb_buckets(&self)->usize { self.table.bucket_count() }
            fn size(&self)->usize { self.table.len() }
            fn allocator(&self)->&crate::ncollection::BaseAllocator { self.table.resource() }
        }
    };
}
pub(in crate::ncollection) use ordered_base_bridge;
macro_rules! ordered_policy_exchange {
    ($map:ident<$key:ident $(,$parameter:ident)*>,$hasher:ident)=>{
        impl<$key,$($parameter),*> $map<$key,$($parameter),*> where $hasher:crate::ncollection::HashPolicy<$key> {
            /// Swaps native policies before existing graph ownership; `map.exchange_with(&mut other, native_swap)`.
            /// A callback panic preserves both graphs and retains any partial policy mutations.
            pub fn exchange_with(&mut self,other:&mut Self,swap_policy:impl FnOnce(&mut $hasher,&mut $hasher)) {
                self.table.exchange_with(&mut other.table,swap_policy);
            }
            /// Preserves native Exchange(self) callbacks without aliased borrows; `map.exchange_self_with(native_self_swap)`.
            /// Unlike move assignment to self, native self Exchange still performs the policy swap.
            pub fn exchange_self_with(&mut self,swap_policy:impl FnOnce(&mut $hasher)) {
                swap_policy(&mut self.table.hasher);
            }
            /// Completes native policy swapping then exchanges both graphs without clearing; `map.move_assign_with(&mut source,native_swap)`.
            pub fn move_assign_with(&mut self,source:&mut Self,swap_policy:impl FnOnce(&mut $hasher,&mut $hasher))->&mut Self {
                self.exchange_with(source,swap_policy);
                self
            }
        }
    };
}
pub(in crate::ncollection) use ordered_policy_exchange;
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
macro_rules! ordered_borrowed_clear {
    ($map:ident<$($parameter:ident),+>)=>{
        impl<$($parameter),+> $map<$($parameter),+> {
            /// Borrows the replacement while destroying nodes; `map.clear_with_borrowed_allocator(Some(&arena))`.
            /// `None` selects the common allocator after clearing; equal allocators retain buckets.
            /// Rust payload panic recovery retains the old allocator; native noexcept destruction terminates.
            pub fn clear_with_borrowed_allocator(&mut self,resource:Option<&crate::ncollection::BaseAllocator>) {
                self.table.clear_with_borrowed_allocator(resource);
            }
        }
    };
}
pub(in crate::ncollection) use ordered_borrowed_clear;
