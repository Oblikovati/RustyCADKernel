//! Both ordered families expose the same native base without copying buckets or nodes.
use super::*;
impl<K, V, H, N: StorageNode<K, V>> OrderedTable<K, V, H, N> {
    pub fn base_storage(&self) -> &BaseMapStorage {
        &self.base
    }
    pub fn base_storage_mut(&mut self) -> &mut BaseMapStorage {
        &mut self.base
    }
    pub fn first_node_pointer(&self) -> *mut MapLinkNode {
        self.first
            .map_or(std::ptr::null_mut(), |slot| self.node_pointer(slot))
    }
}
macro_rules! ordered_base_bridge {
    ($map:ident<$($parameter:ident),+>,$node:ty) => {
        impl<$($parameter),+> crate::ncollection::BaseMap for $map<$($parameter),+> {
            fn nb_buckets(&self)->usize { self.table.bucket_count }
            fn size(&self)->usize { self.table.count }
            fn allocator(&self)->&crate::ncollection::BaseAllocator { &self.table.resource }
        }
        impl<$($parameter),+> crate::ncollection::BasicMapIteratorSource for $map<$($parameter),+> {
            type Node=$node;
            fn basic_bucket_slots(&self)->Option<&[*mut crate::ncollection::MapLinkNode]> {
                let base=self.table.base_storage();
                (!base.buckets().is_null()).then(||base.bucket_slots())
            }
            fn basic_first_node(&self)->*mut Self::Node {
                self.table.first_node_pointer().cast()
            }
        }
        impl<$($parameter),+> $map<$($parameter),+> {
            /// Returns native const Buckets over the actual graph; `map.buckets().is_null()`.
            pub fn buckets(&self)->*const *mut crate::ncollection::MapLinkNode {
                self.table.base_storage().buckets()
            }
            /// Returns native mutable Buckets; `map.buckets_mut()`.
            /// Unsafe writes must preserve the existing nodes, acyclic ownership, order links,
            /// hashes, counts and cursor identity registry; no foreign nodes may be inserted.
            pub fn buckets_mut(&mut self)->*mut *mut crate::ncollection::MapLinkNode {
                self.table.base_storage_mut().buckets_mut()
            }
        }
    };
}
pub(in crate::ncollection) use ordered_base_bridge;
