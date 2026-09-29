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
// Rust translation of IntRes2d_Transition.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntRes2d_Transition`: the kind of crossing near an intersection point.
use super::failures::domain_error;
use super::{Position, Situation, TypeTrans};
use rustycad_tkernel::standard::Failure;
use std::fmt;

/// `IntRes2d_Transition`: IN or OUT (a true crossing), TOUCH (with the side and the
/// relative orientation) or UNDECIDED (superposed curves), with the position on the
/// curve. Setting an IN/OUT or UNDECIDED value keeps the previous situation and
/// orientation, as the native setters do.
///
/// ```
/// use rustycad_tkgeomalgo::int_res2d::{Position, Transition, TypeTrans};
/// let t = Transition::crossing(false, Position::Middle, TypeTrans::In);
/// assert_eq!(t.transition_type(), TypeTrans::In);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Transition {
    tangent: bool,
    posit: Position,
    typetra: TypeTrans,
    situat: Situation,
    oppos: bool,
}

impl Default for Transition {
    /// `IntRes2d_Transition()`: an undecided tangent transition in the middle.
    /// `Transition::default().transition_type() == TypeTrans::Undecided`.
    fn default() -> Self {
        Self {
            tangent: true,
            posit: Position::Middle,
            typetra: TypeTrans::Undecided,
            situat: Situation::Unknown,
            oppos: false,
        }
    }
}

impl Transition {
    /// `IntRes2d_Transition()`. `Transition::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `IntRes2d_Transition(Tangent, Pos, Type)`: an IN or OUT transition;
    /// `Transition::crossing(false, Position::Head, TypeTrans::Out)`.
    pub fn crossing(tangent: bool, pos: Position, kind: TypeTrans) -> Self {
        Self {
            tangent,
            posit: pos,
            typetra: kind,
            situat: Situation::Unknown,
            oppos: false,
        }
    }
    /// `IntRes2d_Transition(Tangent, Pos, Situ, Oppos)`: a TOUCH transition;
    /// `Transition::touch(true, Position::Middle, Situation::Inside, false)`.
    pub fn touch(tangent: bool, pos: Position, situ: Situation, oppos: bool) -> Self {
        Self {
            tangent,
            posit: pos,
            typetra: TypeTrans::Touch,
            situat: situ,
            oppos,
        }
    }
    /// `IntRes2d_Transition(Pos)`: an UNDECIDED transition;
    /// `Transition::undecided(Position::End)`.
    pub fn undecided(pos: Position) -> Self {
        Self {
            tangent: true,
            posit: pos,
            typetra: TypeTrans::Undecided,
            situat: Situation::Unknown,
            oppos: false,
        }
    }
    /// `SetValue(Tangent, Pos, Type)`: an IN or OUT value;
    /// `t.set_value(false, Position::Middle, TypeTrans::In)`.
    pub fn set_value(&mut self, tangent: bool, pos: Position, kind: TypeTrans) {
        self.tangent = tangent;
        self.posit = pos;
        self.typetra = kind;
    }
    /// `SetValue(Tangent, Pos, Situ, Oppos)`: a TOUCH value;
    /// `t.set_touch_value(true, Position::Middle, Situation::Outside, true)`.
    pub fn set_touch_value(&mut self, tangent: bool, pos: Position, situ: Situation, oppos: bool) {
        self.tangent = tangent;
        self.posit = pos;
        self.typetra = TypeTrans::Touch;
        self.situat = situ;
        self.oppos = oppos;
    }
    /// `SetValue(Pos)`: an UNDECIDED value; `t.set_undecided_value(Position::Head)`.
    pub fn set_undecided_value(&mut self, pos: Position) {
        self.posit = pos;
        self.typetra = TypeTrans::Undecided;
    }
    /// `SetPosition(Pos)`. `t.set_position(Position::End)`.
    pub fn set_position(&mut self, pos: Position) {
        self.posit = pos;
    }
    /// `PositionOnCurve()`: Head, Middle or End. `t.position_on_curve()`.
    pub fn position_on_curve(&self) -> Position {
        self.posit
    }
    /// `TransitionType()`. `t.transition_type()`.
    pub fn transition_type(&self) -> TypeTrans {
        self.typetra
    }
    /// `IsTangent()`: the curves are tangent; Standard_DomainError for an UNDECIDED
    /// transition. `t.is_tangent()?`.
    pub fn is_tangent(&self) -> Result<bool, Failure> {
        if self.typetra == TypeTrans::Undecided {
            return Err(domain_error(
                "IntRes2d_Transition::IsTangent",
                "the transition is Undecided; expected In, Out or Touch",
            ));
        }
        Ok(self.tangent)
    }
    /// `Situation()`: the side of a TOUCH transition; Standard_DomainError for the
    /// other types. `t.situation()?`.
    pub fn situation(&self) -> Result<Situation, Failure> {
        self.check_touch("IntRes2d_Transition::Situation")?;
        Ok(self.situat)
    }
    /// `IsOpposite()`: the curves of a TOUCH transition bound two different parts of
    /// the plane; Standard_DomainError for the other types. `t.is_opposite()?`.
    pub fn is_opposite(&self) -> Result<bool, Failure> {
        self.check_touch("IntRes2d_Transition::IsOpposite")?;
        Ok(self.oppos)
    }

    fn check_touch(&self, member: &str) -> Result<(), Failure> {
        if self.typetra != TypeTrans::Touch {
            return Err(domain_error(
                member,
                &format!("the transition is {:?}; expected Touch", self.typetra),
            ));
        }
        Ok(())
    }
}

/// `operator<<(ostream, IntRes2d_Transition)`: the native French description.
impl fmt::Display for Transition {
    fn fmt(&self, os: &mut fmt::Formatter<'_>) -> fmt::Result {
        let position: &str = match self.posit {
            Position::Head => "Debut",
            Position::Middle => "Milieu",
            Position::End => "Fin",
        };
        write!(os, "   Position : {position}\n   Type de transition : ")?;
        if self.typetra == TypeTrans::Undecided {
            return write!(os, "Indeterminee\n\n");
        }
        match self.typetra {
            TypeTrans::In => writeln!(os, "Entrante")?,
            TypeTrans::Out => writeln!(os, "Sortante")?,
            _ => self.fmt_touch(os)?,
        }
        let tangent: &str = if self.tangent { "Oui" } else { "Non" };
        write!(os, "   Cas de tangence : {tangent}\n\n")
    }
}

impl Transition {
    fn fmt_touch(&self, os: &mut fmt::Formatter<'_>) -> fmt::Result {
        let side: &str = match self.situat {
            Situation::Inside => "Interieure\n",
            Situation::Outside => "Exterieure\n",
            Situation::Unknown => "Indeterminee\n",
        };
        let matter: &str = if self.oppos { "Opposee" } else { "Idem" };
        write!(
            os,
            "Touch\n     Position par rapport a l'autre courbe : {side}   Position matiere : {matter}\n"
        )
    }
}
