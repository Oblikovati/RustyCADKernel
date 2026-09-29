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
// Rust translation of Geom2dGcc_Circ2d2TanRad.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dGcc_Circ2d2TanRad`: circles of given radius tangent to two curves, to a
//! curve and a point, or through two points.
use super::circle_solutions::{CircleSolutions, TwoTangencySolver};
use super::{Circ2d2TanRadGeo, QualifiedCurve, circle_of, line_of, point_of};
use crate::gcc_ana;
use crate::gcc_ana::solutions::negative_value;
use crate::gcc_ent::{Position, QualifiedCirc, QualifiedLin, bad_qualifier};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dPoint;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Circle2, Point2};

const MEMBER: &str = "Geom2dGcc_Circ2d2TanRad";

/// `Geom2dGcc_Circ2d2TanRad`: the circles of radius `Radius` tangent to two
/// qualified curves or to a curve and a point (at most 16), or through two points:
/// GccAna_Circ2d2TanRad when the curves are lines and circles, else
/// Geom2dGcc_Circ2d2TanRadGeo. When the analytic or geometric solver takes the
/// arguments in the other order (`Invert`), the consultations swap them back.
/// ```
/// use rustycad_tkgeomalgo::geom2d_gcc::{Circ2d2TanRad, unqualified};
/// use rustycad_tkg2d::geom2d::{Geom2dCartesianPoint, Geom2dCurve, Geom2dLine, Geom2dPoint};
/// use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::{Direction2, Point2};
/// let line: Handle<dyn Geom2dCurve> =
///     Handle::new(Geom2dLine::from_point(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0)?)).cast();
/// let q = unqualified(&Geom2dAdaptorCurve::with_curve(&line)?)?;
/// let point: Handle<dyn Geom2dPoint> = Handle::new(Geom2dCartesianPoint::new(Point2::new(0.0, 1.0))).cast();
/// let circ = Circ2d2TanRad::with_point(&q, &point, 1.0, 1e-9)?;
/// assert!(circ.is_done());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Debug)]
pub struct Circ2d2TanRad {
    solutions: CircleSolutions,
    invert: bool,
}

// The solver kinds of an argument: a line, a circle or another curve.
enum Kind {
    Line,
    Circle,
    Curve,
}

fn kind_of(q: &QualifiedCurve) -> Result<Kind, Failure> {
    Ok(match q.curve().get_type()? {
        CurveType::Line => Kind::Line,
        CurveType::Circle => Kind::Circle,
        _ => Kind::Curve,
    })
}

// GccEnt_QualifiedLin of a line argument; an enclosing line raises
// GccEnt_BadQualifier (natively after WellDone = false).
fn qualified_line(q: &QualifiedCurve) -> Result<QualifiedLin, Failure> {
    if q.is_enclosing() {
        return Err(bad_qualifier(&format!(
            "{MEMBER}: line qualifier GccEnt_enclosing; expected enclosed, outside or unqualified"
        )));
    }
    Ok(QualifiedLin::new(line_of(q.curve())?, q.qualifier()))
}

fn qualified_circ(q: &QualifiedCurve) -> Result<QualifiedCirc, Failure> {
    Ok(QualifiedCirc::new(circle_of(q.curve())?, q.qualifier()))
}

impl Circ2d2TanRad {
    fn start(radius: f64, size: usize) -> Result<Self, Failure> {
        if radius < 0.0 {
            return Err(negative_value(MEMBER, radius));
        }
        Ok(Self {
            solutions: CircleSolutions::new(size, 2),
            invert: false,
        })
    }

    // WellDone, NbrSol, the qualifiers and the Results of the solver that ran.
    fn take(
        &mut self,
        done: bool,
        count: i32,
        source: &impl TwoTangencySolver,
    ) -> Result<(), Failure> {
        self.solutions.well_done = done;
        self.solutions.nbr_sol = count;
        self.solutions.store_qualifiers(source)?;
        self.solutions.store_results(source)
    }

    fn take_ana(&mut self, ana: gcc_ana::Circ2d2TanRad) -> Result<(), Failure> {
        self.take(ana.is_done(), ana.nb_solutions(), &ana)
    }

    fn take_geo(&mut self, geo: Circ2d2TanRadGeo) -> Result<(), Failure> {
        self.take(geo.is_done(), geo.nb_solutions(), &geo)
    }

    /// `Geom2dGcc_Circ2d2TanRad(Qualified1, Qualified2, Radius, Tolerance)`:
    /// Standard_NegativeValue for a negative radius, GccEnt_BadQualifier for an
    /// enclosing line; `Circ2d2TanRad::new(&q1, &q2, 1.0, 1e-9)?`.
    pub fn new(
        qualified1: &QualifiedCurve,
        qualified2: &QualifiedCurve,
        radius: f64,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::start(radius, 16)?;
        let (q1, q2) = (qualified1, qualified2);
        use gcc_ana::Circ2d2TanRad as Ana;
        match (kind_of(q1)?, kind_of(q2)?) {
            (Kind::Circle, Kind::Circle) => circ.take_ana(Ana::circ_circ(
                qualified_circ(q1)?,
                qualified_circ(q2)?,
                radius,
                tolerance,
            )?)?,
            (Kind::Circle, Kind::Line) => circ.take_ana(Ana::circ_lin(
                qualified_circ(q1)?,
                qualified_line(q2)?,
                radius,
                tolerance,
            )?)?,
            (Kind::Line, Kind::Circle) => {
                let ql1 = qualified_line(q1)?;
                circ.invert = true;
                circ.take_ana(Ana::circ_lin(qualified_circ(q2)?, ql1, radius, tolerance)?)?
            }
            (Kind::Line, Kind::Line) => {
                let ql1 = qualified_line(q1)?;
                circ.take_ana(Ana::lin_lin(ql1, qualified_line(q2)?, radius, tolerance)?)?
            }
            (kind1, kind2) => circ.geometric(q1, q2, (kind1, kind2), (radius, tolerance))?,
        }
        Ok(circ)
    }

    // Geom2dGcc_Circ2d2TanRadGeo when an argument is neither a line nor a circle:
    // the line or circle (first one first) is its first argument.
    fn geometric(
        &mut self,
        q1: &QualifiedCurve,
        q2: &QualifiedCurve,
        kinds: (Kind, Kind),
        (radius, tolerance): (f64, f64),
    ) -> Result<(), Failure> {
        let geo = match kinds {
            (Kind::Line, _) => {
                Circ2d2TanRadGeo::lin_curve(qualified_line(q1)?, q2, radius, tolerance)?
            }
            (Kind::Circle, _) => {
                Circ2d2TanRadGeo::circ_curve(qualified_circ(q1)?, q2, radius, tolerance)?
            }
            (_, Kind::Line) => {
                self.invert = true;
                Circ2d2TanRadGeo::lin_curve(qualified_line(q2)?, q1, radius, tolerance)?
            }
            (_, Kind::Circle) => {
                self.invert = true;
                Circ2d2TanRadGeo::circ_curve(qualified_circ(q2)?, q1, radius, tolerance)?
            }
            _ => Circ2d2TanRadGeo::curve_curve(q1, q2, radius, tolerance)?,
        };
        self.take_geo(geo)
    }

    /// `Geom2dGcc_Circ2d2TanRad(Qualified1, Point, Radius, Tolerance)`: GccAna for a
    /// line or a circle, else Geom2dGcc_Circ2d2TanRadGeo;
    /// `Circ2d2TanRad::with_point(&q, &point, 1.0, 1e-9)?`.
    pub fn with_point(
        qualified1: &QualifiedCurve,
        point: &Handle<dyn Geom2dPoint>,
        radius: f64,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::start(radius, 16)?;
        use gcc_ana::Circ2d2TanRad as Ana;
        match kind_of(qualified1)? {
            Kind::Circle => {
                let qc1 = qualified_circ(qualified1)?;
                circ.take_ana(Ana::circ_pnt(qc1, point_of(point)?, radius, tolerance)?)?
            }
            Kind::Line => {
                let ql1 = QualifiedLin::new(line_of(qualified1.curve())?, qualified1.qualifier());
                circ.take_ana(Ana::lin_pnt(ql1, point_of(point)?, radius, tolerance)?)?
            }
            Kind::Curve => circ.take_geo(Circ2d2TanRadGeo::curve_pnt(
                qualified1,
                point_of(point)?,
                radius,
                tolerance,
            )?)?,
        }
        Ok(circ)
    }

    /// `Geom2dGcc_Circ2d2TanRad(Point1, Point2, Radius, Tolerance)`: GccAna;
    /// `Circ2d2TanRad::with_points(&p1, &p2, 1.0, 1e-9)?`.
    pub fn with_points(
        point1: &Handle<dyn Geom2dPoint>,
        point2: &Handle<dyn Geom2dPoint>,
        radius: f64,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::start(radius, 2)?;
        let (p1, p2): (Point2, Point2) = (point_of(point1)?, point_of(point2)?);
        circ.take_ana(gcc_ana::Circ2d2TanRad::pnt_pnt(p1, p2, radius, tolerance)?)?;
        Ok(circ)
    }

    /// `Results(const GccAna_Circ2d2TanRad&)`: copies the circles, TheSame flags and
    /// tangencies of solutions 1 to NbSolutions() of `circ`;
    /// `solver.results_ana(&ana)?`.
    pub fn results_ana(&mut self, circ: &gcc_ana::Circ2d2TanRad) -> Result<(), Failure> {
        self.solutions.store_results(circ)
    }

    /// `Results(const Geom2dGcc_Circ2d2TanRadGeo&)`; `solver.results_geo(&geo)?`.
    pub fn results_geo(&mut self, circ: &Circ2d2TanRadGeo) -> Result<(), Failure> {
        self.solutions.store_results(circ)
    }

    // The stored argument of native argument `arg` (0 or 1) under Invert.
    fn stored(&self, arg: usize) -> usize {
        if self.invert { 1 - arg } else { arg }
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
            .solution(index, "Geom2dGcc_Circ2d2TanRad::ThisSolution")
    }

    /// `WhichQualifier(Index, Qualif1, Qualif2)`; `let (q1, q2) = circ.which_qualifier(1)?;`.
    pub fn which_qualifier(&self, index: i32) -> Result<(Position, Position), Failure> {
        let s = &self.solutions;
        s.check(index, "Geom2dGcc_Circ2d2TanRad::WhichQualifier")?;
        Ok((
            s.qualifier_of(self.stored(0), index)?,
            s.qualifier_of(self.stored(1), index)?,
        ))
    }

    /// `Tangency1(Index, ParSol, ParArg, PntSol)`;
    /// `let (par_sol, par_arg, point) = circ.tangency1(1)?;`.
    pub fn tangency1(&self, index: i32) -> Result<(f64, f64, Point2), Failure> {
        self.solutions
            .tangency_of(self.stored(0), index, "Geom2dGcc_Circ2d2TanRad::Tangency1")
    }

    /// `Tangency2(Index, ParSol, ParArg, PntSol)`;
    /// `let (par_sol, par_arg, point) = circ.tangency2(1)?;`.
    pub fn tangency2(&self, index: i32) -> Result<(f64, f64, Point2), Failure> {
        self.solutions
            .tangency_of(self.stored(1), index, "Geom2dGcc_Circ2d2TanRad::Tangency2")
    }

    /// `IsTheSame1(Index)`; `circ.is_the_same1(1)?`.
    pub fn is_the_same1(&self, index: i32) -> Result<bool, Failure> {
        self.solutions
            .same_of(self.stored(0), index, "Geom2dGcc_Circ2d2TanRad::IsTheSame1")
    }

    /// `IsTheSame2(Index)`; `circ.is_the_same2(1)?`.
    pub fn is_the_same2(&self, index: i32) -> Result<bool, Failure> {
        self.solutions
            .same_of(self.stored(1), index, "Geom2dGcc_Circ2d2TanRad::IsTheSame2")
    }
}
