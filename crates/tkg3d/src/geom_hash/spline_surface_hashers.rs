// Rust translation of ModelingData/TKG3d/GeomHash/GeomHash_BezierSurfaceHasher.pxx, GeomHash_BSplineSurfaceHasher.pxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Bezier and B-spline surface hashers: the hash covers only metadata (degrees, pole
//! counts, knot counts, the two rationality flags packed into one integer), while the
//! comparison walks the full geometric data row by row.
//!
//! As for the curves, the native comparisons read the second surface with the first
//! surface's pole counts inside a `noexcept` operator; a pair whose second surface has
//! fewer poles would terminate the process, and the port reports it as different.
use super::placement_hashers::GeomHashPointHasher;
use super::spline_hashers::{PoleAccess, WeightAccess, same_net};
use crate::geom::{GeomBSplineSurface, GeomBezierSurface};
use rustycad_tkg2d::geom2d_hash::hash_key::{KnotAccess, combined_hash, int_hash, same_knots};
use rustycad_tkg2d::hasher_tolerances;

hasher_tolerances!(
    /// `GeomHash_BezierSurfaceHasher`: metadata hash, full comparison;
    /// `GeomHashBezierSurfaceHasher::default().equal(&a, &b)`.
    GeomHashBezierSurfaceHasher
);
hasher_tolerances!(
    /// `GeomHash_BSplineSurfaceHasher`: metadata hash, full comparison;
    /// `GeomHashBSplineSurfaceHasher::default().equal(&a, &b)`.
    GeomHashBSplineSurfaceHasher
);

/// `static_cast<int>(IsURational()) | (static_cast<int>(IsVRational()) << 1)`;
/// `rational_flags(true, false) == 1`.
fn rational_flags(u_rational: bool, v_rational: bool) -> i32 {
    i32::from(u_rational) | (i32::from(v_rational) << 1)
}

/// The `(i, j)` indices of an `nu` x `nv` net in the native loop order (U outer).
fn net_indices(nu: i32, nv: i32) -> Vec<(i32, i32)> {
    (1..=nu)
        .flat_map(|i| (1..=nv).map(move |j| (i, j)))
        .collect()
}

impl GeomHashBezierSurfaceHasher {
    /// Degrees, pole counts and rationality flags hashed (`operator()(handle)`);
    /// `hasher.hash(&surface)`.
    pub fn hash(&self, surface: &GeomBezierSurface) -> usize {
        combined_hash(&[
            int_hash(surface.u_degree()),
            int_hash(surface.v_degree()),
            int_hash(surface.nb_u_poles()),
            int_hash(surface.nb_v_poles()),
            int_hash(rational_flags(
                surface.is_u_rational(),
                surface.is_v_rational(),
            )),
        ])
    }
    /// Same degrees and rationality, poles and weights within tolerance
    /// (`operator()(handle, handle)`); `hasher.equal(&a, &b)`.
    pub fn equal(&self, first: &GeomBezierSurface, second: &GeomBezierSurface) -> bool {
        if first.u_degree() != second.u_degree() || first.v_degree() != second.v_degree() {
            return false;
        }
        if first.is_u_rational() != second.is_u_rational()
            || first.is_v_rational() != second.is_v_rational()
        {
            return false;
        }
        let points = GeomHashPointHasher::new(self.comp_tolerance, self.hash_tolerance);
        let indices = net_indices(first.nb_u_poles(), first.nb_v_poles());
        let poles: [PoleAccess<'_, (i32, i32)>; 2] =
            [&|(i, j)| first.pole(i, j), &|(i, j)| second.pole(i, j)];
        let weights: [WeightAccess<'_, (i32, i32)>; 2] =
            [&|(i, j)| first.weight(i, j), &|(i, j)| second.weight(i, j)];
        let rational: bool = first.is_u_rational() || first.is_v_rational();
        same_net(points, &indices, rational, poles, weights)
    }
}

impl GeomHashBSplineSurfaceHasher {
    /// Degrees, pole counts, knot counts and rationality flags hashed
    /// (`operator()(handle)`); `hasher.hash(&surface)`.
    pub fn hash(&self, surface: &GeomBSplineSurface) -> usize {
        combined_hash(&[
            int_hash(surface.u_degree()),
            int_hash(surface.v_degree()),
            int_hash(surface.nb_u_poles()),
            int_hash(surface.nb_v_poles()),
            int_hash(surface.nb_u_knots()),
            int_hash(surface.nb_v_knots()),
            int_hash(rational_flags(
                surface.is_u_rational(),
                surface.is_v_rational(),
            )),
        ])
    }
    /// Same degrees, knots and multiplicities in U then V, rationality, then poles and
    /// weights within tolerance (`operator()(handle, handle)`); `hasher.equal(&a, &b)`.
    pub fn equal(&self, first: &GeomBSplineSurface, second: &GeomBSplineSurface) -> bool {
        if first.u_degree() != second.u_degree() || first.v_degree() != second.v_degree() {
            return false;
        }
        if first.nb_u_knots() != second.nb_u_knots() || first.nb_v_knots() != second.nb_v_knots() {
            return false;
        }
        if !self.same_knot_vectors(first, second) {
            return false;
        }
        if first.is_u_rational() != second.is_u_rational()
            || first.is_v_rational() != second.is_v_rational()
        {
            return false;
        }
        let points = GeomHashPointHasher::new(self.comp_tolerance, self.hash_tolerance);
        let indices = net_indices(first.nb_u_poles(), first.nb_v_poles());
        let poles: [PoleAccess<'_, (i32, i32)>; 2] =
            [&|(i, j)| first.pole(i, j), &|(i, j)| second.pole(i, j)];
        let weights: [WeightAccess<'_, (i32, i32)>; 2] =
            [&|(i, j)| first.weight(i, j), &|(i, j)| second.weight(i, j)];
        let rational: bool = first.is_u_rational() || first.is_v_rational();
        same_net(points, &indices, rational, poles, weights)
    }
    // The U knots, then the V knots, each within CompTolerance with equal multiplicities.
    fn same_knot_vectors(&self, first: &GeomBSplineSurface, second: &GeomBSplineSurface) -> bool {
        let u_knots: [KnotAccess<'_>; 2] = [
            (&|i| first.u_knot(i), &|i| first.u_multiplicity(i)),
            (&|i| second.u_knot(i), &|i| second.u_multiplicity(i)),
        ];
        let v_knots: [KnotAccess<'_>; 2] = [
            (&|i| first.v_knot(i), &|i| first.v_multiplicity(i)),
            (&|i| second.v_knot(i), &|i| second.v_multiplicity(i)),
        ];
        let comp: f64 = self.comp_tolerance;
        same_knots(comp, first.nb_u_knots(), u_knots).unwrap_or(false)
            && same_knots(comp, first.nb_v_knots(), v_knots).unwrap_or(false)
    }
}
