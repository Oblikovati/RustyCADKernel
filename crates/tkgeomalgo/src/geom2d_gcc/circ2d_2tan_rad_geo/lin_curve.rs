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
// Rust translation of the line-curve constructor of Geom2dGcc_Circ2d2TanRadGeo.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The circles of given radius tangent to a line and a curve.
use super::{Circ2d2TanRadGeo, NB_SOL_MAX, cotes, offset_of};
use crate::gcc_ana::{check_line_position, check_qualifier};
use crate::gcc_ent::{Position, QualifiedLin};
use crate::geom2d_gcc::domains::clamped_domain;
use crate::geom2d_gcc::{CurveTool, QCurve};
use crate::geom2d_int::TheIntConicCurveOfGInter;
use crate::int_res2d::Domain;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Direction2, Line2, Point2};

const MEMBER: &str = "Geom2dGcc_Circ2d2TanRadGeo";

impl Circ2d2TanRadGeo {
    /// `Geom2dGcc_Circ2d2TanRadGeo(GccEnt_QualifiedLin Qualified1, Qualified2, Radius,
    /// Tolerance)`: the line offset by the radius intersected with the offset curve;
    /// an unqualified line qualifies each solution by the side of its centre.
    /// GccEnt_BadQualifier for an enclosing line, Standard_NegativeValue for a
    /// negative radius; `Circ2d2TanRadGeo::lin_curve(ql, &q2, 1.0, 1e-9)?`.
    pub fn lin_curve(
        qualified1: QualifiedLin,
        qualified2: &QCurve,
        radius: f64,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let qualifiers_ok = check_line_position(qualified1.qualifier(), MEMBER)
            .and_then(|()| check_qualifier(qualified2.qualifier(), MEMBER));
        let l1: Line2 = qualified1.qualified();
        let (x1dir, y1dir) = (l1.direction().x(), l1.direction().y());
        let normal: Result<Direction2, Failure> = Direction2::new(-y1dir, x1dir);
        let mut circ = Self::start(qualifiers_ok, radius)?;
        let norm_l1: Direction2 = normal?;
        let tol: f64 = tolerance.abs();
        let (cote1, cote2) = cotes(qualified1.qualifier(), qualified2.qualifier(), radius);
        for &c1 in &cote1 {
            let point = Point2::from(l1.location().xy() + c1 * norm_l1.xy());
            // ligne avec deport.
            let line = Line2::new(point, l1.direction());
            let d1 = Domain::new();
            for &c2 in &cote2 {
                if circ.solutions.nbr_sol >= NB_SOL_MAX {
                    break;
                }
                let offset = offset_of(qualified2, c2)?;
                let d2: Domain = clamped_domain(&offset, tol)?;
                let intp = TheIntConicCurveOfGInter::line(line, &d1, &offset, &d2, tol, tol)?;
                circ.take_line_points(&intp, qualified1, qualified2, radius, norm_l1)?;
            }
        }
        Ok(circ)
    }

    // The centres of one line-offset intersection.
    fn take_line_points(
        &mut self,
        intp: &TheIntConicCurveOfGInter,
        qualified1: QualifiedLin,
        qualified2: &QCurve,
        radius: f64,
        norm_l1: Direction2,
    ) -> Result<(), Failure> {
        if !intp.is_done() {
            return Ok(());
        }
        let l1: Line2 = qualified1.qualified();
        let origin1: Point2 = l1.location();
        if !intp.is_empty()? {
            let mut i: i32 = 1;
            while i <= intp.nb_points()? && self.solutions.nbr_sol < NB_SOL_MAX {
                let point = intp.point(i)?;
                let center: Point2 = point.value();
                let dc1 = Direction2::from_xy(origin1.xy() - center.xy())?;
                let q1: Position = if !qualified1.is_unqualified() {
                    qualified1.qualifier()
                } else if dc1.dot(norm_l1) > 0.0 {
                    Position::Outside
                } else {
                    Position::Enclosed
                };
                let (pararg1, pararg2) = (point.param_on_first(), point.param_on_second());
                let pnttg1sol: Point2 = elclib::value(pararg1, &l1);
                let pnttg2sol: Point2 = CurveTool::value(qualified2.curve(), pararg2)?;
                let tangencies = [(pnttg1sol, pararg1), (pnttg2sol, pararg2)];
                self.push(center, radius, (q1, qualified2.qualifier()), tangencies)?;
                i += 1;
            }
        }
        self.solutions.well_done = true;
        Ok(())
    }
}
