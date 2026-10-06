//! Native policy swaps finish before the actual node graph changes owners.
use super::*;
impl<K, V, H: HashPolicy<K>, N: StorageNode<K, V>> OrderedTable<K, V, H, N> {
    pub fn exchange_with(&mut self, other: &mut Self, swap_policy: impl FnOnce(&mut H, &mut H)) {
        swap_policy(&mut self.hasher, &mut other.hasher);
        self.exchange_graph(other);
    }
}
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
