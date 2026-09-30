// Rust translation of ModelingData/TKG3d/GeomHash/GeomHash_TrimmedCurveHasher.pxx, GeomHash_OffsetCurveHasher.pxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Hashers of the curves that wrap a basis curve: the basis goes through the
//! polymorphic [`GeomHashCurveHasher`], then the trim bounds or the offset distance
//! and direction are rounded (hash) or compared within tolerance (equality).
use super::curve_hasher::GeomHashCurveHasher;
use super::placement_hashers::GeomHashDirectionHasher;
use crate::geom::{GeomCurve, GeomOffsetCurve, GeomTrimmedCurve};
use rustycad_tkg2d::geom2d_hash::hash_key::{
    combined_hash, leading_and_rounded, rounded_hash, values_within,
};
use rustycad_tkg2d::hasher_tolerances;

hasher_tolerances!(
    /// `GeomHash_TrimmedCurveHasher`: a trimmed curve by basis curve and bounds;
    /// `GeomHashTrimmedCurveHasher::default().hash(&trimmed)`.
    GeomHashTrimmedCurveHasher
);
hasher_tolerances!(
    /// `GeomHash_OffsetCurveHasher`: an offset curve by basis curve, distance and
    /// reference direction; `GeomHashOffsetCurveHasher::default().hash(&offset)`.
    GeomHashOffsetCurveHasher
);

impl GeomHashTrimmedCurveHasher {
    fn curves(&self) -> GeomHashCurveHasher {
        GeomHashCurveHasher::new(self.comp_tolerance, self.hash_tolerance)
    }
    /// Basis hash and rounded bounds (`operator()(handle)`); `hasher.hash(&trimmed)`.
    pub fn hash(&self, curve: &GeomTrimmedCurve) -> usize {
        let basis_hash: usize = self.curves().hash(&curve.basis_curve());
        let bounds = [curve.first_parameter(), curve.last_parameter()];
        leading_and_rounded(&[basis_hash], &bounds, self.hash_tolerance)
    }
    /// Equal basis curves and bounds (`operator()(handle, handle)`); `hasher.equal(&a, &b)`.
    pub fn equal(&self, first: &GeomTrimmedCurve, second: &GeomTrimmedCurve) -> bool {
        let bounds = [
            (first.first_parameter(), second.first_parameter()),
            (first.last_parameter(), second.last_parameter()),
        ];
        self.curves()
            .equal(&first.basis_curve(), &second.basis_curve())
            && values_within(self.comp_tolerance, &bounds)
    }
}

impl GeomHashOffsetCurveHasher {
    fn parts(&self) -> (GeomHashCurveHasher, GeomHashDirectionHasher) {
        (
            GeomHashCurveHasher::new(self.comp_tolerance, self.hash_tolerance),
            GeomHashDirectionHasher::new(self.comp_tolerance, self.hash_tolerance),
        )
    }
    /// Basis hash, rounded offset and direction hash, in that order (`operator()(handle)`);
    /// `hasher.hash(&offset)`.
    pub fn hash(&self, curve: &GeomOffsetCurve) -> usize {
        let (curves, directions) = self.parts();
        let factor: f64 = 1.0 / self.hash_tolerance;
        combined_hash(&[
            curves.hash(&curve.basis_curve()),
            rounded_hash(curve.offset(), factor),
            directions.hash(curve.direction()),
        ])
    }
    /// Equal basis curves, offsets and directions (`operator()(handle, handle)`);
    /// `hasher.equal(&a, &b)`.
    pub fn equal(&self, first: &GeomOffsetCurve, second: &GeomOffsetCurve) -> bool {
        let (curves, directions) = self.parts();
        curves.equal(&first.basis_curve(), &second.basis_curve())
            && values_within(self.comp_tolerance, &[(first.offset(), second.offset())])
            && directions.equal(first.direction(), second.direction())
    }
}
