// Copyright (c) 1997-1999 Matra Datavision
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
// Rust translation of Perform(Path, Tol), Perform(Param, Tol), Choix and the static
// Penalite, EvalAngle and DistMini of
// ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_SectionPlacement.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The search of the best placement: the start of the path, the plane of a planar
//! section, the path end, the curve-curve extrema and the projection on the path,
//! each accepted by the distance and angle penalty of `Choix`.
use super::section_placement::GeomFillSectionPlacement;
use super::section_placement_axis::tangente;
use super::sweep_failures::live;
use crate::int_curve_surface::HInter;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeombase::extrema::{ExtCC, ExtPC, POnCurv};
use rustycad_tkmath::gp::{Axis3, Point3, Vector3};
use std::f64::consts::PI;

/// The native plane tolerance of the path ends (`IntTol`).
const INT_TOL: f64 = 1.0e-5;

/// The static `Penalite(angle, dist)`: grows with the relative distance and with
/// the inverse of the angle.
fn penalite(angle: f64, dist: f64) -> f64 {
    let mut penal = if dist < 1.0 {
        dist.sqrt()
    } else if dist < 2.0 {
        dist.powf(2.0)
    } else {
        dist + 2.0
    };
    if angle > 1.0e-3 {
        penal += 1.0 / angle - 2.0 / PI;
    } else {
        penal += 1.0e3;
    }
    penal
}

/// The static `EvalAngle(V1, V2)`: the angle folded into `[0, PI/2]`.
fn eval_angle(v1: Vector3, v2: Vector3) -> Result<f64, Failure> {
    let angle = v1.angle(v2)?;
    Ok(if angle > PI / 2.0 { PI - angle } else { angle })
}

/// The static `DistMini(Ext, C, Dist, Param)`: the smallest of the end distances and
/// the extrema; `param` stays when none is below RealLast().
fn dist_mini(ext: &ExtPC<'_>, c: &dyn Adaptor3dCurve, param: f64) -> Result<(f64, f64), Failure> {
    let (mut dist2, mut param) = (f64::MAX, param);
    let (dist1, dist_last, _, _) = ext.trimmed_square_distances();
    if dist1 < dist2 || dist_last < dist2 {
        (dist2, param) = if dist1 < dist_last {
            (dist1, c.first_parameter()?)
        } else {
            (dist_last, c.last_parameter()?)
        };
    }
    if ext.is_done() {
        for ii in 1..=ext.nb_ext()? {
            if ext.square_distance(ii)? < dist2 {
                dist2 = ext.square_distance(ii)?;
                param = ext.point(ii)?.parameter();
            }
        }
    }
    Ok((dist2.sqrt(), param))
}

/// The two points of a placement: on the path and on the section.
#[derive(Clone, Copy)]
struct PlacedPoints {
    pon_path: Point3,
    pon_sec: Point3,
}

impl GeomFillSectionPlacement {
    /// `Perform(Path, Tol)`: the best placement of the section on `path`;
    /// `sp.perform_on_path(&path, 1e-4)?`.
    pub fn perform_on_path(
        &mut self,
        path: &Handle<dyn Adaptor3dCurve>,
        tol: f64,
    ) -> Result<(), Failure> {
        let p = live(path, "GeomFill_SectionPlacement::Perform", "Path")?;
        if self.my_is_point {
            let projector = ExtPC::with_curve(self.my_point, p, precision::confusion())?;
            (self.dist, self.path_param) = dist_mini(&projector, p, self.path_param)?;
            self.angle_max = PI / 2.0;
        } else {
            self.perform_curve(path, p, tol)?;
        }
        self.done = true;
        Ok(())
    }

    /// `Perform(ParamOnPath, Tol)`: the placement at a given path parameter;
    /// `sp.perform_at(0.5, 1e-4)?`.
    pub fn perform_at(&mut self, param: f64, tol: f64) -> Result<(), Failure> {
        self.done = true;
        let path = self.law("GeomFill_SectionPlacement::Perform")?.get_curve();
        let p = live(&path, "GeomFill_SectionPlacement::Perform", "Path")?;
        self.path_param = param;
        if self.my_is_point {
            self.dist = p.value(self.path_param)?.distance(&self.my_point);
            self.angle_max = PI / 2.0;
        } else {
            self.sec_param = self.section_adaptor().first_parameter()?;
            self.place_at(p, tol)?;
            if self.isplan {
                self.angle_max = PI / 2.0 - self.angle_max;
            }
        }
        self.done = true;
        Ok(())
    }

    /// `VRef`: the section axis as a vector.
    fn vref(&self) -> Vector3 {
        Vector3::from(self.the_axe.direction())
    }

    /// The path point and tangent at `PathParam`, the distance to the section point
    /// at `SecParam` (improved by a projection when above `tol`) and `AngleMax`.
    fn place_at(&mut self, path: &dyn Adaptor3dCurve, tol: f64) -> Result<PlacedPoints, Failure> {
        let (pon_path, dp1) = tangente(path, self.path_param)?;
        let mut pon_sec = self.section_adaptor().value(self.sec_param)?;
        self.dist = pon_path.distance(&pon_sec);
        if self.dist > tol
            && let Some((dist, param)) = self.section_nearest(pon_path, self.sec_param)?
        {
            (self.dist, self.sec_param) = (dist, param);
            pon_sec = self.section_adaptor().value(self.sec_param)?;
        }
        self.angle_max = eval_angle(self.vref(), dp1)?;
        Ok(PlacedPoints { pon_path, pon_sec })
    }

    /// `myExt.Perform(P)` then `DistMini`, or `None` when the projection fails.
    fn section_nearest(&self, point: Point3, param: f64) -> Result<Option<(f64, f64)>, Failure> {
        let adp = self.section_adaptor();
        let mut ext = ExtPC::new();
        ext.initialize(
            adp,
            adp.first_parameter()?,
            adp.last_parameter()?,
            precision::confusion(),
        )?;
        ext.perform(point)?;
        if !ext.is_done() {
            return Ok(None);
        }
        Ok(Some(dist_mini(&ext, adp, param)?))
    }

    fn perform_curve(
        &mut self,
        path: &Handle<dyn Adaptor3dCurve>,
        p: &dyn Adaptor3dCurve,
        tol: f64,
    ) -> Result<(), Failure> {
        self.path_param = p.first_parameter()?;
        self.sec_param = self.section_adaptor().first_parameter()?;
        let placed = self.place_at(p, tol)?;
        if self.isplan {
            self.angle_max = PI / 2.0 - self.angle_max;
            return self.plane_search(path, p, tol, placed);
        }
        self.free_search(p, tol, placed)
    }

    /// `Choix(dist, angle)`: whether a candidate is better than the current one.
    pub(crate) fn choix(&self, dist: f64, angle: f64) -> bool {
        let evoldist = dist - self.dist;
        let evolangle = angle - self.angle_max;
        if evoldist < -self.gabarit {
            return true;
        }
        if evoldist.abs() < self.gabarit {
            if evolangle > 0.5 {
                return true;
            }
            let (current, candidate) = (self.dist / self.gabarit, dist / self.gabarit);
            if penalite(angle, candidate) < penalite(self.angle_max, current) {
                return true;
            }
        }
        false
    }
}

impl GeomFillSectionPlacement {
    /// The planar section: the path end on the section plane nearest to the axis
    /// location, else the nearest intersection of the path with the plane, else the
    /// path end nearer to the plane. The native `PonPath` updates of this branch are
    /// never read afterwards and are left out.
    fn plane_search(
        &mut self,
        path: &Handle<dyn Adaptor3dCurve>,
        p: &dyn Adaptor3dCurve,
        tol: f64,
        placed: PlacedPoints,
    ) -> Result<(), Failure> {
        let (loc, vref) = (self.the_axe.location(), self.vref());
        let mut dist_center = precision::infinite();
        let v1 = Vector3::between(placed.pon_path, loc);
        if v1.dot(vref).abs() <= INT_TOL {
            dist_center = v1.magnitude();
        }
        let plast = p.value(p.last_parameter()?)?;
        let v1 = Vector3::from(loc.xyz() - plast.xyz());
        if v1.dot(vref).abs() <= INT_TOL && v1.magnitude() < dist_center {
            dist_center = v1.magnitude();
            self.path_param = p.last_parameter()?;
        }
        let plan: Handle<GeomPlane> = Handle::new(GeomPlane::new(Axis3::from_normal(
            loc,
            self.the_axe.direction(),
        )?));
        let found = self.plane_intersections(path, p, &plan, dist_center)?;
        if !found {
            self.plane_end_fallback(p, tol, &plan)?;
        }
        Ok(())
    }

    /// The intersections of the path with the plane nearer than `dist_center`;
    /// false when the intersection fails or finds no point.
    fn plane_intersections(
        &mut self,
        path: &Handle<dyn Adaptor3dCurve>,
        p: &dyn Adaptor3dCurve,
        plan: &Handle<GeomPlane>,
        mut dist_center: f64,
    ) -> Result<bool, Failure> {
        let surface: Handle<dyn GeomSurface> = plan.clone().cast();
        let adplan: Handle<dyn Adaptor3dSurface> =
            Handle::new(GeomAdaptorSurface::with_surface(&surface)?).cast();
        let mut intersector = HInter::new();
        intersector.perform(path, &adplan)?;
        if !intersector.is_done() {
            return Ok(false);
        }
        let loc = self.the_axe.location();
        for ii in 1..=intersector.nb_points()? {
            let w = intersector.point(ii)?.w();
            let a_dist = p.value(w)?.distance(&loc);
            if a_dist < dist_center {
                dist_center = a_dist;
                self.path_param = w;
            }
        }
        Ok(intersector.nb_points()? != 0)
    }

    /// No intersection: the first path end when both ends are on the plane or the
    /// first is nearer, else the placement at the last end.
    fn plane_end_fallback(
        &mut self,
        p: &dyn Adaptor3dCurve,
        tol: f64,
        plan: &Handle<GeomPlane>,
    ) -> Result<(), Failure> {
        let (pf, pl) = (p.first_parameter()?, p.last_parameter()?);
        let plane = live(plan, "GeomFill_SectionPlacement::Perform", "plan")?.pln();
        let first_distance = plane.square_distance(p.value(pf)?);
        let last_distance = plane.square_distance(p.value(pl)?);
        let sq = precision::square_confusion();
        let both_on = first_distance.abs() < sq && last_distance.abs() < sq;
        if both_on || first_distance < last_distance {
            self.path_param = pf;
            return Ok(());
        }
        self.path_param = pl;
        self.place_at(p, tol)?;
        self.angle_max = PI / 2.0 - self.angle_max;
        Ok(())
    }
}

/// A curve-curve extremum: its distance and its points on the path and the section.
type CurveExtremum = (f64, POnCurv, POnCurv);

impl GeomFillSectionPlacement {
    /// The non-planar section: the projection of the start point, then the last
    /// path end, the curve-curve extrema and the projection on the path, each tried
    /// while the distance is above `tol`. `taux` carries the last projection
    /// parameter from one step to the next, as natively.
    fn free_search(
        &mut self,
        p: &dyn Adaptor3dCurve,
        tol: f64,
        placed: PlacedPoints,
    ) -> Result<(), Failure> {
        let mut taux = 0.0;
        if let Some((distaux, t)) = self.section_nearest(placed.pon_path, taux)? {
            taux = t;
            if distaux < self.dist {
                (self.dist, self.sec_param) = (distaux, taux);
            }
        }
        if self.dist > tol {
            taux = self.try_last_end(p, taux)?;
        }
        if self.dist <= tol {
            return Ok(());
        }
        let mut pon_sec = placed.pon_sec;
        if self.try_curve_extrema(p, tol, &mut pon_sec)? {
            return Ok(());
        }
        self.try_path_projection(p, pon_sec, taux)
    }

    /// The projection of the last path end on the section, kept when `Choix`
    /// accepts it; returns the projection parameter.
    fn try_last_end(&mut self, p: &dyn Adaptor3dCurve, taux: f64) -> Result<f64, Failure> {
        let last = p.last_parameter()?;
        let (pt, dp1) = tangente(p, last)?;
        let alpha = eval_angle(self.vref(), dp1)?;
        let Some((distaux, t)) = self.section_nearest(pt, taux)? else {
            return Ok(taux);
        };
        if self.choix(distaux, alpha) {
            (self.dist, self.sec_param, self.angle_max) = (distaux, t, alpha);
            self.path_param = last;
        }
        Ok(t)
    }

    /// The extrema between the path and the section (`Extrema_ExtCC` at the curve
    /// resolutions of `tol / 100`); true when `Choix` accepted one.
    fn try_curve_extrema(
        &mut self,
        p: &dyn Adaptor3dCurve,
        tol: f64,
        pon_sec: &mut Point3,
    ) -> Result<bool, Failure> {
        let mut trouve = false;
        for (distaux, p1, p2) in self.curve_extrema(p, tol)? {
            let (_, dp1) = tangente(p, p1.parameter())?;
            let alpha = eval_angle(self.vref(), dp1)?;
            if self.choix(distaux, alpha) {
                trouve = true;
                (self.dist, self.angle_max) = (distaux, alpha);
                (self.path_param, self.sec_param) = (p1.parameter(), p2.parameter());
                *pon_sec = *p2.value();
            }
        }
        Ok(trouve)
    }

    fn curve_extrema(
        &self,
        p: &dyn Adaptor3dCurve,
        tol: f64,
    ) -> Result<Vec<CurveExtremum>, Failure> {
        let adp = self.section_adaptor();
        let ext = ExtCC::with_ranges(
            p,
            adp,
            (p.first_parameter()?, p.last_parameter()?),
            (adp.first_parameter()?, adp.last_parameter()?),
            p.resolution(tol / 100.0)?,
            adp.resolution(tol / 100.0)?,
        )?;
        if !ext.is_done() || ext.is_parallel()? {
            return Ok(Vec::new());
        }
        (1..=ext.nb_ext()?)
            .map(|ii| {
                let (p1, p2) = ext.points(ii)?;
                Ok((ext.square_distance(ii)?.sqrt(), p1, p2))
            })
            .collect()
    }

    /// The projection of the section point on the path, kept when `Choix` accepts it.
    fn try_path_projection(
        &mut self,
        p: &dyn Adaptor3dCurve,
        pon_sec: Point3,
        taux: f64,
    ) -> Result<(), Failure> {
        let mut pext = ExtPC::new();
        pext.initialize(
            p,
            p.first_parameter()?,
            p.last_parameter()?,
            precision::confusion(),
        )?;
        pext.perform(pon_sec)?;
        if !pext.is_done() {
            return Ok(());
        }
        let (distaux, t) = dist_mini(&pext, p, taux)?;
        let (_, dp1) = tangente(p, t)?;
        let alpha = eval_angle(self.vref(), dp1)?;
        if self.choix(distaux, alpha) {
            (self.dist, self.angle_max, self.path_param) = (distaux, alpha, t);
        }
        Ok(())
    }
}
