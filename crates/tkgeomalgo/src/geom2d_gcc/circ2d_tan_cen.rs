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
// Rust translation of Geom2dGcc_Circ2dTanCen.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dGcc_Circ2dTanCen`: circles of given centre tangent to a curve.
use super::{Circ2dTanCenGeo, QualifiedCurve, circle_of, line_of, point_of};
use crate::gcc_ana;
use crate::gcc_ana::solutions::{Slots, Tangencies, check_solution};
use crate::gcc_ent::{Position, QualifiedCirc};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dPoint;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Circle2, Point2};

/// `Geom2dGcc_Circ2dTanCen`: the circles centred at a point and tangent to a
/// qualified curve: GccAna_Circ2dTanCen for a line or a circle, else
/// Geom2dGcc_Circ2dTanCenGeo.
/// ```
/// use rustycad_tkgeomalgo::geom2d_gcc::{Circ2dTanCen, unqualified};
/// use rustycad_tkg2d::geom2d::{Geom2dCartesianPoint, Geom2dCurve, Geom2dLine, Geom2dPoint};
/// use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::{Direction2, Point2};
/// let line: Handle<dyn Geom2dCurve> =
///     Handle::new(Geom2dLine::from_point(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0)?)).cast();
/// let q = unqualified(&Geom2dAdaptorCurve::with_curve(&line)?)?;
/// let center: Handle<dyn Geom2dPoint> = Handle::new(Geom2dCartesianPoint::new(Point2::new(1.0, 2.0))).cast();
/// let circ = Circ2dTanCen::new(&q, &center, 1e-9)?;
/// assert_eq!(circ.this_solution(1)?.radius(), 2.0);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Debug)]
pub struct Circ2dTanCen {
    well_done: bool,
    nbr_sol: i32,
    cirsol: Slots<Circle2>,
    qualifier1: Slots<Position>,
    tangency1: Tangencies,
}

impl Circ2dTanCen {
    /// `Geom2dGcc_Circ2dTanCen(Qualified1, Pcenter, Tolerance)`. The GccAna tangency
    /// of a solution identical to the argument raises StdFail_NotDone, which leaves
    /// the constructor as natively (a centre at the centre of a circle argument);
    /// `Circ2dTanCen::new(&q, &center, 1e-9)?`.
    pub fn new(
        qualified1: &QualifiedCurve,
        pcenter: &Handle<dyn Geom2dPoint>,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self {
            well_done: false,
            nbr_sol: 0,
            cirsol: Slots::new(2),
            qualifier1: Slots::new(2),
            tangency1: Tangencies::new(2),
        };
        let c1 = qualified1.curve();
        let type1: CurveType = c1.get_type()?;
        let pcenter: Point2 = point_of(pcenter)?;
        match type1 {
            CurveType::Circle => {
                let qc1 = QualifiedCirc::new(circle_of(c1)?, qualified1.qualifier());
                circ.store_ana(&gcc_ana::Circ2dTanCen::tangent_to_circle(
                    qc1, pcenter, tolerance,
                )?)?;
            }
            CurveType::Line => {
                circ.store_ana(&gcc_ana::Circ2dTanCen::tangent_to_line(
                    line_of(c1)?,
                    pcenter,
                )?)?;
            }
            _ => circ.store_geo(&Circ2dTanCenGeo::new(qualified1, pcenter, tolerance)?)?,
        }
        Ok(circ)
    }

    // The solutions of GccAna_Circ2dTanCen, with their TheSame flags.
    fn store_ana(&mut self, ana: &gcc_ana::Circ2dTanCen) -> Result<(), Failure> {
        self.well_done = ana.is_done();
        self.nbr_sol = ana.nb_solutions();
        for j in 1..=self.nbr_sol {
            self.cirsol.set(j, ana.this_solution(j)?)?;
            self.qualifier1.set(j, ana.which_qualifier(j)?)?;
            let same: i32 = i32::from(ana.is_the_same1(j)?);
            let (par_sol, par_arg, point) = ana.tangency1(j)?;
            self.tangency1.record(j, point, par_sol, par_arg)?;
            self.tangency1.same.set(j, same)?;
        }
        Ok(())
    }

    // The solutions of Geom2dGcc_Circ2dTanCenGeo.
    fn store_geo(&mut self, geo: &Circ2dTanCenGeo) -> Result<(), Failure> {
        self.well_done = geo.is_done();
        self.nbr_sol = geo.nb_solutions();
        for j in 1..=self.nbr_sol {
            self.cirsol.set(j, geo.this_solution(j)?)?;
            let (par_sol, par_arg, point) = geo.tangency1(j)?;
            self.tangency1.record(j, point, par_sol, par_arg)?;
            self.qualifier1.set(j, geo.which_qualifier(j)?)?;
        }
        Ok(())
    }

    /// `IsDone()`; `circ.is_done()`.
    pub fn is_done(&self) -> bool {
        self.well_done
    }

    /// `NbSolutions()`; `circ.nb_solutions()`.
    pub fn nb_solutions(&self) -> i32 {
        self.nbr_sol
    }

    /// `ThisSolution(Index)`; `circ.this_solution(1)?`.
    pub fn this_solution(&self, index: i32) -> Result<Circle2, Failure> {
        let member = "Geom2dGcc_Circ2dTanCen::ThisSolution";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        self.cirsol.get(index)
    }

    /// `WhichQualifier(Index, Qualif1)`; `circ.which_qualifier(1)?`.
    pub fn which_qualifier(&self, index: i32) -> Result<Position, Failure> {
        let member = "Geom2dGcc_Circ2dTanCen::WhichQualifier";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        self.qualifier1.get(index)
    }

    /// `Tangency1(Index, ParSol, ParArg, PntSol)`: StdFail_NotDone for a solution
    /// identical to the argument; `let (par_sol, par_arg, point) = circ.tangency1(1)?;`.
    pub fn tangency1(&self, index: i32) -> Result<(f64, f64, Point2), Failure> {
        let member = "Geom2dGcc_Circ2dTanCen::Tangency1";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        self.tangency1.checked(index, member)
    }

    /// `IsTheSame1(Index)`; `circ.is_the_same1(1)?`.
    pub fn is_the_same1(&self, index: i32) -> Result<bool, Failure> {
        let member = "Geom2dGcc_Circ2dTanCen::IsTheSame1";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        self.tangency1.is_the_same(index)
    }
}
