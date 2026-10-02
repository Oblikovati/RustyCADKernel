// BOPDS_Pair.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Pair equality is unordered, while native operator< compares the stored order.
use rustycad_tkernel::ncollection::HashKey;
use rustycad_tkernel::standard::hash_bytes;
/// `BOPDS_Pair`; `Pair::new(1, 2).is_equal(&Pair::new(2, 1))`.
#[derive(Clone, Copy, Debug, Eq)]
pub struct Pair {
    first: i32,
    second: i32,
}
impl Default for Pair {
    fn default() -> Self {
        Self::new(-1, -1)
    }
}
impl Pair {
    /// Stores both indices without sorting; `Pair::new(3, 1)`.
    pub fn new(first: i32, second: i32) -> Self {
        Self { first, second }
    }
    /// Replaces both indices; `pair.set_indices(3, 1)`.
    pub fn set_indices(&mut self, first: i32, second: i32) {
        self.first = first;
        self.second = second;
    }
    /// Returns the stored order; `Pair::new(3, 1).indices() == (3, 1)`.
    pub fn indices(&self) -> (i32, i32) {
        (self.first, self.second)
    }
    /// Tests native lexicographic ordering; `Pair::new(1, 2).is_less(&Pair::new(2, 1))`.
    pub fn is_less(&self, other: &Self) -> bool {
        self.indices() < other.indices()
    }
    /// Compares both possible orientations; `Pair::new(1, 2).is_equal(&Pair::new(2, 1))`.
    pub fn is_equal(&self, other: &Self) -> bool {
        self.indices() == other.indices() || self.indices() == (other.second, other.first)
    }
    /// Hashes the sorted native integer bytes; `pair.hash_code()`.
    pub fn hash_code(&self) -> usize {
        let mut indices = [self.first, self.second];
        indices.sort();
        let bytes: Vec<u8> = indices.into_iter().flat_map(i32::to_ne_bytes).collect();
        hash_bytes(&bytes)
    }
}
impl PartialEq for Pair {
    fn eq(&self, other: &Self) -> bool {
        self.is_equal(other)
    }
}
impl std::hash::Hash for Pair {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(self.hash_code());
    }
}

impl HashKey for Pair {
    fn occt_hash(&self) -> usize {
        self.hash_code()
    }
}
