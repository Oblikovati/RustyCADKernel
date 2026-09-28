// Copyright (c) 2025 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of ModelingData/TKG3d/GeomHash/GeomHash_PlaneHasher.pxx, _CylindricalSurfaceHasher.pxx, _ConicalSurfaceHasher.pxx, _SphericalSurfaceHasher.pxx, _ToroidalSurfaceHasher.pxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Hashers of the elementary surfaces: the `Position().Ax2()` placement hash combined
//! with the rounded radii (or radius and semi-angle), and a placement comparison plus
//! the same values within tolerance.
//!
//! `gp_Ax3::Ax2()` reverses the main direction of a left-handed frame, so a surface
//! and its mirror image with the same axis can hash alike.
use super::conic_hashers::{placement_and_values, placement_and_values_equal};
use super::placement_hashers::GeomHashAxisPlacement;
use crate::geom::{
    GeomConicalSurface, GeomCylindricalSurface, GeomElementarySurface, GeomPlane,
    GeomSphericalSurface, GeomToroidalSurface,
};
use rustycad_tkg2d::hasher_tolerances;
use rustycad_tkmath::gp::Axis2;

hasher_tolerances!(
    /// `GeomHash_PlaneHasher`: a plane by its placement;
    /// `GeomHashPlaneHasher::default().hash(&plane)`.
    GeomHashPlaneHasher
);
hasher_tolerances!(
    /// `GeomHash_CylindricalSurfaceHasher`: a cylinder by placement and radius;
    /// `GeomHashCylindricalSurfaceHasher::default().hash(&cylinder)`.
    GeomHashCylindricalSurfaceHasher
);
hasher_tolerances!(
    /// `GeomHash_ConicalSurfaceHasher`: a cone by placement, reference radius and
    /// semi-angle; `GeomHashConicalSurfaceHasher::default().hash(&cone)`.
    GeomHashConicalSurfaceHasher
);
hasher_tolerances!(
    /// `GeomHash_SphericalSurfaceHasher`: a sphere by placement and radius;
    /// `GeomHashSphericalSurfaceHasher::default().hash(&sphere)`.
    GeomHashSphericalSurfaceHasher
);
hasher_tolerances!(
    /// `GeomHash_ToroidalSurfaceHasher`: a torus by placement and both radii;
    /// `GeomHashToroidalSurfaceHasher::default().hash(&torus)`.
    GeomHashToroidalSurfaceHasher
);

/// `Position().Ax2()` of an elementary surface.
///
/// A `gp_Ax3` keeps orthonormal X and Y directions, so the cross product never fails;
/// if it did, the native exception would escape a `noexcept` operator and terminate
/// the process, which the panic mirrors; `frame_of(&plane)`.
fn frame_of(surface: &impl GeomElementarySurface) -> Axis2 {
    surface
        .position()
        .ax2()
        .expect("gp_Ax3 keeps orthonormal X and Y directions; expected Ax2() to succeed")
}

// The hash and comparison of `frame_of` plus the given values, for one tolerance pair.
fn elementary_hash(comp: f64, tol: f64, frame: Axis2, values: &[f64]) -> usize {
    placement_and_values(GeomHashAxisPlacement::new(comp, tol), frame, values)
}
fn elementary_equal(comp: f64, tol: f64, frames: [Axis2; 2], values: &[(f64, f64)]) -> bool {
    placement_and_values_equal(GeomHashAxisPlacement::new(comp, tol), frames, values)
}

impl GeomHashPlaneHasher {
    /// The placement hash alone (`operator()(handle)`); `hasher.hash(&plane)`.
    pub fn hash(&self, plane: &GeomPlane) -> usize {
        GeomHashAxisPlacement::new(self.comp_tolerance, self.hash_tolerance).hash(frame_of(plane))
    }
    /// Placements equal (`operator()(handle, handle)`); `hasher.equal(&a, &b)`.
    pub fn equal(&self, first: &GeomPlane, second: &GeomPlane) -> bool {
        let axes = GeomHashAxisPlacement::new(self.comp_tolerance, self.hash_tolerance);
        axes.equal(frame_of(first), frame_of(second))
    }
}

impl GeomHashCylindricalSurfaceHasher {
    /// Placement and rounded radius (`operator()(handle)`); `hasher.hash(&cylinder)`.
    pub fn hash(&self, cylinder: &GeomCylindricalSurface) -> usize {
        let (comp, tol) = (self.comp_tolerance, self.hash_tolerance);
        elementary_hash(comp, tol, frame_of(cylinder), &[cylinder.radius()])
    }
    /// Placements and radii equal (`operator()(handle, handle)`); `hasher.equal(&a, &b)`.
    pub fn equal(&self, first: &GeomCylindricalSurface, second: &GeomCylindricalSurface) -> bool {
        let (comp, tol) = (self.comp_tolerance, self.hash_tolerance);
        let frames = [frame_of(first), frame_of(second)];
        elementary_equal(comp, tol, frames, &[(first.radius(), second.radius())])
    }
}

impl GeomHashConicalSurfaceHasher {
    /// Placement, rounded reference radius and rounded semi-angle (`operator()(handle)`);
    /// `hasher.hash(&cone)`.
    pub fn hash(&self, cone: &GeomConicalSurface) -> usize {
        let (comp, tol) = (self.comp_tolerance, self.hash_tolerance);
        let values = [cone.ref_radius(), cone.semi_angle()];
        elementary_hash(comp, tol, frame_of(cone), &values)
    }
    /// Placements, radii and semi-angles equal (`operator()(handle, handle)`);
    /// `hasher.equal(&a, &b)`.
    pub fn equal(&self, first: &GeomConicalSurface, second: &GeomConicalSurface) -> bool {
        let (comp, tol) = (self.comp_tolerance, self.hash_tolerance);
        let frames = [frame_of(first), frame_of(second)];
        let values = [
            (first.ref_radius(), second.ref_radius()),
            (first.semi_angle(), second.semi_angle()),
        ];
        elementary_equal(comp, tol, frames, &values)
    }
}

impl GeomHashSphericalSurfaceHasher {
    /// Placement and rounded radius (`operator()(handle)`); `hasher.hash(&sphere)`.
    pub fn hash(&self, sphere: &GeomSphericalSurface) -> usize {
        let (comp, tol) = (self.comp_tolerance, self.hash_tolerance);
        elementary_hash(comp, tol, frame_of(sphere), &[sphere.radius()])
    }
    /// Placements and radii equal (`operator()(handle, handle)`); `hasher.equal(&a, &b)`.
    pub fn equal(&self, first: &GeomSphericalSurface, second: &GeomSphericalSurface) -> bool {
        let (comp, tol) = (self.comp_tolerance, self.hash_tolerance);
        let frames = [frame_of(first), frame_of(second)];
        elementary_equal(comp, tol, frames, &[(first.radius(), second.radius())])
    }
}

impl GeomHashToroidalSurfaceHasher {
    /// Placement and rounded major and minor radii (`operator()(handle)`);
    /// `hasher.hash(&torus)`.
    pub fn hash(&self, torus: &GeomToroidalSurface) -> usize {
        let (comp, tol) = (self.comp_tolerance, self.hash_tolerance);
        let radii = [torus.major_radius(), torus.minor_radius()];
        elementary_hash(comp, tol, frame_of(torus), &radii)
    }
    /// Placements and both radii equal (`operator()(handle, handle)`);
    /// `hasher.equal(&a, &b)`.
    pub fn equal(&self, first: &GeomToroidalSurface, second: &GeomToroidalSurface) -> bool {
        let (comp, tol) = (self.comp_tolerance, self.hash_tolerance);
        let frames = [frame_of(first), frame_of(second)];
        let radii = [
            (first.major_radius(), second.major_radius()),
            (first.minor_radius(), second.minor_radius()),
        ];
        elementary_equal(comp, tol, frames, &radii)
    }
}
