//! Protected native lookup writes its output node before invoking equality (OCCT 3d097a0).
use super::*;
impl<K, V, H: HashPolicy<K>, N: StorageNode<K, V>> OrderedTable<K, V, H, N> {
    pub fn lookup_node(&self, key: &K, node: &mut *mut N) -> bool {
        if self.count == 0 {
            return false;
        }
        self.lookup_nodes_at_hash(key, node, self.bounded_hash(key, self.bucket_count))
    }
    pub fn lookup_node_with_hash(&self, key: &K, node: &mut *mut N, hash: &mut usize) -> bool {
        *hash = self.bounded_hash(key, self.bucket_count);
        if self.count == 0 {
            return false;
        }
        self.lookup_nodes_at_hash(key, node, *hash)
    }
    pub fn keys_equal(&self, left: &K, right: &K) -> bool {
        self.hasher.equal(left, right)
    }
    pub fn bounded_hash(&self, key: &K, upper: usize) -> usize {
        assert_ne!(upper, 0, "hash upper bound 0; expected a positive modulus");
        self.hasher.hash(key) % upper + 1
    }
    pub(super) fn lookup_nodes_at_hash(&self, key: &K, node: &mut *mut N, hash: usize) -> bool {
        *node = self.base.bucket_slots()[hash].cast();
        while !node.is_null() {
            // SAFETY: only live owned graph nodes are assigned here; no incoming pointer is read.
            if self.keys_equal(unsafe { &*N::key((*node).cast()) }, key) {
                return true;
            }
            // Native stores this successor in the caller output, including a terminal null.
            *node = unsafe { (*(*node).cast::<MapLinkNode>()).next().cast() };
        }
        false
    }
}
macro_rules! ordered_protected_lookup {
    ($map:ident<$key:ident $(,$parameter:ident)*>,$hasher:ident,$node:ty)=>{
        impl<$key,$($parameter),*> $map<$key,$($parameter),*> where $hasher:crate::ncollection::HashPolicy<$key> {
            /// Preserves output on empty maps and hash failure; `map.lookup_node(&key, &mut node)`.
            /// Returns a raw identity only; dereference requires the original map's lifetime and aliasing rules.
            pub fn lookup_node(&self,key:&$key,node:&mut *mut $node)->bool {
                self.table.lookup_node(key,node)
            }
            /// Hashes even when empty, then walks actual nodes; `map.lookup_node_with_hash(&key, &mut node, &mut hash)`.
            /// Equality failure leaves `node` at the failing candidate; raw identity grants no mutable access.
            pub fn lookup_node_with_hash(&self,key:&$key,node:&mut *mut $node,hash:&mut usize)->bool {
                self.table.lookup_node_with_hash(key,node,hash)
            }
            /// Invokes the native equality policy once in argument order; `map.keys_equal(&left, &right)`.
            pub fn keys_equal(&self,left:&$key,right:&$key)->bool {
                self.table.keys_equal(left,right)
            }
            /// Computes native one-based hash for a positive modulus; `map.bounded_hash(&key, 101)`.
            /// A zero modulus panics because the native remainder operation has no defined result.
            pub fn bounded_hash(&self,key:&$key,upper:usize)->usize {
                self.table.bounded_hash(key,upper)
            }
        }
    };
}
pub(in crate::ncollection) use ordered_protected_lookup;
