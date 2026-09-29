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
// Rust translation of the line-line constructor of Geom2dGcc_Circ2d2TanOnGeo.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The circles tangent to two lines with their centre on a curve.
use super::{
    Circ2d2TanOnGeo, NB_SOL_MAX, line_qualifier, line_side_value, line_touch, on_curve_of,
};
use crate::gcc_ana::{Lin2dBisec, check_line_position};
use crate::gcc_ent::{Position, QualifiedLin};
use crate::geom2d_int::TheIntConicCurveOfGInter;
use crate::int_res2d::Domain;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Direction2, Line2, Point2};

const MEMBER: &str = "Geom2dGcc_Circ2d2TanOnGeo";

// The side test of a centre for a qualified line: enclosed on the left (<= 0),
// outside on the right (>= 0), unqualified anywhere.
fn side_ok(q: &QualifiedLin, center: Point2) -> bool {
    match q.qualifier() {
        Position::Enclosed => line_side_value(&q.qualified(), center) <= 0.0,
        Position::Outside => line_side_value(&q.qualified(), center) >= 0.0,
        Position::Unqualified => true,
        _ => false,
    }
}

impl Circ2d2TanOnGeo {
    /// `Geom2dGcc_Circ2d2TanOnGeo(GccEnt_QualifiedLin Qualified1, GccEnt_QualifiedLin
    /// Qualified2, OnCurv, Tolerance)`: the line bisectors intersected with the curve;
    /// the radius is the mean distance to both lines;
    /// `Circ2d2TanOnGeo::lin_lin(ql1, ql2, &on, 1e-9)?`.
    pub fn lin_lin(
        qualified1: QualifiedLin,
        qualified2: QualifiedLin,
        on_curv: &Geom2dAdaptorCurve,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty();
        check_line_position(qualified1.qualifier(), MEMBER)?;
        check_line_position(qualified2.qualifier(), MEMBER)?;
        let tol: f64 = tolerance.abs();
        let (l1, l2): (Line2, Line2) = (qualified1.qualified(), qualified2.qualified());
        let dnor1 = Direction2::new(-l1.direction().y(), l1.direction().x())?;
        let dnor2 = Direction2::new(-l2.direction().y(), l2.direction().x())?;
        let bis = Lin2dBisec::new(l1, l2)?;
        if !bis.is_done() {
            return Ok(circ);
        }
        let mut intp = TheIntConicCurveOfGInter::new();
        let (c2, d2) = on_curve_of(on_curv, tol)?;
        let d1 = Domain::new();
        for i in 1..=bis.nb_solutions()? {
            intp.perform_line(bis.this_solution(i)?, &d1, &c2, &d2, tol, tol)?;
            if !intp.is_done() {
                continue;
            }
            if !intp.is_empty()? {
                let mut j: i32 = 1;
                while j <= intp.nb_points()? && circ.solutions.nbr_sol < NB_SOL_MAX {
                    let point = intp.point(j)?;
                    circ.lines_centre(
                        [qualified1, qualified2],
                        [dnor1, dnor2],
                        point.value(),
                        point.param_on_second(),
                    )?;
                    j += 1;
                }
            }
            circ.solutions.well_done = true;
        }
        Ok(circ)
    }

    // The solution at one centre, when it lies on the qualified sides of both
    // lines.
    fn lines_centre(
        &mut self,
        qs: [QualifiedLin; 2],
        dnor: [Direction2; 2],
        center: Point2,
        parcen3: f64,
    ) -> Result<(), Failure> {
        if !side_ok(&qs[0], center) || !side_ok(&qs[1], center) {
            return Ok(());
        }
        let (l1, l2): (Line2, Line2) = (qs[0].qualified(), qs[1].qualified());
        let radius: f64 = (l1.distance(center) + l2.distance(center)) / 2.0;
        let (index, circle) = self.open(center, radius, parcen3)?;
        let dc1 = Direction2::from_xy(l1.location().xy() - center.xy())?;
        let dc2 = Direction2::from_xy(l2.location().xy() - center.xy())?;
        self.solutions.qualifier[0].set(index, line_qualifier(qs[0].qualifier(), dc1, dnor[0]))?;
        self.solutions.qualifier[1].set(index, line_qualifier(qs[1].qualifier(), dc2, dnor[1]))?;
        for (arg, line) in [l1, l2].into_iter().enumerate() {
            let point: Point2 = line_touch(&line, center, radius)?;
            self.touch(arg, index, &circle, point, elclib::parameter(&line, point)?)?;
        }
        Ok(())
    }
}
