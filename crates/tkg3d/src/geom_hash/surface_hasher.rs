// Rust translation of ModelingData/TKG3d/GeomHash/GeomHash_SurfaceHasher.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomHash_SurfaceHasher`: the polymorphic hasher that dispatches on the exact
//! dynamic type of a `Geom_Surface` to the typed hashers.
use super::elementary_hashers::{
    GeomHashConicalSurfaceHasher, GeomHashCylindricalSurfaceHasher, GeomHashPlaneHasher,
    GeomHashSphericalSurfaceHasher, GeomHashToroidalSurfaceHasher,
};
use super::spline_surface_hashers::{GeomHashBSplineSurfaceHasher, GeomHashBezierSurfaceHasher};
use super::swept_hashers::{
    GeomHashSurfaceOfLinearExtrusionHasher, GeomHashSurfaceOfRevolutionHasher,
};
use super::wrapped_surface_hashers::{
    GeomHashOffsetSurfaceHasher, GeomHashRectangularTrimmedSurfaceHasher,
};
use crate::geom::{
    GeomBSplineSurface, GeomBezierSurface, GeomConicalSurface, GeomCylindricalSurface,
    GeomOffsetSurface, GeomPlane, GeomRectangularTrimmedSurface, GeomSphericalSurface, GeomSurface,
    GeomSurfaceOfLinearExtrusion, GeomSurfaceOfRevolution, GeomToroidalSurface,
};
use rustycad_tkernel::ncollection::HashPolicy;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d_hash::class_dispatch::{decided_equality, unknown_class_hash};
use rustycad_tkg2d::{hasher_class_dispatch, hasher_tolerances};

hasher_tolerances!(
    /// `GeomHash_SurfaceHasher`: hashes and compares any `Geom_Surface` handle, for
    /// geometry deduplication in hashed containers; a null handle hashes to 0;
    /// `GeomHashSurfaceHasher::default().hash(&surface)`.
    GeomHashSurfaceHasher
);

impl GeomHashSurfaceHasher {
    /// The typed hash of the surface's exact class, 0 for a null handle, and the
    /// Standard_CStringHasher hash of the type name for any other class, such as the
    /// GeomEval surfaces (`operator()(const occ::handle<Geom_Surface>&)`);
    /// `hasher.hash(&surface)`.
    pub fn hash(&self, surface: &Handle<dyn GeomSurface>) -> usize {
        let (comp, tol) = (self.comp_tolerance, self.hash_tolerance);
        hasher_class_dispatch!(hash surface, comp, tol;
            GeomPlane => GeomHashPlaneHasher,
            GeomCylindricalSurface => GeomHashCylindricalSurfaceHasher,
            GeomConicalSurface => GeomHashConicalSurfaceHasher,
            GeomSphericalSurface => GeomHashSphericalSurfaceHasher,
            GeomToroidalSurface => GeomHashToroidalSurfaceHasher,
            GeomSurfaceOfRevolution => GeomHashSurfaceOfRevolutionHasher,
            GeomSurfaceOfLinearExtrusion => GeomHashSurfaceOfLinearExtrusionHasher,
            GeomBezierSurface => GeomHashBezierSurfaceHasher,
            GeomBSplineSurface => GeomHashBSplineSurfaceHasher,
            GeomRectangularTrimmedSurface => GeomHashRectangularTrimmedSurfaceHasher,
            GeomOffsetSurface => GeomHashOffsetSurfaceHasher);
        unknown_class_hash(surface)
    }

    /// Whether two surfaces are equal for deduplication (`operator()(handle, handle)`):
    /// both null, the same object, or the same exact class compared by its typed
    /// hasher; objects of other classes compare by identity; `hasher.equal(&a, &b)`.
    pub fn equal(&self, first: &Handle<dyn GeomSurface>, second: &Handle<dyn GeomSurface>) -> bool {
        if let Some(decided) = decided_equality(first, second) {
            return decided;
        }
        let (comp, tol) = (self.comp_tolerance, self.hash_tolerance);
        hasher_class_dispatch!(equal first, second, comp, tol;
            GeomPlane => GeomHashPlaneHasher,
            GeomCylindricalSurface => GeomHashCylindricalSurfaceHasher,
            GeomConicalSurface => GeomHashConicalSurfaceHasher,
            GeomSphericalSurface => GeomHashSphericalSurfaceHasher,
            GeomToroidalSurface => GeomHashToroidalSurfaceHasher,
            GeomSurfaceOfRevolution => GeomHashSurfaceOfRevolutionHasher,
            GeomSurfaceOfLinearExtrusion => GeomHashSurfaceOfLinearExtrusionHasher,
            GeomBezierSurface => GeomHashBezierSurfaceHasher,
            GeomBSplineSurface => GeomHashBSplineSurfaceHasher,
            GeomRectangularTrimmedSurface => GeomHashRectangularTrimmedSurfaceHasher,
            GeomOffsetSurface => GeomHashOffsetSurfaceHasher);
        // Other classes compare by object identity, already found different.
        false
    }
}

impl HashPolicy<Handle<dyn GeomSurface>> for GeomHashSurfaceHasher {
    fn hash(&self, key: &Handle<dyn GeomSurface>) -> usize {
        GeomHashSurfaceHasher::hash(self, key)
    }
    fn equal(&self, first: &Handle<dyn GeomSurface>, second: &Handle<dyn GeomSurface>) -> bool {
        GeomHashSurfaceHasher::equal(self, first, second)
    }
}
