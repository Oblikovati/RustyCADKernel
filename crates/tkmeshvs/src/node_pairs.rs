// SPDX-License-Identifier: GPL-2.0-only
//! Martensite undirected mesh links preserve stored order while sharing native hash policy.
//!
//! ```
//! use martensite_tkmeshvs::MeshVSTwoNodes;
//! let link = MeshVSTwoNodes::new(8, 3);
//! assert_eq!(link, MeshVSTwoNodes::new(3, 8));
//! assert_eq!(link.first, 8);
//! ```
use rustycad_tkernel::{
    ncollection::{HashKey, HashPolicy},
    standard::hash_bytes,
};

/// Native `std::pair<int,int>` retains ordered tuple equality; use the explicit symmetric policy for links.
pub type MeshVSNodePair = (i32, i32);

/// Native node IDs retain their original orientation; `MeshVSTwoNodes::new(8,3)`.
#[derive(Clone, Copy, Debug, Default)]
pub struct MeshVSTwoNodes {
    /// Original first node; `link.first`.
    pub first: i32,
    /// Original second node; `link.second`.
    pub second: i32,
}
impl MeshVSTwoNodes {
    /// Preserve constructor argument order; `MeshVSTwoNodes::new(5,0)` also models the one-argument native default.
    pub fn new(first: i32, second: i32) -> Self {
        // Sorting belongs only to hashing; callers retain the first inserted orientation.
        Self { first, second }
    }
}
impl PartialEq for MeshVSTwoNodes {
    fn eq(&self, other: &Self) -> bool {
        // Ordinary NodePair tuple equality remains ordered outside this mesh-link policy.
        MeshVSSymmetricPairHasher.equal(&(self.first, self.second), &(other.first, other.second))
    }
}
impl Eq for MeshVSTwoNodes {}
impl HashKey for MeshVSTwoNodes {
    fn occt_hash(&self) -> usize {
        // The same policy must serve native TwoNodes and std::pair map keys.
        MeshVSSymmetricPairHasher.hash(&(self.first, self.second))
    }
}

/// Stateless native pair hash/equality; `Map::<MeshVSNodePair,MeshVSSymmetricPairHasher>::default()`.
#[derive(Clone, Copy, Debug, Default)]
pub struct MeshVSSymmetricPairHasher;
impl HashPolicy<MeshVSNodePair> for MeshVSSymmetricPairHasher {
    fn hash(&self, pair: &MeshVSNodePair) -> usize {
        let (mut first, mut second) = *pair;
        if first > second {
            std::mem::swap(&mut first, &mut second);
        }
        let mut bytes = [0_u8; 8];
        bytes[..4].copy_from_slice(&first.to_ne_bytes());
        bytes[4..].copy_from_slice(&second.to_ne_bytes());
        hash_bytes(&bytes)
    }
    fn equal(&self, first: &MeshVSNodePair, second: &MeshVSNodePair) -> bool {
        // Keep the native direct/reversed comparison without mutating either key.
        (first.0 == second.0 && first.1 == second.1) || (first.0 == second.1 && first.1 == second.0)
    }
}
