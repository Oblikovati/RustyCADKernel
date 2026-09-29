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
// Rust translation of Geom2dGcc_Circ2dTanCenGeo.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dGcc_Circ2dTanCenGeo`: circles of given centre tangent to a curve.
use super::{CurveTool, QCurve};
use crate::gcc_ana::solutions::{Slots, Tangencies, check_index, check_solution, circle_at};
use crate::gcc_ent::Position;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::extrema::ExtPC2d;
use rustycad_tkmath::gp::{self, Circle2, Point2, Vector2};

/// `Geom2dGcc_Circ2dTanCenGeo`: the circles centred at `Pcenter` tangent to a
/// qualified curve, from the nearest and farthest extrema of Extrema_ExtPC2d
/// (Standard_Failure when Extrema fails or finds none).
/// ```
/// use rustycad_tkgeomalgo::geom2d_gcc::{Circ2dTanCenGeo, unqualified};
/// use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
/// use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::{Direction2, Point2};
/// let line: Handle<dyn Geom2dCurve> =
///     Handle::new(Geom2dLine::from_point(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0)?)).cast();
/// let q = unqualified(&Geom2dAdaptorCurve::with_range(&line, -5.0, 5.0)?)?;
/// let circ = Circ2dTanCenGeo::new(&q, Point2::new(1.0, 2.0), 1e-9)?;
/// assert_eq!(circ.this_solution(1)?.radius(), 2.0);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Debug)]
pub struct Circ2dTanCenGeo {
    well_done: bool,
    nbr_sol: i32,
    cirsol: Slots<Circle2>,
    qualifier1: Slots<Position>,
    tangency1: Tangencies,
}

// The nearest (1) and farthest (2) extrema: (square distance, parameter, point, index).
type Extreme = (f64, f64, Point2, i32);

impl Circ2dTanCenGeo {
    /// `Geom2dGcc_Circ2dTanCenGeo(Qualified1, Pcenter, Tolerance)`: one circle when
    /// the nearest and farthest extrema coincide, else up to two, each kept when
    /// the centre direction is normal to the curve (dot product at most 1e-12) and
    /// the qualifier accepts its side (gp_VectorWithNullMagnitude for a centre on
    /// the curve); `Circ2dTanCenGeo::new(&q, center, 1e-9)?`.
    pub fn new(qualified1: &QCurve, pcenter: Point2, tolerance: f64) -> Result<Self, Failure> {
        let mut circ = Self {
            well_done: false,
            nbr_sol: 0,
            cirsol: Slots::new(2),
            qualifier1: Slots::new(2),
            tangency1: Tangencies::new(2),
        };
        let tol: f64 = tolerance.abs();
        let curve = qualified1.curve();
        let (first, last) = (
            CurveTool::first_parameter(curve)?,
            CurveTool::last_parameter(curve)?,
        );
        let distmin = ExtPC2d::with_range(pcenter, curve, first, last, tol)?;
        let [near, far] = extremes(&distmin)?;
        let nbsol: usize = if near.3 == far.3 { 1 } else { 2 };
        for &(dist2, param, p_tan, _) in [near, far].iter().take(nbsol) {
            circ.try_extreme(qualified1, pcenter, dist2, param, p_tan)?;
        }
        Ok(circ)
    }

    // The circle through the extremum at `param` when it is a tangency the
    // qualifier accepts.
    fn try_extreme(
        &mut self,
        qualified1: &QCurve,
        pcenter: Point2,
        dist2: f64,
        param: f64,
        p_tan: Point2,
    ) -> Result<(), Failure> {
        let (point1, tan1) = CurveTool::d1(qualified1.curve(), param)?;
        let normetan1: f64 = tan1.magnitude();
        let vec1 = Vector2::between(point1, pcenter);
        let normevec1: f64 = vec1.magnitude();
        let dot1: f64 = if normevec1 >= gp::resolution() && normetan1 >= gp::resolution() {
            vec1.dot(tan1) / (normevec1 * normetan1)
        } else {
            0.0
        };
        // `if (dot1 <= Tol)`: a NaN product is no tangency.
        let normal: bool = dot1 <= 1.0e-12;
        if !normal {
            return Ok(());
        }
        let angle1: f64 = vec1.angle(tan1)?;
        if !(qualified1.is_unqualified()
            || (qualified1.is_enclosing() && angle1 <= 0.0)
            || (qualified1.is_outside() && angle1 >= 0.0)
            || (qualified1.is_enclosed() && angle1 <= 0.0))
        {
            return Ok(());
        }
        self.nbr_sol += 1;
        let index: i32 = self.nbr_sol;
        self.cirsol.set(index, circle_at(pcenter, dist2.sqrt())?)?;
        self.qualifier1.set(index, qualified1.qualifier())?;
        self.tangency1.record(index, p_tan, 0.0, param)?;
        self.well_done = true;
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

    /// `ThisSolution(Index)`: Standard_OutOfRange outside `1..=NbSolutions()` (no
    /// done test, as natively); `circ.this_solution(1)?`.
    pub fn this_solution(&self, index: i32) -> Result<Circle2, Failure> {
        check_index(
            self.nbr_sol,
            index,
            "Geom2dGcc_Circ2dTanCenGeo::ThisSolution",
        )?;
        self.cirsol.get(index)
    }

    /// `WhichQualifier(Index, Qualif1)`; `circ.which_qualifier(1)?`.
    pub fn which_qualifier(&self, index: i32) -> Result<Position, Failure> {
        let member = "Geom2dGcc_Circ2dTanCenGeo::WhichQualifier";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        self.qualifier1.get(index)
    }

    /// `Tangency1(Index, ParSol, ParArg, PntSol)`;
    /// `let (par_sol, par_arg, point) = circ.tangency1(1)?;`.
    pub fn tangency1(&self, index: i32) -> Result<(f64, f64, Point2), Failure> {
        let member = "Geom2dGcc_Circ2dTanCenGeo::Tangency1";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        self.tangency1.raw(index)
    }
}

// The nearest and farthest extrema of a done Extrema_ExtPC2d with at least one
// extremum (Standard_Failure otherwise); a later extremum replaces an earlier one
// only when strictly nearer (farther).
fn extremes(distmin: &ExtPC2d<'_>) -> Result<[Extreme; 2], Failure> {
    if !distmin.is_done() || distmin.nb_ext()? == 0 {
        return Err(Failure::with_type(
            "Standard_Failure",
            "Geom2dGcc_Circ2dTanCenGeo: Extrema_ExtPC2d found no extremum; expected at least one",
            "",
        ));
    }
    let mut near: Extreme = (f64::MAX, 0.0, Point2::default(), 0);
    let mut far: Extreme = (0.0, 0.0, Point2::default(), 0);
    for i in 1..=distmin.nb_ext()? {
        let point = distmin.point(i)?;
        let square_distance: f64 = distmin.square_distance(i)?;
        let candidate: Extreme = (square_distance, point.parameter(), *point.value(), i);
        if near.3 == 0 || square_distance < near.0 {
            near = candidate;
        }
        if far.3 == 0 || square_distance > far.0 {
            far = candidate;
        }
    }
    Ok([near, far])
}
