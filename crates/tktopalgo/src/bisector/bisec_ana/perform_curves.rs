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
// Rust translation of ModelingAlgorithms/TKTopAlgo/Bisector/Bisector_BisecAna.cxx (Perform of two curves) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Bisector_BisecAna::Perform(Cu1, Cu2, ...)`: the bisector of two circles or
//! lines that meet at (or near) a point.
use super::BisecAna;
use super::distance::{DistanceQuery, DistanceResult, distance};
use super::solutions::Trimmed;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkgeomalgo::gcc_int::Bisec;
use rustycad_tkmath::geom_abs::JoinType;
use rustycad_tkmath::gp::{Circle2, Line2, Point2, Vector2};
use std::f64::consts::PI;

/// The arguments of the curve-curve Perform, with the tangents it derives from the
/// two curves (`tan1` already reversed) and the jgv (OCC26296) reference vector.
pub(super) struct CurvesContext {
    pub apoint: Point2,
    pub afirstvector: Vector2,
    pub asecondvector: Vector2,
    pub adirection: f64,
    pub ajointype: JoinType,
    pub tolerance: f64,
    pub oncurve: bool,
    pub tan1: Vector2,
    pub tan2: Vector2,
    pub line_bis_vec: Vector2,
    pub first_curve: Handle<dyn Geom2dCurve>,
    pub second_curve: Handle<dyn Geom2dCurve>,
}

/// The analytic support of each curve: `cas` 1 (two circles), 2 (a circle and a
/// line, whatever the order) or 3 (two lines).
pub(super) enum CurvesCase {
    Circles(Circle2, Circle2),
    CircleLine(Circle2, Line2),
    Lines(Line2, Line2),
}

impl CurvesContext {
    fn query(&self, first: Vector2, second: Vector2) -> DistanceQuery {
        DistanceQuery {
            apoint: self.apoint,
            first,
            second,
            vec_ref: self.line_bis_vec,
            adirection: self.adirection,
        }
    }
    /// The first `Distance` pass (sense true: only the status) with the tangents.
    pub(super) fn status_pass(&self, solution: &dyn Bisec) -> Result<DistanceResult, Failure> {
        distance(self.query(self.tan1, self.tan2), solution, true, false)
    }
    /// The second `Distance` pass (sense false): the reversed tangents on the curve,
    /// the sector vectors otherwise.
    pub(super) fn sense_pass(&self, solution: &dyn Bisec) -> Result<DistanceResult, Failure> {
        if self.oncurve {
            return distance(self.query(self.tan2, self.tan1), solution, false, false);
        }
        let query: DistanceQuery = self.query(self.afirstvector, self.asecondvector);
        distance(query, solution, false, false)
    }
    /// The sector pass of two non parallel lines off the curve: the special case
    /// `IsBisecOfTwoLines` of `Distance`.
    pub(super) fn lines_pass(&self, solution: &dyn Bisec) -> Result<DistanceResult, Failure> {
        if self.oncurve {
            return self.sense_pass(solution);
        }
        let query: DistanceQuery = self.query(self.afirstvector, self.asecondvector);
        distance(query, solution, false, true)
    }
}

// The basis of a trimmed curve, the curve itself otherwise.
fn basis_of(curve: &Handle<dyn Geom2dCurve>) -> Handle<dyn Geom2dCurve> {
    match curve.cast::<Geom2dTrimmedCurve>().get() {
        Some(trimmed) => trimmed.basis_curve(),
        None => curve.clone(),
    }
}

// The analytic case of two basis curves; `None` for the "Not yet implemented"
// types (natively printed on std::cout, not by the port), which raise
// StdFail_NotDone below.
fn classify(
    first: &Handle<dyn Geom2dCurve>,
    second: &Handle<dyn Geom2dCurve>,
) -> Result<Option<CurvesCase>, Failure> {
    let (c1, c2) = (first.cast::<Geom2dCircle>(), second.cast::<Geom2dCircle>());
    let (l1, l2) = (first.cast::<Geom2dLine>(), second.cast::<Geom2dLine>());
    Ok(match (c1.get(), c2.get(), l1.get(), l2.get()) {
        (Some(c1), Some(c2), _, _) => Some(CurvesCase::Circles(c1.circ2d()?, c2.circ2d()?)),
        (Some(c1), None, _, Some(l2)) => Some(CurvesCase::CircleLine(c1.circ2d()?, l2.lin2d())),
        (None, Some(c2), Some(l1), _) => Some(CurvesCase::CircleLine(c2.circ2d()?, l1.lin2d())),
        (None, None, Some(l1), Some(l2)) => Some(CurvesCase::Lines(l1.lin2d(), l2.lin2d())),
        _ => None,
    })
}

impl BisecAna {
    /// `Perform(Cu1, Cu2, P, V1, V2, Sense, jointype, Tolerance, oncurve)`: the bisector
    /// of two circles or lines (or trimmed ones) through `p`, in the sector of `v1`,
    /// `v2` and `sense`; conics whose size is below `tolerance` become lines.
    /// StdFail_NotDone for other curve types;
    /// `bisector.perform_curves(&c1, &c2, p, v1, v2, 1.0, JoinType::Arc, 1e-7, true)?`.
    #[allow(clippy::too_many_arguments)] // The native signature.
    pub fn perform_curves(
        &self,
        cu1: &Handle<dyn Geom2dCurve>,
        cu2: &Handle<dyn Geom2dCurve>,
        p: Point2,
        v1: Vector2,
        v2: Vector2,
        sense: f64,
        jointype: JoinType,
        tolerance: f64,
        oncurve: bool,
    ) -> Result<(), Failure> {
        let context: CurvesContext =
            curves_context(cu1, cu2, p, [v1, v2], sense, jointype, tolerance, oncurve)?;
        let Some(case) = classify(&basis_of(cu1), &basis_of(cu2))? else {
            return Err(Failure::with_type(
                "StdFail_NotDone",
                "Bisector_BisecAna::Perform: the curves are not circles or lines; expected Geom2d_Circle or Geom2d_Line bases",
                "",
            ));
        };
        let result: Option<Trimmed> = match case {
            CurvesCase::Circles(c1, c2) => context.bisect_circles(c1, c2)?,
            CurvesCase::CircleLine(c1, l2) => context.bisect_circle_line(c1, l2)?,
            CurvesCase::Lines(l1, l2) => Some(context.bisect_lines(l1, l2)?),
        };
        self.store(result);
        Ok(())
    }
}

// The tangents at the common point (the end of `cu1`, the start of `cu2`) and the
// reference vector, normal to the chord of the two points off the curve.
#[allow(clippy::too_many_arguments)] // The native arguments of Perform.
fn curves_context(
    cu1: &Handle<dyn Geom2dCurve>,
    cu2: &Handle<dyn Geom2dCurve>,
    apoint: Point2,
    vectors: [Vector2; 2],
    adirection: f64,
    ajointype: JoinType,
    tolerance: f64,
    oncurve: bool,
) -> Result<CurvesContext, Failure> {
    let (pnt1, mut tan1) = cu1.d1(cu1.last_parameter())?;
    let (pnt2, tan2) = cu2.d1(cu2.first_parameter())?;
    let mut line_bis_vec = Vector2::new(0., 0.);
    if !oncurve {
        line_bis_vec = Vector2::between(pnt1, pnt2);
        line_bis_vec.rotate(PI / 2.);
    }
    tan1.reverse();
    Ok(CurvesContext {
        apoint,
        afirstvector: vectors[0],
        asecondvector: vectors[1],
        adirection,
        ajointype,
        tolerance,
        oncurve,
        tan1,
        tan2,
        line_bis_vec,
        first_curve: cu1.clone(),
        second_curve: cu2.clone(),
    })
}
