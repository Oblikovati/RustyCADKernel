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
// Rust translation of Geom2dGcc_Circ2d2TanRadGeo.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dGcc_Circ2d2TanRadGeo`: circles of given radius tangent to a curve and a
//! line, a circle, a point or another curve, whose centres are the intersections of
//! the curves offset by the radius (Geom2dInt).
mod circ_curve;
mod curve_curve;
mod curve_pnt;
mod lin_curve;

use super::QCurve;
use super::circle_solutions::CircleSolutions;
use super::domains::offset_curve;
use crate::gcc_ana::solutions::negative_value;
use crate::gcc_ent::Position;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::adaptor2d::Adaptor2dOffsetCurve;
use rustycad_tkmath::gp::{Circle2, Point2};

/// The largest number of solutions (`aNbSolMAX`).
const NB_SOL_MAX: i32 = 16;

/// `Geom2dGcc_Circ2d2TanRadGeo`: the circles of radius `Radius` tangent to a
/// qualified line, circle or curve and a qualified curve (or a point), at most 16.
/// Each argument is offset by the radius on the sides its qualifier allows and the
/// offsets are intersected: every intersection point is a centre.
/// ```
/// use rustycad_tkgeomalgo::gcc_ent::{Position, QualifiedLin};
/// use rustycad_tkgeomalgo::geom2d_gcc::{Circ2d2TanRadGeo, unqualified};
/// use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
/// use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::{Direction2, Line2, Point2};
/// let line: Handle<dyn Geom2dCurve> =
///     Handle::new(Geom2dLine::from_point(Point2::new(0.0, 0.0), Direction2::new(0.0, 1.0)?)).cast();
/// let q2 = unqualified(&Geom2dAdaptorCurve::with_range(&line, -10.0, 10.0)?)?;
/// let x_axis = QualifiedLin::new(Line2::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0)?), Position::Unqualified);
/// let circ = Circ2d2TanRadGeo::lin_curve(x_axis, &q2, 1.0, 1e-9)?;
/// assert_eq!(circ.nb_solutions(), 4);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Debug)]
pub struct Circ2d2TanRadGeo {
    solutions: CircleSolutions,
}

// The offsets of both arguments allowed by their qualifiers (`cote1`, `cote2`): an
// enclosed argument +R, an outside one -R, an unqualified one both; an enclosing
// argument on either side gives no offset at all, as the native if-chain does.
fn cotes(q1: Position, q2: Position, radius: f64) -> (Vec<f64>, Vec<f64>) {
    let side = |q: Position| -> Vec<f64> {
        match q {
            Position::Enclosed => vec![radius],
            Position::Outside => vec![-radius],
            Position::Unqualified => vec![radius, -radius],
            _ => Vec::new(),
        }
    };
    if q1 == Position::Enclosing || q2 == Position::Enclosing {
        return (Vec::new(), Vec::new());
    }
    (side(q1), side(q2))
}

// `Adaptor2d_OffsetCurve(new Geom2dAdaptor_Curve(Cu), -cote)`: the curve offset to
// the side of the solution centres.
fn offset_of(qualified: &QCurve, cote: f64) -> Result<Adaptor2dOffsetCurve, Failure> {
    offset_curve(qualified.curve(), -cote)
}

impl Circ2d2TanRadGeo {
    // The empty solver of a constructor: GccEnt_BadQualifier then
    // Standard_NegativeValue, as each native constructor checks them.
    fn start(qualifiers_ok: Result<(), Failure>, radius: f64) -> Result<Self, Failure> {
        qualifiers_ok?;
        if radius < 0.0 {
            return Err(negative_value("Geom2dGcc_Circ2d2TanRadGeo", radius));
        }
        Ok(Self {
            solutions: CircleSolutions::new(NB_SOL_MAX as usize, 2),
        })
    }

    // Records solution NbrSol + 1 (circle of centre `center`), its qualifiers and
    // tangencies (TheSame = 0); `tangencies` are (point, ParArg) of both arguments.
    fn push(
        &mut self,
        center: Point2,
        radius: f64,
        qualifiers: (Position, Position),
        tangencies: [(Point2, f64); 2],
    ) -> Result<Circle2, Failure> {
        let s = &mut self.solutions;
        s.nbr_sol += 1;
        let index: i32 = s.nbr_sol;
        let circle: Circle2 = crate::gcc_ana::solutions::circle_at(center, radius)?;
        s.cirsol.set(index, circle)?;
        s.qualifier[0].set(index, qualifiers.0)?;
        s.qualifier[1].set(index, qualifiers.1)?;
        for (arg, (point, par_arg)) in tangencies.into_iter().enumerate() {
            let par_sol: f64 = rustycad_tkmath::elclib::parameter(&circle, point)?;
            s.store_tangency(arg, index, false, (par_sol, par_arg, point))?;
        }
        Ok(circle)
    }

    /// `IsDone()`; `circ.is_done()`.
    pub fn is_done(&self) -> bool {
        self.solutions.well_done
    }

    /// `NbSolutions()`; `circ.nb_solutions()`.
    pub fn nb_solutions(&self) -> i32 {
        self.solutions.nbr_sol
    }

    /// `ThisSolution(Index)`; `circ.this_solution(1)?`.
    pub fn this_solution(&self, index: i32) -> Result<Circle2, Failure> {
        self.solutions
            .solution(index, "Geom2dGcc_Circ2d2TanRadGeo::ThisSolution")
    }

    /// `WhichQualifier(Index, Qualif1, Qualif2)`. The curve-curve constructor never
    /// writes the second qualifier (it writes the first twice), whose slot is
    /// uninitialized natively and GccEnt_unqualified here;
    /// `let (q1, q2) = circ.which_qualifier(1)?;`.
    pub fn which_qualifier(&self, index: i32) -> Result<(Position, Position), Failure> {
        let s = &self.solutions;
        s.check(index, "Geom2dGcc_Circ2d2TanRadGeo::WhichQualifier")?;
        Ok((s.qualifier_of(0, index)?, s.qualifier_of(1, index)?))
    }

    /// `Tangency1(Index, ParSol, ParArg, PntSol)`;
    /// `let (par_sol, par_arg, point) = circ.tangency1(1)?;`.
    pub fn tangency1(&self, index: i32) -> Result<(f64, f64, Point2), Failure> {
        self.solutions
            .tangency_of(0, index, "Geom2dGcc_Circ2d2TanRadGeo::Tangency1")
    }

    /// `Tangency2(Index, ParSol, ParArg, PntSol)`;
    /// `let (par_sol, par_arg, point) = circ.tangency2(1)?;`.
    pub fn tangency2(&self, index: i32) -> Result<(f64, f64, Point2), Failure> {
        self.solutions
            .tangency_of(1, index, "Geom2dGcc_Circ2d2TanRadGeo::Tangency2")
    }

    /// `IsTheSame1(Index)`: always false; `circ.is_the_same1(1)?`.
    pub fn is_the_same1(&self, index: i32) -> Result<bool, Failure> {
        self.solutions
            .same_of(0, index, "Geom2dGcc_Circ2d2TanRadGeo::IsTheSame1")
    }

    /// `IsTheSame2(Index)`: always false; `circ.is_the_same2(1)?`.
    pub fn is_the_same2(&self, index: i32) -> Result<bool, Failure> {
        self.solutions
            .same_of(1, index, "Geom2dGcc_Circ2d2TanRadGeo::IsTheSame2")
    }
}
