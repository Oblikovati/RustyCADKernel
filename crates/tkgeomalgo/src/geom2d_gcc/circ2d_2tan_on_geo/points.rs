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
// Rust translation of the constructors through points of Geom2dGcc_Circ2d2TanOnGeo.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The circles through a point (tangent to a circle or a line) or through two
//! points, with their centre on a curve.
use super::{
    Circ2d2TanOnGeo, NB_SOL_MAX, circle_touch, line_qualifier, line_side_value, line_touch,
    on_curve_of, perform_bisector,
};
use crate::gcc_ana::solutions::qualify;
use crate::gcc_ana::{
    CircPnt2dBisec, LinPnt2dBisec, Pnt2dBisec, check_line_position, check_qualifier,
};
use crate::gcc_ent::{Position, QualifiedCirc, QualifiedLin};
use crate::gcc_int::IType;
use crate::geom2d_int::TheIntConicCurveOfGInter;
use crate::int_res2d::Domain;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Direction2, Line2, Point2};

const MEMBER: &str = "Geom2dGcc_Circ2d2TanOnGeo";

impl Circ2d2TanOnGeo {
    // Every centre of a done intersection (at most 8 solutions), then WellDone.
    fn each_centre(
        &mut self,
        intp: &TheIntConicCurveOfGInter,
        mut centre: impl FnMut(&mut Self, Point2, f64) -> Result<(), Failure>,
    ) -> Result<(), Failure> {
        if !intp.is_done() {
            return Ok(());
        }
        if !intp.is_empty()? {
            let mut j: i32 = 1;
            while j <= intp.nb_points()? && self.solutions.nbr_sol < NB_SOL_MAX {
                let point = intp.point(j)?;
                centre(self, point.value(), point.param_on_second())?;
                j += 1;
            }
        }
        self.solutions.well_done = true;
        Ok(())
    }

    /// `Geom2dGcc_Circ2d2TanOnGeo(GccEnt_QualifiedCirc Qualified1, Point2, OnCurv,
    /// Tolerance)`: the circle-point bisectors intersected with the curve; the radius
    /// is the distance to the point; `Circ2d2TanOnGeo::circ_pnt(qc, point, &on, 1e-9)?`.
    pub fn circ_pnt(
        qualified1: QualifiedCirc,
        point2: Point2,
        on_curv: &Geom2dAdaptorCurve,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty();
        check_qualifier(qualified1.qualifier(), MEMBER)?;
        let tol: f64 = tolerance.abs();
        let bis = CircPnt2dBisec::new(qualified1.qualified(), point2);
        if !bis.is_done() {
            return Ok(circ);
        }
        let mut intp = TheIntConicCurveOfGInter::new();
        let (c2, d2) = on_curve_of(on_curv, tol)?;
        let kinds = [IType::Cir, IType::Lin, IType::Ell, IType::Hpr];
        for i in 1..=bis.nb_solutions() {
            perform_bisector(
                &mut intp,
                &bis.this_solution(i)?,
                &kinds,
                (&c2, &d2),
                (tol, tol),
            )?;
            circ.each_centre(&intp, |circ, center, parcen3| {
                circ.circle_point_centre(qualified1, point2, center, parcen3, tol)
            })?;
        }
        Ok(circ)
    }

    // The solution at one centre, when it lies on the qualified side of the circle
    // (tests inclusive of the tolerance, as natively in this constructor).
    fn circle_point_centre(
        &mut self,
        qualified1: QualifiedCirc,
        point2: Point2,
        center: Point2,
        parcen3: f64,
        tol: f64,
    ) -> Result<(), Failure> {
        let c1: Circle2 = qualified1.qualified();
        let r1: f64 = c1.radius();
        let radius: f64 = center.distance(point2);
        let dist1: f64 = c1.location().distance(center);
        let ok: bool = match qualified1.qualifier() {
            Position::Enclosed => dist1 - r1 <= tol,
            Position::Outside => r1 - dist1 <= tol,
            Position::Enclosing | Position::Unqualified => true,
            Position::NoQualifier => false,
        };
        if !ok {
            return Ok(());
        }
        let (index, circle) = self.open(center, radius, parcen3)?;
        let distcc1: f64 = center.distance(c1.location());
        self.solutions.qualifier[0].set(
            index,
            qualify(qualified1.qualifier(), distcc1, radius, r1, tol, false),
        )?;
        self.solutions.qualifier[1].set(index, Position::NoQualifier)?;
        if dist1 <= tol && (radius - r1).abs() <= tol {
            self.solutions.tangency[0].same.set(index, 1)?;
        } else {
            let point: Point2 = circle_touch(c1.location(), center, radius)?;
            self.touch(0, index, &circle, point, elclib::parameter(&c1, point)?)?;
        }
        self.touch(1, index, &circle, point2, 0.0)
    }

    /// `Geom2dGcc_Circ2d2TanOnGeo(GccEnt_QualifiedLin Qualified1, Qualified2 (a point),
    /// OnCurv, Tolerance)`: the line-point bisector intersected with the curve; the
    /// radius is the distance to the line; `Circ2d2TanOnGeo::lin_pnt(ql, point, &on, 1e-9)?`.
    pub fn lin_pnt(
        qualified1: QualifiedLin,
        point2: Point2,
        on_curv: &Geom2dAdaptorCurve,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty();
        let tol: f64 = tolerance.abs();
        check_line_position(qualified1.qualifier(), MEMBER)?;
        let l1: Line2 = qualified1.qualified();
        let normal = Direction2::new(-l1.direction().y(), l1.direction().x())?;
        let bis = LinPnt2dBisec::new(l1, point2)?;
        if !bis.is_done() {
            return Ok(circ);
        }
        let mut intp = TheIntConicCurveOfGInter::new();
        let (c2, d2) = on_curve_of(on_curv, tol)?;
        perform_bisector(
            &mut intp,
            &bis.this_solution()?,
            &[IType::Lin, IType::Par],
            (&c2, &d2),
            (tol, tol),
        )?;
        circ.each_centre(&intp, |circ, center, parcen3| {
            circ.line_point_centre(qualified1, normal, point2, center, parcen3)
        })?;
        Ok(circ)
    }

    // The solution at one centre, when it lies on the qualified side of the line.
    fn line_point_centre(
        &mut self,
        qualified1: QualifiedLin,
        normal: Direction2,
        point2: Point2,
        center: Point2,
        parcen3: f64,
    ) -> Result<(), Failure> {
        let l1: Line2 = qualified1.qualified();
        let radius: f64 = l1.distance(center);
        let ok: bool = match qualified1.qualifier() {
            Position::Enclosed => line_side_value(&l1, center) <= 0.0,
            Position::Outside => line_side_value(&l1, center) >= 0.0,
            Position::Unqualified => true,
            _ => false,
        };
        if !ok {
            return Ok(());
        }
        let (index, circle) = self.open(center, radius, parcen3)?;
        self.solutions.qualifier[1].set(index, Position::NoQualifier)?;
        let dc2 = Direction2::from_xy(l1.location().xy() - center.xy())?;
        self.solutions.qualifier[0]
            .set(index, line_qualifier(qualified1.qualifier(), dc2, normal))?;
        let point1: Point2 = line_touch(&l1, center, radius)?;
        self.touch(0, index, &circle, point1, elclib::parameter(&l1, point1)?)?;
        self.touch(1, index, &circle, point2, 0.0)
    }

    /// `Geom2dGcc_Circ2d2TanOnGeo(Point1, Point2, OnCurv, Tolerance)`: the point
    /// bisector intersected with the curve; no solution for coincident points;
    /// `Circ2d2TanOnGeo::pnt_pnt(p1, p2, &on, 1e-9)?`.
    pub fn pnt_pnt(
        point1: Point2,
        point2: Point2,
        on_curv: &Geom2dAdaptorCurve,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty();
        let tol: f64 = tolerance.abs();
        let bis = Pnt2dBisec::new(point1, point2)?;
        if !bis.is_done() {
            return Ok(circ);
        }
        let mut intp = TheIntConicCurveOfGInter::new();
        let (c2, d2) = on_curve_of(on_curv, tol)?;
        if !bis.has_solution() {
            return Ok(circ);
        }
        intp.perform_line(bis.this_solution()?, &Domain::new(), &c2, &d2, tol, tol)?;
        circ.each_centre(&intp, |circ, center, parcen3| {
            let (index, circle) = circ.open(center, point2.distance(center), parcen3)?;
            circ.solutions.qualifier[0].set(index, Position::NoQualifier)?;
            circ.solutions.qualifier[1].set(index, Position::NoQualifier)?;
            circ.touch(0, index, &circle, point1, 0.0)?;
            circ.touch(1, index, &circle, point2, 0.0)
        })?;
        Ok(circ)
    }
}
