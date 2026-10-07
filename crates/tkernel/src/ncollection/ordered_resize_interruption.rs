//! Typed interruption contains native partial graphs while ordinary maps retain safe invariants.
macro_rules! interrupted_resize {
    ($map:ident,$token:ident<$key:ident $(,$value:ident)*;$policy:ident>,$node:ty,$stored:ty)=>{
        /// Owns a failed native rehash; `let (map, panic) = interruption.recover()` repairs it explicitly.
        /// No safe map or payload traversal is exposed while bucket reachability may overlap.
        /// Dropping this token abandons nodes and replacement buckets without K/V callbacks,
        /// then tears down the detached base and policy. This is Rust containment, not native Drop parity.
        pub struct $token<$key,$($value,)*$policy> {
            progress:Box<crate::ncollection::ordered_table::ResizeInterruption<$key,$stored,$policy,$node>>,
        }
        impl<$key,$($value,)*$policy:super::HashPolicy<$key>> $map<$key,$($value,)*$policy> {
            /// Consumes the receiver and captures hash panic progress; `map.resize_with_progress(1000)`.
            /// Success returns the resized map. Failure retains its actual native partial graph.
            pub fn resize_with_progress(self,requested:usize)->Result<Self,$token<$key,$($value,)*$policy>> {
                self.table.resize_with_progress(requested).map(|table|Self{table}).map_err(|progress|$token{progress})
            }
        }
        impl<$key,$($value,)*$policy> $token<$key,$($value,)*$policy> {
            /// Observes the original panic without replacing it; `failure.panic_payload().is::<String>()`.
            pub fn panic_payload(&self)->&(dyn std::any::Any+Send){self.progress.panic_payload()}
            /// Restores original links and releases the replacement without callbacks; `failure.recover()`.
            pub fn recover(self)->($map<$key,$($value,)*$policy>,Box<dyn std::any::Any+Send>) {
                let (table,panic)=self.progress.recover();($map{table},panic)
            }
            /// Runs native Clear only for fully certified disjoint chains; `failure.native_clear(true)`.
            /// A duplicate, cycle or unknown pointer returns the untouched failure. Lost nodes and the
            /// replacement table are intentionally abandoned without payload destruction, as in OCCT.
            pub fn native_clear(self,release:bool)->Result<($map<$key,$($value,)*$policy>,Box<dyn std::any::Any+Send>),Self> {
                self.progress.native_clear(release).map(|(table,panic)|($map{table},panic)).map_err(|progress|Self{progress})
            }
            /// Returns the unchanged native Size, including lost nodes; `failure.len()`.
            pub fn len(&self)->usize{self.progress.table().count}
            /// Reports whether native Size is zero; `failure.is_empty()`.
            pub fn is_empty(&self)->bool{self.len()==0}
            /// Returns the old modulus retained by native failure; `failure.bucket_count()`.
            pub fn bucket_count(&self)->usize{self.progress.table().bucket_count}
            /// Returns the actual old bucket pointer for raw observation; `failure.buckets()`.
            /// Dereferencing requires live-node and aliasing checks; safe payload traversal is withheld.
            pub fn buckets(&self)->*const *mut super::MapLinkNode{self.progress.table().base_storage().buckets()}
            /// Returns the actual replacement pointer; `failure.replacement_buckets()`.
            /// It remains owned until repair or explicit abandonment; writes must preserve ownership.
            pub fn replacement_buckets(&self)->*const *mut super::MapLinkNode{self.progress.replacement().buckets()}
            /// Reports the pending modulus; `failure.replacement_bucket_count()`.
            pub fn replacement_bucket_count(&self)->usize{self.progress.replacement().bucket_count()}
            /// Returns original live node identities, never copied nodes; `failure.node_pointers()`.
            /// Raw dereferences require this token to remain live and unmodified.
            pub fn node_pointers(&self)->Vec<*mut $node>{self.progress.node_pointers()}
            /// Borrows the sole native allocator owner; `failure.allocator().ref_count()`.
            pub fn allocator(&self)->&super::BaseAllocator{crate::ncollection::BaseMap::allocator(self.progress.table().base_storage())}
        }
    };
}
pub(super) use interrupted_resize;
