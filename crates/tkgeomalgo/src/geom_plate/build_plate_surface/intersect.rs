// Copyright (c) 1996-1999 Matra Datavision
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
// Rust translation of Intersect and CalculNbPtsInit of
// ModelingAlgorithms/TKGeomAlgo/GeomPlate/GeomPlate_BuildPlateSurface.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The intersections of the 2d curves on the initial surface: the parameter zones
//! where points would duplicate or where the G1 constraints disagree.
use super::GeomPlateBuildPlateSurface;
use crate::geom_plate::GeomPlateCurveConstraint;
use crate::geom2d_int::GInter;
use crate::int_res2d::IntersectionPoint;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkmath::gp::{Point3, Vector2, Vector3};
use std::f64::consts::PI;

/// The parameter zones of each curve (index `i - 1`): `PntInter` holds the zones of
/// duplicated points, `PntG1G1` the zones of incompatible G1 constraints; each zone
/// is two values.
pub(super) type ZonesPerCurve = Vec<Vec<f64>>;

/// One intersection between curves `i` and `j` with their adaptors.
struct CurvePair<'a> {
    i: usize,
    j: usize,
    ci: &'a Geom2dAdaptorCurve,
    cj: &'a Geom2dAdaptorCurve,
    lin_i: &'a GeomPlateCurveConstraint,
    lin_j: &'a GeomPlateCurveConstraint,
    point: IntersectionPoint,
}

impl CurvePair<'_> {
    fn params(&self) -> (f64, f64) {
        (self.point.param_on_first(), self.point.param_on_second())
    }

    /// The `(Tol, A1)` of an incompatible zone: `100 * Tol3d`, or `100000 * Tol3d`
    /// when the 2d tangents are opposite within `TolAng`.
    fn zone_tolerance(&self, tol3d: f64, tol_ang: f64) -> Result<f64, Failure> {
        let (u1, u2) = self.params();
        let v1: Vector2 = live2d(&self.lin_i.curve2d_on_surf()?)?.d1(u1)?.1;
        let v2: Vector2 = live2d(&self.lin_j.curve2d_on_surf()?)?.d1(u2)?.1;
        let mut a1: f64 = v1.angle(v2)?;
        if a1 > PI / 2.0 {
            a1 = PI - a1;
        }
        Ok(if (a1.abs() - PI).abs() < tol_ang {
            100000.0 * tol3d
        } else {
            100.0 * tol3d
        })
    }
}

fn live2d(
    curve: &rustycad_tkernel::standard::Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve>,
) -> Result<&dyn rustycad_tkg2d::geom2d::Geom2dCurve, Failure> {
    crate::geom_plate::live(curve, "the 2d curve of a constraint")
}

/// Appends the zone `[u - half, u + half]`.
fn push_zone(zones: &mut [Vec<f64>], i: usize, u: f64, half: f64) {
    zones[i].push(u - half);
    zones[i].push(u + half);
}

impl GeomPlateBuildPlateSurface {
    /// Sets the number of points of each curve from its share of the total length
    /// (`CalculNbPtsInit`): `NbPtsOnCur` points per curve on average, times 1.2 for
    /// G0 and 0.7 for G2, at least 3.
    pub(super) fn calcul_nb_pts_init(&self) -> Result<(), Failure> {
        let nt_lin_cont: i32 = self.nt_lin_cont();
        let nt_point: i32 = self.nb_pts_on_cur * nt_lin_cont;
        let mut len_t: f64 = 0.0;
        for i in 1..=nt_lin_cont {
            len_t += self.lin(i)?.length()?;
        }
        for i in 1..=nt_lin_cont {
            let lin = self.lin(i)?;
            let share = |factor: Option<f64>| -> Result<i32, Failure> {
                let base: f64 = factor.map_or(f64::from(nt_point), |f| f * f64::from(nt_point));
                Ok((base * lin.length()? / len_t) as i32)
            };
            match lin.order() {
                0 => lin.set_nb_points(share(Some(1.2))?),
                1 => lin.set_nb_points(share(None)?),
                2 => lin.set_nb_points(share(Some(0.7))?),
                _ => {}
            }
            if lin.nb_points() < 3 {
                lin.set_nb_points(3);
            }
        }
        Ok(())
    }

    /// `CalculNbPtsInit` when there are curves and points per curve, then `Intersect`
    /// when there are curves: the zones to leave out of the discretisation.
    pub(super) fn prepare_intersections(&self) -> Result<(ZonesPerCurve, ZonesPerCurve), Failure> {
        let n: usize = self.lin_cont.len();
        if n != 0 && self.nb_pts_on_cur != 0 {
            self.calcul_nb_pts_init()?;
        }
        let (mut pnt_inter, mut pnt_g1g1) = (vec![Vec::new(); n], vec![Vec::new(); n]);
        if n != 0 {
            self.intersect(&mut pnt_inter, &mut pnt_g1g1)?;
        }
        Ok((pnt_inter, pnt_g1g1))
    }

    /// Intersects each 2d curve with itself and the next ones (`Intersect`): an
    /// intersection of close 3d points leaves out a zone of 3d length `Tol3d` on the
    /// first curve (and an incompatible G1-G1 or G0-G1 zone), one of distant 3d points
    /// leaves out a zone on both curves.
    pub(super) fn intersect(
        &self,
        pnt_inter: &mut ZonesPerCurve,
        pnt_g1g1: &mut ZonesPerCurve,
    ) -> Result<(), Failure> {
        let mut intersection = GInter::new();
        let nt: i32 = self.nt_lin_cont();
        for i in 1..=nt {
            let lin_i = self.lin(i)?;
            let ci = Geom2dAdaptorCurve::with_curve(&lin_i.curve2d_on_surf()?)?;
            for j in i..=nt {
                let lin_j = self.lin(j)?;
                let cj = Geom2dAdaptorCurve::with_curve(&lin_j.curve2d_on_surf()?)?;
                let tol: f64 = self.tol2d * 10.0;
                if i == j {
                    intersection.perform_self(&ci, tol, tol)?;
                } else {
                    intersection.perform_between(&ci, &cj, tol, tol)?;
                }
                if intersection.is_empty()? {
                    continue;
                }
                for k in 1..=intersection.nb_points()? {
                    let (iu, ju) = ((i - 1) as usize, (j - 1) as usize);
                    let point = intersection.point(k)?;
                    let pair = CurvePair {
                        i: iu,
                        j: ju,
                        ci: &ci,
                        cj: &cj,
                        lin_i,
                        lin_j,
                        point,
                    };
                    self.treat_intersection(&pair, pnt_inter, pnt_g1g1)?;
                }
            }
        }
        Ok(())
    }

    /// The zones of one intersection point.
    // The negated test keeps the native branch for a NaN value.
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    fn treat_intersection(
        &self,
        pair: &CurvePair<'_>,
        pnt_inter: &mut ZonesPerCurve,
        pnt_g1g1: &mut ZonesPerCurve,
    ) -> Result<(), Failure> {
        let (u1, u2) = pair.params();
        let p1: Point3 = pair.lin_i.d0(u1)?;
        let p2: Point3 = pair.lin_j.d0(u2)?;
        let dist: f64 = p1.distance(&p2);
        if !(dist < self.tol3d) {
            // 2d intersection of distant 3d points: a zone on both curves.
            push_zone(pnt_inter, pair.i, u1, pair.ci.resolution(dist)?);
            if pair.j != pair.i {
                push_zone(pnt_inter, pair.j, u2, pair.cj.resolution(dist)?);
            }
            return Ok(());
        }
        push_zone(pnt_inter, pair.i, u1, self.duplicate_zone(pair.ci, u1)?);
        let (order_i, order_j) = (pair.lin_i.order(), pair.lin_j.order());
        if order_i == 1 && order_j == 1 {
            self.g1_g1_zone(pair, pnt_g1g1)?;
        }
        if (order_i == 0 && order_j == 1) || (order_i == 1 && order_j == 0) {
            self.g0_g1_zone(pair, pnt_g1g1)?;
        }
        Ok(())
    }

    /// The half length of the zone around `u` on `ci` of 3d length `Tol3d`, at most
    /// 100 times the curve resolution.
    fn duplicate_zone(&self, ci: &Geom2dAdaptorCurve, u: f64) -> Result<f64, Failure> {
        let mut tolint: f64 = ci.resolution(self.tol3d)?;
        let (_, v2d) = ci.d1(u)?;
        let mut aux: f64 = v2d.magnitude();
        if aux > 1.0e-7 {
            aux = self.tol3d / aux;
            if aux > 100.0 * tolint {
                tolint *= 100.0;
            } else {
                tolint = aux;
            }
        } else {
            tolint *= 100.0;
        }
        Ok(tolint)
    }

    /// Two G1 constraints meeting with different normals or curvatures leave out a
    /// zone on both curves.
    fn g1_g1_zone(
        &self,
        pair: &CurvePair<'_>,
        pnt_g1g1: &mut ZonesPerCurve,
    ) -> Result<(), Failure> {
        let (u1, u2) = pair.params();
        let d2i = pair.lin_i.d2(u1)?;
        let d2j = pair.lin_j.d2(u2)?;
        let v16: Vector3 = d2i.d1u ^ d2i.d1v;
        let v26: Vector3 = d2j.d1u ^ d2j.d1v;
        let mut ant: f64 = v16.angle(v26)?;
        if ant > PI / 2.0 {
            ant = PI - ant;
        }
        // v15 and v25 are the fifth derivative outputs of D2, the mixed derivatives.
        let incompatible: bool = (v16 * d2i.d2uv - v16 * d2j.d2uv).abs() > self.tol3d / 1000.0
            || ant.abs() > self.tol3d / 1000.0;
        if !incompatible {
            return Ok(());
        }
        let tol: f64 = pair.zone_tolerance(self.tol3d, self.tol_ang)?;
        push_zone(pnt_g1g1, pair.i, u1, pair.ci.resolution(tol)?);
        push_zone(pnt_g1g1, pair.j, u2, pair.cj.resolution(tol)?);
        Ok(())
    }

    /// A G0 curve that is not tangent to the surface of the G1 one leaves out a zone
    /// on the G1 curve, larger with the angle.
    // The negated test keeps the native branch for a NaN value.
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    fn g0_g1_zone(
        &self,
        pair: &CurvePair<'_>,
        pnt_g1g1: &mut ZonesPerCurve,
    ) -> Result<(), Failure> {
        let (u1, u2) = pair.params();
        let i_is_g0: bool = pair.lin_i.order() == 0;
        let (vec, d1) = if i_is_g0 {
            (curve_tangent(pair.lin_i, u1)?, pair.lin_j.d1(u2)?)
        } else {
            (curve_tangent(pair.lin_j, u2)?, pair.lin_i.d1(u1)?)
        };
        let n: Vector3 = d1.d1u ^ d1.d1v;
        let angle: f64 = (PI / 2.0 - vec.angle(n)?).abs();
        if !(angle > self.tol_ang / 10.0) {
            return Ok(());
        }
        let tol: f64 = pair.zone_tolerance(self.tol3d, self.tol_ang)?;
        if i_is_g0 {
            let coin: f64 = pair.cj.resolution(tol)? * (angle / self.tol_ang * 10.0);
            push_zone(pnt_g1g1, pair.j, u2, coin);
        } else {
            let coin: f64 = pair.ci.resolution(tol)? * (angle / self.tol_ang * 10.0);
            push_zone(pnt_g1g1, pair.i, u1, coin);
        }
        Ok(())
    }
}

/// The tangent at `u` of the 3d curve of `lin` (`theCurve->D1`).
fn curve_tangent(lin: &GeomPlateCurveConstraint, u: f64) -> Result<Vector3, Failure> {
    let curve = lin.curve3d();
    Ok(
        crate::geom_plate::live(&curve, "the 3d curve of a constraint")?
            .d1(u)?
            .1,
    )
}
