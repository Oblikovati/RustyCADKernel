//! Native Clear borrows its replacement handle until all node destruction completes.
use super::*;
impl<K, V, H, N: StorageNode<K, V>> OrderedTable<K, V, H, N> {
    pub fn clear_with_allocator(&mut self, resource: Option<BaseAllocator>) {
        if self.clear_allocator_nodes(resource.as_ref()) {
            self.base = BaseMapStorage::new(self.bucket_count, resource);
        }
    }
    pub fn clear_with_borrowed_allocator(&mut self, resource: Option<&BaseAllocator>) {
        if self.clear_allocator_nodes(resource) {
            // OCCT 3d097a0 assigns its allocator after Clear returns, never during payload Drop.
            self.base = BaseMapStorage::new(self.bucket_count, resource.cloned());
        }
    }
    fn clear_allocator_nodes(&mut self, resource: Option<&BaseAllocator>) -> bool {
        let release = resource != Some(self.resource());
        self.clear(release);
        release
    }
}
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
