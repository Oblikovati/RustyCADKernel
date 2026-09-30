use super::DegreeOfFreedom;
use rustycad_tkernel::ncollection::HashKey;
/// Triangle made of three oriented links; `BRepMeshTriangle::default()`.
#[derive(Clone, Copy, Debug, Default)]
pub struct BRepMeshTriangle {
    /// Native public edge indices.
    pub edges: [i32; 3],
    /// Native public edge orientations.
    pub orientations: [bool; 3],
    /// Native public constraint flag.
    pub movability: DegreeOfFreedom,
}
impl BRepMeshTriangle {
    /// Retain link order and orientation; `BRepMeshTriangle::new([1,2,3],[true;3],DegreeOfFreedom::Free)`.
    pub fn new(edges: [i32; 3], orientations: [bool; 3], movability: DegreeOfFreedom) -> Self {
        Self {
            edges,
            orientations,
            movability,
        }
    }
    /// Replace all fields; `triangle.initialize([1,2,3],[false;3],DegreeOfFreedom::Fixed)`.
    pub fn initialize(
        &mut self,
        edges: [i32; 3],
        orientations: [bool; 3],
        movability: DegreeOfFreedom,
    ) {
        *self = Self::new(edges, orientations, movability);
    }
    /// Copy links and their orientations; `triangle.edges()`.
    pub fn edges(&self) -> ([i32; 3], [bool; 3]) {
        (self.edges, self.orientations)
    }
    /// Read the native constraint flag; `triangle.movability()`.
    pub fn movability(&self) -> DegreeOfFreedom {
        self.movability
    }
    /// Replace the constraint flag; `triangle.set_movability(DegreeOfFreedom::Deleted)`.
    pub fn set_movability(&mut self, movability: DegreeOfFreedom) {
        self.movability = movability;
    }
    /// Compare cyclic edge order, ignoring orientations and excluding deletion; `triangle.is_equal(&other)`.
    pub fn is_equal(&self, other: &Self) -> bool {
        if self.movability == DegreeOfFreedom::Deleted
            || other.movability == DegreeOfFreedom::Deleted
        {
            return false;
        }
        (0..3).any(|shift| (0..3).all(|i| self.edges[i] == other.edges[(i + shift) % 3]))
    }
}
impl PartialEq for BRepMeshTriangle {
    fn eq(&self, other: &Self) -> bool {
        self.is_equal(other)
    }
}
impl HashKey for BRepMeshTriangle {
    fn occt_hash(&self) -> usize {
        let mut sorted = self.edges;
        sorted.sort_unstable();
        let mut bytes = [0u8; 12];
        for (index, edge) in sorted.into_iter().enumerate() {
            bytes[index * 4..index * 4 + 4].copy_from_slice(&edge.to_ne_bytes());
        }
        rustycad_tkernel::standard::hash_bytes(&bytes)
    }
}
