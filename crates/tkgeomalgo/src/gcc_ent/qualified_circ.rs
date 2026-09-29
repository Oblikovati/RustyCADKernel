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
// Rust translation of GccEnt_QualifiedCirc.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! GccEnt_QualifiedCirc: a 2d circle with a qualifier.
use super::Position;
use rustycad_tkmath::gp::Circle2;

/// GccEnt_QualifiedCirc: a circle argument of a construction algorithm and the
/// position its solutions must have relative to it.
/// ```
/// use rustycad_tkgeomalgo::gcc_ent::{Position, QualifiedCirc};
/// use rustycad_tkmath::gp::Circle2;
/// let qualified = QualifiedCirc::new(Circle2::default(), Position::Outside);
/// assert!(qualified.is_outside());
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct QualifiedCirc {
    the_qualified: Circle2,
    the_qualifier: Position,
}

impl QualifiedCirc {
    /// GccEnt_QualifiedCirc(Qualified, Qualifier);
    /// `QualifiedCirc::new(circle, Position::Enclosing)`.
    pub fn new(qualified: Circle2, qualifier: Position) -> Self {
        Self {
            the_qualified: qualified,
            the_qualifier: qualifier,
        }
    }

    /// Qualified: the circle; `qualified.qualified().radius()`.
    pub fn qualified(&self) -> Circle2 {
        self.the_qualified
    }

    /// Qualifier: the stored position. The native documentation promises
    /// GccEnt_noqualifier for an unqualified circle, but the code returns the stored
    /// GccEnt_unqualified, as this port does; `qualified.qualifier()`.
    pub fn qualifier(&self) -> Position {
        self.the_qualifier
    }

    /// IsUnqualified: true when all solutions apply; `qualified.is_unqualified()`.
    pub fn is_unqualified(&self) -> bool {
        self.the_qualifier == Position::Unqualified
    }

    /// IsEnclosing: the solution encloses the circle; `qualified.is_enclosing()`.
    pub fn is_enclosing(&self) -> bool {
        self.the_qualifier == Position::Enclosing
    }

    /// IsEnclosed: the solution is enclosed by the circle; `qualified.is_enclosed()`.
    pub fn is_enclosed(&self) -> bool {
        self.the_qualifier == Position::Enclosed
    }

    /// IsOutside: the solution and the circle are outside each other;
    /// `qualified.is_outside()`.
    pub fn is_outside(&self) -> bool {
        self.the_qualifier == Position::Outside
    }
}
