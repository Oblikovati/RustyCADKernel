// Copyright (c) 1993-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
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
// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_Curve.hxx (the constant overrides that Geom2d, Geom, Geom2dEval and GeomEval curves repeat) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The one-line virtual overrides that many curve classes write identically:
//! the parameter range of an open curve and the infinite continuity of an
//! analytic one. The macros expand to members of a `Geom2dCurve` or `GeomCurve`
//! implementation; the signatures of both traits are the same for these members.

/// `FirstParameter`, `LastParameter` and `IsClosed` of an open curve on
/// `[first, last]`: `IsClosed` is always false;
/// `rustycad_tkg2d::open_curve_range!(-precision::infinite(), precision::infinite());`.
#[macro_export]
macro_rules! open_curve_range {
    ($first:expr, $last:expr) => {
        fn first_parameter(&self) -> f64 {
            $first
        }
        fn last_parameter(&self) -> f64 {
            $last
        }
        fn is_closed(&self) -> ::core::result::Result<bool, ::rustycad_tkernel::standard::Failure> {
            Ok(false)
        }
    };
}

/// `IsPeriodic`, `Continuity` and `IsCN` of a non-periodic curve that is CN
/// everywhere (lines, parabolas, hyperbolas, Bezier and eval curves);
/// `rustycad_tkg2d::non_periodic_cn_curve!();`.
#[macro_export]
macro_rules! non_periodic_cn_curve {
    () => {
        fn is_periodic(&self) -> bool {
            false
        }
        fn continuity(&self) -> ::rustycad_tkmath::geom_abs::Shape {
            ::rustycad_tkmath::geom_abs::Shape::CN
        }
        fn is_cn(
            &self,
            _n: i32,
        ) -> ::core::result::Result<bool, ::rustycad_tkernel::standard::Failure> {
            Ok(true)
        }
    };
}

/// `Reverse` and `ReversedParameter` of an eval curve, which keeps its defining
/// parameterization: both raise the package's Standard_NotImplemented through
/// `$unsupported(member, detail)`;
/// `rustycad_tkg2d::unsupported_curve_reversal!(unsupported, "Geom2dEval_SineWaveCurve");`.
#[macro_export]
macro_rules! unsupported_curve_reversal {
    ($unsupported:path, $class:literal) => {
        fn reverse(&self) -> ::core::result::Result<(), ::rustycad_tkernel::standard::Failure> {
            Err($unsupported(concat!($class, "::Reverse"), ""))
        }
        fn reversed_parameter(
            &self,
            u: f64,
        ) -> ::core::result::Result<f64, ::rustycad_tkernel::standard::Failure> {
            let detail: String = format!("(U = {u})");
            Err($unsupported(
                concat!($class, "::ReversedParameter"),
                &detail,
            ))
        }
    };
}
