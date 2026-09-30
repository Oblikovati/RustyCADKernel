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
// Rust translation of the solver selection of Geom2dGcc_Circ2d2TanOn.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The choice between GccAna_Circ2d2TanOn, Geom2dGcc_Circ2d2TanOnGeo and
//! Geom2dGcc_Circ2d2TanOnIter from the curve types.
use super::Circ2d2TanOn;
use crate::gcc_ana::Circ2d2TanOn as Ana;
use crate::gcc_ent::{QualifiedCirc, QualifiedLin};
use crate::geom2d_gcc::{
    Circ2d2TanOnGeo, Circ2d2TanOnIter, QualifiedCurve, circle_of, line_of, point_of,
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dPoint;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Circle2, Line2, Point2};

// A line or circle argument as GccAna takes it.
#[derive(Clone, Copy)]
enum Analytic {
    Circ(QualifiedCirc),
    Lin(QualifiedLin),
}

fn analytic_of(q: &QualifiedCurve) -> Result<Option<Analytic>, Failure> {
    Ok(match q.curve().get_type()? {
        CurveType::Circle => Some(Analytic::Circ(QualifiedCirc::new(
            circle_of(q.curve())?,
            q.qualifier(),
        ))),
        CurveType::Line => Some(Analytic::Lin(QualifiedLin::new(
            line_of(q.curve())?,
            q.qualifier(),
        ))),
        _ => None,
    })
}

// A line or circle locus.
#[derive(Clone, Copy)]
enum Locus {
    Circ(Circle2),
    Lin(Line2),
}

fn locus_of(on_curve: &Geom2dAdaptorCurve) -> Result<Option<Locus>, Failure> {
    Ok(match on_curve.get_type()? {
        CurveType::Circle => Some(Locus::Circ(circle_of(on_curve)?)),
        CurveType::Line => Some(Locus::Lin(line_of(on_curve)?)),
        _ => None,
    })
}

impl Circ2d2TanOn {
    // GccAna (line or circle locus) or Geom2dGcc_Circ2d2TanOnGeo for two lines or
    // circles; false for another curve. A line first and a circle second go to the
    // solvers in the other order (Invert, set once done).
    pub(super) fn two_analytic(
        &mut self,
        qs: [&QualifiedCurve; 2],
        on_curve: &Geom2dAdaptorCurve,
        tol: f64,
    ) -> Result<bool, Failure> {
        use Analytic::{Circ, Lin};
        let (Some(a1), Some(a2)) = (analytic_of(qs[0])?, analytic_of(qs[1])?) else {
            return Ok(false);
        };
        let Some(locus) = locus_of(on_curve)? else {
            self.two_geometric((a1, a2), on_curve, tol)?;
            return Ok(true);
        };
        let ana = match (a1, a2, locus) {
            (Circ(c1), Circ(c2), Locus::Circ(on)) => Ana::circ_circ_on_circ(c1, c2, on, tol)?,
            (Circ(c1), Circ(c2), Locus::Lin(on)) => Ana::circ_circ_on_line(c1, c2, on, tol)?,
            (Circ(c1), Lin(l2), Locus::Circ(on)) => Ana::circ_lin_on_circ(c1, l2, on, tol)?,
            (Circ(c1), Lin(l2), Locus::Lin(on)) => Ana::circ_lin_on_line(c1, l2, on, tol)?,
            (Lin(l1), Circ(c2), Locus::Circ(on)) => Ana::circ_lin_on_circ(c2, l1, on, tol)?,
            (Lin(l1), Circ(c2), Locus::Lin(on)) => Ana::circ_lin_on_line(c2, l1, on, tol)?,
            (Lin(l1), Lin(l2), Locus::Circ(on)) => Ana::lin_lin_on_circ(l1, l2, on, tol)?,
            (Lin(l1), Lin(l2), Locus::Lin(on)) => Ana::lin_lin_on_line(l1, l2, on, tol)?,
        };
        self.take(ana.is_done(), || ana.nb_solutions(), &ana)?;
        self.invert = matches!((a1, a2), (Lin(_), Circ(_))) && self.solutions.well_done;
        Ok(true)
    }

    // Geom2dGcc_Circ2d2TanOnGeo for two lines or circles on another curve.
    fn two_geometric(
        &mut self,
        args: (Analytic, Analytic),
        on_curve: &Geom2dAdaptorCurve,
        tol: f64,
    ) -> Result<(), Failure> {
        use Analytic::{Circ, Lin};
        let geo = match args {
            (Circ(c1), Circ(c2)) => Circ2d2TanOnGeo::circ_circ(c1, c2, on_curve, tol)?,
            (Circ(c1), Lin(l2)) => Circ2d2TanOnGeo::circ_lin(c1, l2, on_curve, tol)?,
            (Lin(l1), Circ(c2)) => Circ2d2TanOnGeo::circ_lin(c2, l1, on_curve, tol)?,
            (Lin(l1), Lin(l2)) => Circ2d2TanOnGeo::lin_lin(l1, l2, on_curve, tol)?,
        };
        self.take(geo.is_done(), || geo.nb_solutions(), &geo)?;
        self.invert = matches!(args, (Lin(_), Circ(_))) && self.solutions.well_done;
        Ok(())
    }

    // For two curves that are not both lines or circles, the iteration on a line or
    // circle locus (a circle one copies no qualifiers, as natively).
    pub(super) fn iter_on_line_or_circle(
        &mut self,
        [q1, q2]: [&QualifiedCurve; 2],
        on_curve: &Geom2dAdaptorCurve,
        tol: f64,
        params: [f64; 3],
    ) -> Result<(), Failure> {
        match locus_of(on_curve)? {
            Some(Locus::Circ(on)) => {
                let iter = Circ2d2TanOnIter::curve_curve_on_circle(q1, q2, on, params, tol)?;
                self.store_iter(&iter, true, false)
            }
            Some(Locus::Lin(on)) => {
                let iter = Circ2d2TanOnIter::curve_curve_on_line(q1, q2, on, params, tol)?;
                self.store_iter(&iter, true, true)
            }
            None => Ok(()),
        }
    }

    // GccAna (line or circle locus) or Geom2dGcc_Circ2d2TanOnGeo for a line or
    // circle and a point; false for another curve.
    pub(super) fn one_analytic_point(
        &mut self,
        q1: &QualifiedCurve,
        point: &Handle<dyn Geom2dPoint>,
        on_curve: &Geom2dAdaptorCurve,
        tol: f64,
    ) -> Result<bool, Failure> {
        let Some(a1) = analytic_of(q1)? else {
            return Ok(false);
        };
        let Some(locus) = locus_of(on_curve)? else {
            let geo = match a1 {
                Analytic::Circ(c1) => {
                    Circ2d2TanOnGeo::circ_pnt(c1, point_of(point)?, on_curve, tol)?
                }
                Analytic::Lin(l1) => Circ2d2TanOnGeo::lin_pnt(l1, point_of(point)?, on_curve, tol)?,
            };
            self.take(geo.is_done(), || geo.nb_solutions(), &geo)?;
            return Ok(true);
        };
        let pnt: Point2 = point_of(point)?;
        let ana = match (a1, locus) {
            (Analytic::Circ(c1), Locus::Circ(on)) => Ana::circ_pnt_on_circ(c1, pnt, on, tol)?,
            (Analytic::Circ(c1), Locus::Lin(on)) => Ana::circ_pnt_on_line(c1, pnt, on, tol)?,
            (Analytic::Lin(l1), Locus::Circ(on)) => Ana::lin_pnt_on_circ(l1, pnt, on, tol)?,
            (Analytic::Lin(l1), Locus::Lin(on)) => Ana::lin_pnt_on_line(l1, pnt, on, tol)?,
        };
        self.take(ana.is_done(), || ana.nb_solutions(), &ana)?;
        Ok(true)
    }

    // The iteration of another curve and a point: on a line or circle locus (only
    // the first TheSame flag copied), else on the curve.
    pub(super) fn iter_with_point(
        &mut self,
        q1: &QualifiedCurve,
        point: Point2,
        on_curve: &Geom2dAdaptorCurve,
        tol: f64,
        [param1, param_on]: [f64; 2],
    ) -> Result<(), Failure> {
        let params = [param1, param_on];
        match locus_of(on_curve)? {
            Some(Locus::Circ(on)) => {
                let iter = Circ2d2TanOnIter::curve_pnt_on_circle(q1, point, on, params, tol)?;
                self.store_iter(&iter, false, true)
            }
            Some(Locus::Lin(on)) => {
                let iter = Circ2d2TanOnIter::curve_pnt_on_line(q1, point, on, params, tol)?;
                self.store_iter(&iter, false, true)
            }
            None => {
                let iter = Circ2d2TanOnIter::curve_pnt_on_curve(q1, point, on_curve, params, tol)?;
                self.store_iter(&iter, true, true)
            }
        }
    }

    // GccAna (line or circle locus) or Geom2dGcc_Circ2d2TanOnGeo for two points.
    pub(super) fn points_on(
        &mut self,
        point1: &Handle<dyn Geom2dPoint>,
        point2: &Handle<dyn Geom2dPoint>,
        on_curve: &Geom2dAdaptorCurve,
        tol: f64,
    ) -> Result<(), Failure> {
        let (p1, p2): (Point2, Point2) = (point_of(point1)?, point_of(point2)?);
        match locus_of(on_curve)? {
            Some(Locus::Circ(on)) => {
                let ana = Ana::pnt_pnt_on_circ(p1, p2, on, tol)?;
                self.take(ana.is_done(), || ana.nb_solutions(), &ana)
            }
            Some(Locus::Lin(on)) => {
                let ana = Ana::pnt_pnt_on_line(p1, p2, on, tol)?;
                self.take(ana.is_done(), || ana.nb_solutions(), &ana)
            }
            None => {
                let geo = Circ2d2TanOnGeo::pnt_pnt(p1, p2, on_curve, tol)?;
                self.take(geo.is_done(), || geo.nb_solutions(), &geo)
            }
        }
    }
}
