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
// Rust translation of Geom2dGcc_Circ2d3TanIter.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dGcc_Circ2d3TanIter`: a circle tangent to three arguments (curves,
//! circles, lines, points), found from start parameters.
mod circles;
mod curves;
mod lines;

use super::FunctionTanCuCuCu;
use crate::gcc_ana;
use crate::gcc_ana::solutions::not_done;
use crate::gcc_ent::Position;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{self, Circle2, Point2, Vector2, Xy};
use rustycad_tkmath::math::{FunctionSet, FunctionSetRoot, Vector};

/// `Geom2dGcc_Circ2d3TanIter`: the circle through the three points that
/// math_FunctionSetRoot finds from the start parameters on
/// Geom2dGcc_FunctionTanCuCuCu, kept when it is tangent to the arguments (normal
/// directions within a cosine of 1e-12) on the sides their qualifiers allow.
///
/// FunctionTanCuCuCu gives no derivatives (its native `Values` writes nothing), so
/// the search stops at once on the start point, clamped to the bounds.
/// ```
/// use rustycad_tkgeomalgo::geom2d_gcc::{Circ2d3TanIter, unqualified};
/// use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
/// use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::{Direction2, Point2};
/// let line: Handle<dyn Geom2dCurve> =
///     Handle::new(Geom2dLine::from_point(Point2::new(0.0, -1.0), Direction2::new(1.0, 0.0)?)).cast();
/// let q = unqualified(&Geom2dAdaptorCurve::with_range(&line, -5.0, 5.0)?)?;
/// let circ = Circ2d3TanIter::curve_pnt_pnt(&q, Point2::new(-1.0, 0.0), Point2::new(1.0, 0.0), 0.0, 1e-9)?;
/// assert!(circ.is_done());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Debug)]
pub struct Circ2d3TanIter {
    well_done: bool,
    cirsol: Circle2,
    qualifier: [Position; 3],
    the_same: [bool; 3],
    pnttg: [Point2; 3],
    parsol: [f64; 3],
    pararg: [f64; 3],
}

impl Default for Circ2d3TanIter {
    fn default() -> Self {
        Self {
            well_done: false,
            cirsol: Circle2::default(),
            qualifier: [Position::NoQualifier; 3],
            the_same: [false; 3],
            pnttg: [Point2::default(); 3],
            parsol: [0.0; 3],
            pararg: [0.0; 3],
        }
    }
}

// The bounds, start point and tolerances of one search (math_Vector(1, 3) each).
struct Search {
    umin: [f64; 3],
    umax: [f64; 3],
    ufirst: [f64; 3],
    tol: [f64; 3],
}

// `math_Vector v(1, 3)` holding `values`.
fn vector3(values: [f64; 3]) -> Result<Vector<'static>, Failure> {
    let mut v: Vector<'static> = Vector::with_bounds(1, 3)?;
    for (i, value) in (1..=3).zip(values) {
        v[i] = value;
    }
    Ok(v)
}

// The root of math_FunctionSetRoot on `func` from `search`, when done; then the
// native `Func.Value(Ufirst, Umin)`, at the root (or, with `value_first`, at the start
// point before the root is read, as the circle-line-curve constructor does).
fn solve(
    func: &mut FunctionTanCuCuCu<'_>,
    search: &Search,
    value_first: bool,
) -> Result<Option<[f64; 3]>, Failure> {
    let (umin, umax) = (vector3(search.umin)?, vector3(search.umax)?);
    let mut ufirst = vector3(search.ufirst)?;
    let mut root = FunctionSetRoot::new(func, &vector3(search.tol)?, 100)?;
    root.perform_bounded(func, &ufirst, &umin, &umax, false)?;
    func.raised()?;
    if !root.is_done() {
        return Ok(None);
    }
    let mut values = vector3(search.umin)?;
    if value_first {
        func.value(&ufirst, &mut values);
        func.raised()?;
    }
    root.copy_root(&mut ufirst)?;
    if !value_first {
        func.value(&ufirst, &mut values);
        func.raised()?;
    }
    Ok(Some([ufirst[1], ufirst[2], ufirst[3]]))
}

// Vec . Tan / (|Vec| |Tan|), or 0 when either is shorter than gp::Resolution().
fn cosine(vec: Vector2, tan: Vector2) -> f64 {
    let (normevec, normetan) = (vec.magnitude(), tan.magnitude());
    if normevec >= gp::resolution() && normetan >= gp::resolution() {
        return vec.dot(tan) / (normevec * normetan);
    }
    0.0
}

// `dot1 <= Tol && dot2 <= Tol && dot3 <= Tol` with Tol = 1e-12 (NaN fails).
fn all_normal(dots: [f64; 3]) -> bool {
    dots.iter().all(|&dot| dot <= 1.0e-12)
}

// The circle through the three points (GccAna_Circ2d3Tan), if done.
fn circle_through(points: [Point2; 3], tol: f64) -> Result<Option<Circle2>, Failure> {
    let circ = gcc_ana::Circ2d3Tan::pnt_pnt_pnt(points[0], points[1], points[2], tol)?;
    if !circ.is_done() {
        return Ok(None);
    }
    Ok(Some(circ.this_solution(1)?))
}

// The point of a circle at `u`: centre + R (cos u, sin u).
fn circle_point(circle: &Circle2, u: f64) -> Point2 {
    Point2::from(circle.location().xy() + circle.radius() * Xy::new(u.cos(), u.sin()))
}

// (-sin u, cos u): the unit tangent of a circle at `u`.
fn unit_tangent(u: f64) -> Vector2 {
    Vector2::new(-u.sin(), u.cos())
}

impl Circ2d3TanIter {
    // Records the accepted solution: qualifiers, tangency points, argument
    // parameters and solution parameters.
    fn accept(
        &mut self,
        qualifier: [Position; 3],
        pnttg: [Point2; 3],
        pararg: [f64; 3],
        parsol: [f64; 3],
    ) {
        (self.qualifier, self.pnttg, self.pararg, self.parsol) = (qualifier, pnttg, pararg, parsol);
        self.well_done = true;
    }

    /// `IsDone()`; `circ.is_done()`.
    pub fn is_done(&self) -> bool {
        self.well_done
    }

    /// `ThisSolution()`: the last circle through the three points, even when it was
    /// rejected or the search failed (a default circle), as natively (no check);
    /// `circ.this_solution()`.
    pub fn this_solution(&self) -> Circle2 {
        self.cirsol
    }

    /// `WhichQualifier(Qualif1, Qualif2, Qualif3)`; `let (q1, q2, q3) = circ.which_qualifier()?;`.
    pub fn which_qualifier(&self) -> Result<(Position, Position, Position), Failure> {
        self.check_done("Geom2dGcc_Circ2d3TanIter::WhichQualifier")?;
        let [q1, q2, q3] = self.qualifier;
        Ok((q1, q2, q3))
    }

    /// `Tangency1(ParSol, ParArg, PntSol)`; `let (par_sol, par_arg, point) = circ.tangency1()?;`.
    pub fn tangency1(&self) -> Result<(f64, f64, Point2), Failure> {
        self.check_done("Geom2dGcc_Circ2d3TanIter::Tangency1")?;
        if self.the_same[0] {
            return Err(not_done("Geom2dGcc_Circ2d3TanIter::Tangency1"));
        }
        Ok(self.tangency(0))
    }

    /// `Tangency2(ParSol, ParArg, PntSol)`; `let (par_sol, par_arg, point) = circ.tangency2()?;`.
    pub fn tangency2(&self) -> Result<(f64, f64, Point2), Failure> {
        self.check_done("Geom2dGcc_Circ2d3TanIter::Tangency2")?;
        Ok(self.tangency(1))
    }

    /// `Tangency3(ParSol, ParArg, PntSol)`; `let (par_sol, par_arg, point) = circ.tangency3()?;`.
    pub fn tangency3(&self) -> Result<(f64, f64, Point2), Failure> {
        self.check_done("Geom2dGcc_Circ2d3TanIter::Tangency3")?;
        Ok(self.tangency(2))
    }

    /// `IsTheSame1()`: false; `circ.is_the_same1()?`.
    pub fn is_the_same1(&self) -> Result<bool, Failure> {
        self.check_done("Geom2dGcc_Circ2d3TanIter::IsTheSame1")?;
        Ok(self.the_same[0])
    }

    /// `IsTheSame2()`: natively the flag of the third argument (false);
    /// `circ.is_the_same2()?`.
    pub fn is_the_same2(&self) -> Result<bool, Failure> {
        self.check_done("Geom2dGcc_Circ2d3TanIter::IsTheSame2")?;
        Ok(self.the_same[2])
    }

    /// `IsTheSame3()`: natively always true once done; `circ.is_the_same3()?`.
    pub fn is_the_same3(&self) -> Result<bool, Failure> {
        self.check_done("Geom2dGcc_Circ2d3TanIter::IsTheSame3")?;
        Ok(true)
    }

    fn tangency(&self, arg: usize) -> (f64, f64, Point2) {
        (self.parsol[arg], self.pararg[arg], self.pnttg[arg])
    }

    fn check_done(&self, member: &str) -> Result<(), Failure> {
        if !self.well_done {
            return Err(not_done(member));
        }
        Ok(())
    }
}
