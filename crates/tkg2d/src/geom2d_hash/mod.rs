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
// Rust translation of ModelingData/TKG2d/Geom2dHash at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The Geom2dHash package: hashers for geometry deduplication.
//!
//! Each hasher holds `CompTolerance` (equality, default Precision::Angular()) and
//! `HashTolerance` (the rounding step of hashed values, default Precision::Confusion()).
//! The hash values reproduce OCCT bit for bit on the pinned 64-bit profile, because
//! they decide the iteration order of the hashed containers that use them.

/// Declares a hasher struct with its two public tolerances, `new` and the native
/// default arguments (`Precision::Angular()`, `Precision::Confusion()`).
macro_rules! hasher_tolerances {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct $name {
            /// Tolerance of the equality comparison (`CompTolerance`).
            pub comp_tolerance: f64,
            /// Rounding step of hashed values (`HashTolerance`).
            pub hash_tolerance: f64,
        }
        impl $name {
            /// The hasher with explicit tolerances (the native constructor);
            #[doc = concat!("`", stringify!($name), "::new(1e-12, 1e-7)`.")]
            pub fn new(comp_tolerance: f64, hash_tolerance: f64) -> Self {
                Self { comp_tolerance, hash_tolerance }
            }
        }
        impl Default for $name {
            fn default() -> Self {
                Self::new(
                    rustycad_tkernel::precision::angular(),
                    rustycad_tkernel::precision::confusion(),
                )
            }
        }
    };
}

mod hash_key;

mod placement_hashers;
pub use placement_hashers::{
    Geom2dHashAxisPlacement, Geom2dHashDirectionHasher, Geom2dHashPointHasher,
};
mod conic_hashers;
pub use conic_hashers::{
    Geom2dHashCircleHasher, Geom2dHashEllipseHasher, Geom2dHashHyperbolaHasher,
    Geom2dHashLineHasher, Geom2dHashParabolaHasher,
};
mod spline_hashers;
pub use spline_hashers::{Geom2dHashBSplineCurveHasher, Geom2dHashBezierCurveHasher};
mod wrapped_hashers;
pub use wrapped_hashers::{Geom2dHashOffsetCurveHasher, Geom2dHashTrimmedCurveHasher};
mod curve_hasher;
pub use curve_hasher::Geom2dHashCurveHasher;
