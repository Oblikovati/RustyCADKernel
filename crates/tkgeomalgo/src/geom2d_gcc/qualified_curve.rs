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
// Rust translation of Geom2dGcc_QualifiedCurve.hxx/.cxx and Geom2dGcc_QCurve.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dGcc_QualifiedCurve` and `Geom2dGcc_QCurve`: a 2d curve with a qualifier.
use super::copy_adaptor;
use crate::gcc_ent::Position;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;

/// `Geom2dGcc_QualifiedCurve`: a curve argument of a construction algorithm and the
/// position its solutions must have relative to it (the interior of a curve is its
/// left-hand side). The curve is a value copy of the given adaptor, as natively.
/// ```
/// use rustycad_tkgeomalgo::gcc_ent::Position;
/// use rustycad_tkgeomalgo::geom2d_gcc::QualifiedCurve;
/// use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
/// use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::{Direction2, Point2};
/// let line: Handle<dyn Geom2dCurve> =
///     Handle::new(Geom2dLine::from_point(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0)?)).cast();
/// let q = QualifiedCurve::new(&Geom2dAdaptorCurve::with_curve(&line)?, Position::Outside)?;
/// assert!(q.is_outside());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct QualifiedCurve {
    the_qualifier: Position,
    the_qualified: Geom2dAdaptorCurve,
}

/// `Geom2dGcc_QCurve`: the qualified curve of the iterative and geometric solvers.
/// Its members and bodies are those of `Geom2dGcc_QualifiedCurve`, so it is the
/// same type (docs/geomalgo-design.md, section 9).
pub type QCurve = QualifiedCurve;

impl QualifiedCurve {
    /// `Geom2dGcc_QualifiedCurve(Curve, Qualifier)` (and `Geom2dGcc_QCurve(Curve,
    /// Qualifier)`): keeps a copy of `curve`; `QualifiedCurve::new(&adaptor, Position::Enclosed)?`.
    pub fn new(curve: &Geom2dAdaptorCurve, qualifier: Position) -> Result<Self, Failure> {
        Ok(Self {
            the_qualified: copy_adaptor(curve)?,
            the_qualifier: qualifier,
        })
    }

    /// `Qualified()`: a copy of the curve; `q.qualified()?.get_type()`.
    pub fn qualified(&self) -> Result<Geom2dAdaptorCurve, Failure> {
        copy_adaptor(&self.the_qualified)
    }

    // The stored curve, where the solvers read the native copy without changing it.
    pub(crate) fn curve(&self) -> &Geom2dAdaptorCurve {
        &self.the_qualified
    }

    /// `Qualifier()`: the stored position; `q.qualifier()`.
    pub fn qualifier(&self) -> Position {
        self.the_qualifier
    }

    /// `IsUnqualified()`: every solution applies; `q.is_unqualified()`.
    pub fn is_unqualified(&self) -> bool {
        self.the_qualifier == Position::Unqualified
    }

    /// `IsEnclosing()`: the solution encloses the curve; `q.is_enclosing()`.
    pub fn is_enclosing(&self) -> bool {
        self.the_qualifier == Position::Enclosing
    }

    /// `IsEnclosed()`: the solution is enclosed by the curve; `q.is_enclosed()`.
    pub fn is_enclosed(&self) -> bool {
        self.the_qualifier == Position::Enclosed
    }

    /// `IsOutside()`: the solution and the curve are outside each other;
    /// `q.is_outside()`.
    pub fn is_outside(&self) -> bool {
        self.the_qualifier == Position::Outside
    }
}
