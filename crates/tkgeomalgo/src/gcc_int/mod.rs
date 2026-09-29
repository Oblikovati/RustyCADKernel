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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/GccInt (GccInt_IType.hxx, GccInt_Bisec.hxx/.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package GccInt: the elementary bisecting loci (lines, circles, ellipses,
//! hyperbolas, parabolas and points) that the GccAna bisector algorithms return.
//!
//! `GccInt_Bisec` is an abstract transient class; it is the trait [`Bisec`],
//! reached through `Handle<dyn Bisec>` as docs/geom-design.md fixes for abstract
//! classes, and each concrete bisector is a struct with plain fields (none of them
//! has a mutator).
mod bisectors;
pub use bisectors::{BCirc, BElips, BHyper, BLine, BParab};

use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::standard::{Failure, StandardTransient, Transient};
use rustycad_tkmath::gp::{Circle2, Ellipse2, Hyperbola2, Line2, Parabola2, Point2};

/// GccInt_IType: the kind of a bisecting locus; `IType::Cir as i32 == 1`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IType {
    /// GccInt_Lin: a line.
    Lin,
    /// GccInt_Cir: a circle.
    Cir,
    /// GccInt_Ell: an ellipse.
    Ell,
    /// GccInt_Par: a parabola.
    Par,
    /// GccInt_Hpr: a hyperbola.
    Hpr,
    /// GccInt_Pnt: a point.
    Pnt,
}

// Every accessor of the base class raises Standard_NotImplemented natively (the
// header documents a DomainError, the code throws Standard_NotImplemented).
fn not_implemented(member: &str) -> Failure {
    Failure::with_type(
        "Standard_NotImplemented",
        &format!("GccInt_Bisec::{member}: not provided by this bisector type"),
        "",
    )
}

/// GccInt_Bisec: an elementary bisecting locus between two points, lines or
/// circles. [`Bisec::arc_type`] tells which accessor holds its geometry; the other
/// accessors fail with Standard_NotImplemented.
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkgeomalgo::gcc_int::{BLine, Bisec, IType};
/// use rustycad_tkmath::gp::Line2;
/// let bisector: Handle<dyn Bisec> = Handle::new(BLine::new(Line2::default())).cast();
/// assert_eq!(bisector.arc_type(), IType::Lin);
/// assert!(bisector.circle().is_err());
/// ```
pub trait Bisec: Transient {
    /// ArcType: the type of the locus.
    fn arc_type(&self) -> IType;
    /// Point: the locus when it is a point.
    fn point(&self) -> Result<Point2, Failure> {
        Err(not_implemented("Point"))
    }
    /// Line: the locus when it is a line.
    fn line(&self) -> Result<Line2, Failure> {
        Err(not_implemented("Line"))
    }
    /// Circle: the locus when it is a circle.
    fn circle(&self) -> Result<Circle2, Failure> {
        Err(not_implemented("Circle"))
    }
    /// Hyperbola: the locus when it is a hyperbola.
    fn hyperbola(&self) -> Result<Hyperbola2, Failure> {
        Err(not_implemented("Hyperbola"))
    }
    /// Parabola: the locus when it is a parabola.
    fn parabola(&self) -> Result<Parabola2, Failure> {
        Err(not_implemented("Parabola"))
    }
    /// Ellipse: the locus when it is an ellipse.
    fn ellipse(&self) -> Result<Ellipse2, Failure> {
        Err(not_implemented("Ellipse"))
    }
}
impl_transient_interface!(dyn Bisec, "GccInt_Bisec", StandardTransient);
