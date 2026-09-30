// Rust translation of ModelingData/TKG3d/GeomHash/GeomHash_RectangularTrimmedSurfaceHasher.pxx, GeomHash_OffsetSurfaceHasher.pxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Hashers of the surfaces that wrap a basis surface: the basis goes through the
//! polymorphic [`GeomHashSurfaceHasher`], then the trim bounds or the offset distance
//! are rounded (hash) or compared within tolerance (equality).
use super::surface_hasher::GeomHashSurfaceHasher;
use crate::geom::{GeomOffsetSurface, GeomRectangularTrimmedSurface, GeomSurface};
use rustycad_tkg2d::geom2d_hash::hash_key::{leading_and_rounded, values_within};
use rustycad_tkg2d::hasher_tolerances;

hasher_tolerances!(
    /// `GeomHash_RectangularTrimmedSurfaceHasher`: a trimmed surface by basis surface
    /// and parametric bounds;
    /// `GeomHashRectangularTrimmedSurfaceHasher::default().hash(&trimmed)`.
    GeomHashRectangularTrimmedSurfaceHasher
);
hasher_tolerances!(
    /// `GeomHash_OffsetSurfaceHasher`: an offset surface by basis surface and distance;
    /// `GeomHashOffsetSurfaceHasher::default().hash(&offset)`.
    GeomHashOffsetSurfaceHasher
);

impl GeomHashRectangularTrimmedSurfaceHasher {
    fn surfaces(&self) -> GeomHashSurfaceHasher {
        GeomHashSurfaceHasher::new(self.comp_tolerance, self.hash_tolerance)
    }
    /// Basis hash and the rounded `Bounds` U1, U2, V1, V2 (`operator()(handle)`);
    /// `hasher.hash(&trimmed)`.
    pub fn hash(&self, surface: &GeomRectangularTrimmedSurface) -> usize {
        let basis_hash: usize = self.surfaces().hash(&surface.basis_surface());
        let (u1, u2, v1, v2) = surface.bounds();
        leading_and_rounded(&[basis_hash], &[u1, u2, v1, v2], self.hash_tolerance)
    }
    /// Equal basis surfaces, then the four bounds within tolerance
    /// (`operator()(handle, handle)`); `hasher.equal(&a, &b)`.
    pub fn equal(
        &self,
        first: &GeomRectangularTrimmedSurface,
        second: &GeomRectangularTrimmedSurface,
    ) -> bool {
        if !self
            .surfaces()
            .equal(&first.basis_surface(), &second.basis_surface())
        {
            return false;
        }
        let (u11, u21, v11, v21) = first.bounds();
        let (u12, u22, v12, v22) = second.bounds();
        let bounds = [(u11, u12), (u21, u22), (v11, v12), (v21, v22)];
        values_within(self.comp_tolerance, &bounds)
    }
}

impl GeomHashOffsetSurfaceHasher {
    fn surfaces(&self) -> GeomHashSurfaceHasher {
        GeomHashSurfaceHasher::new(self.comp_tolerance, self.hash_tolerance)
    }
    /// Basis hash and rounded offset (`operator()(handle)`); `hasher.hash(&offset)`.
    pub fn hash(&self, surface: &GeomOffsetSurface) -> usize {
        let basis_hash: usize = self.surfaces().hash(&surface.basis_surface());
        leading_and_rounded(&[basis_hash], &[surface.offset()], self.hash_tolerance)
    }
    /// Equal basis surfaces and offsets (`operator()(handle, handle)`);
    /// `hasher.equal(&a, &b)`.
    pub fn equal(&self, first: &GeomOffsetSurface, second: &GeomOffsetSurface) -> bool {
        self.surfaces()
            .equal(&first.basis_surface(), &second.basis_surface())
            && values_within(self.comp_tolerance, &[(first.offset(), second.offset())])
    }
}
