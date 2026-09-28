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
// Rust translation of ModelingData/TKG3d/GeomHash at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The GeomHash package: hashers for curve, surface and mesh deduplication.
//!
//! Each hasher holds `CompTolerance` (equality) and `HashTolerance` (the rounding step
//! of hashed values); the geometry hashers default to Precision::Angular() and
//! Precision::Confusion(), the mesh hashers to Precision::Computational() for both.
//! The hash values reproduce OCCT bit for bit on the pinned 64-bit profile, because
//! they decide the iteration order of the hashed containers that use them. The hash
//! expressions and the exact-type dispatch are shared with Geom2dHash
//! (`rustycad_tkg2d::geom2d_hash::{hash_key, class_dispatch}`).
//!
//! `operator()(key)` is `hash` and `operator()(key, key)` is `equal`; every hasher also
//! implements the NCollection `HashPolicy`. The hashers of one geometry class take the
//! object, as the Geom2dHash port does; the polymorphic and mesh hashers take handles,
//! which may be null.
mod placement_hashers;
pub use placement_hashers::{
    GeomHashAxisPlacement, GeomHashDirectionHasher, GeomHashPointHasher, GeomHashVectorHasher,
};
mod conic_hashers;
pub use conic_hashers::{
    GeomHashCircleHasher, GeomHashEllipseHasher, GeomHashHyperbolaHasher, GeomHashLineHasher,
    GeomHashParabolaHasher,
};
mod spline_hashers;
pub use spline_hashers::{GeomHashBSplineCurveHasher, GeomHashBezierCurveHasher};
mod wrapped_hashers;
pub use wrapped_hashers::{GeomHashOffsetCurveHasher, GeomHashTrimmedCurveHasher};
mod curve_hasher;
pub use curve_hasher::GeomHashCurveHasher;
mod elementary_hashers;
pub use elementary_hashers::{
    GeomHashConicalSurfaceHasher, GeomHashCylindricalSurfaceHasher, GeomHashPlaneHasher,
    GeomHashSphericalSurfaceHasher, GeomHashToroidalSurfaceHasher,
};
mod swept_hashers;
pub use swept_hashers::{
    GeomHashSurfaceOfLinearExtrusionHasher, GeomHashSurfaceOfRevolutionHasher,
};
mod spline_surface_hashers;
pub use spline_surface_hashers::{GeomHashBSplineSurfaceHasher, GeomHashBezierSurfaceHasher};
mod wrapped_surface_hashers;
pub use wrapped_surface_hashers::{
    GeomHashOffsetSurfaceHasher, GeomHashRectangularTrimmedSurfaceHasher,
};
mod surface_hasher;
pub use surface_hasher::GeomHashSurfaceHasher;
mod polygon_hashers;
pub use polygon_hashers::{GeomHashPolygon2DHasher, GeomHashPolygon3DHasher};
mod polygon_on_tri_hasher;
pub use polygon_on_tri_hasher::{GeomHashPolygonOnTriHasher, PolygonOnTriHashKey};
mod triangulation_hasher;
pub use triangulation_hasher::GeomHashTriangulationHasher;
