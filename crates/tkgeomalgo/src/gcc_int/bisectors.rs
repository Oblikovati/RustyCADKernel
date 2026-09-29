// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of GccInt_BCirc, GccInt_BElips, GccInt_BHyper, GccInt_BLine and GccInt_BParab
// (.hxx/.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The concrete bisecting loci of GccInt: one struct per native class, each
//! overriding `ArcType` and the one accessor that returns its geometry.
use super::{Bisec, IType};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, StandardTransient};
use rustycad_tkmath::gp::{Circle2, Ellipse2, Hyperbola2, Line2, Parabola2};

// One native bisector class: its struct, RTTI and the Bisec overrides.
macro_rules! bisector_class {
    (
        $(#[$doc:meta])*
        $class:ident, $native:literal, $geometry:ty, $field:ident, $kind:ident, $accessor:ident
    ) => {
        $(#[$doc])*
        #[derive(Default)]
        pub struct $class {
            parent: StandardTransient,
            $field: $geometry,
        }
        impl_transient!($class, $native, dyn Bisec, parent, interfaces: [dyn Bisec]);
        impl $class {
            #[doc = concat!(
                "The ", $native, " constructor: a bisecting locus of geometry `value`; ",
                "`", stringify!($class), "::new(value)`."
            )]
            pub fn new(value: $geometry) -> Self {
                Self {
                    parent: StandardTransient::default(),
                    $field: value,
                }
            }
        }
        impl Bisec for $class {
            fn arc_type(&self) -> IType {
                IType::$kind
            }
            fn $accessor(&self) -> Result<$geometry, Failure> {
                Ok(self.$field)
            }
        }
    };
}

bisector_class!(
    /// GccInt_BCirc: a circle as the bisecting locus of two circles or points;
    /// `BCirc::new(circle).circle()`.
    BCirc, "GccInt_BCirc", Circle2, cir, Cir, circle
);
bisector_class!(
    /// GccInt_BElips: an ellipse as the bisecting locus of two circles;
    /// `BElips::new(ellipse).ellipse()`.
    BElips, "GccInt_BElips", Ellipse2, eli, Ell, ellipse
);
bisector_class!(
    /// GccInt_BHyper: a hyperbola as the bisecting locus of two circles;
    /// `BHyper::new(hyperbola).hyperbola()`.
    BHyper, "GccInt_BHyper", Hyperbola2, hyp, Hpr, hyperbola
);
bisector_class!(
    /// GccInt_BLine: a line as the bisecting locus of two lines or points;
    /// `BLine::new(line).line()`.
    BLine, "GccInt_BLine", Line2, lin, Lin, line
);
bisector_class!(
    /// GccInt_BParab: a parabola as the bisecting locus of a line and a point or
    /// a circle; `BParab::new(parabola).parabola()`.
    BParab, "GccInt_BParab", Parabola2, par, Par, parabola
);
