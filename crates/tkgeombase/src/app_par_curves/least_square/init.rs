// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/AppParCurves/AppParCurves_LeastSquare.gxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AppParCurves_LeastSquare::Init` and `Affect`: the point matrix, the fixed
//! extremity poles, the unknown range `resinit..=resfin` and the extremity tangents
//! and curvatures read from the MultiLine.
use super::LeastSquare;
use crate::app_par_curves::arrays::{line_points, scratch};
use crate::app_par_curves::constraint::Constraint;
use crate::app_par_curves::line_tool::LineTool;
use crate::app_par_curves::multi_point::MultiPoint;
use crate::native_failure::array_failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};
use rustycad_tkmath::math::{Matrix, Vector};

// Row `row` of `m` from column `column`: the coordinates of each 3D then 2D point.
fn store_coordinates(
    m: &mut Matrix<'_>,
    row: i32,
    points: &Array1<'_, Point3>,
    points2d: &Array1<'_, Point2>,
    [nb_p, nb_p2d]: [i32; 2],
) -> Result<(), Failure> {
    let mut i2: i32 = 1;
    for i in 1..=nb_p {
        let p: Point3 = points[i];
        (
            *m.change_value(row, i2)?,
            *m.change_value(row, i2 + 1)?,
            *m.change_value(row, i2 + 2)?,
        ) = (p.x(), p.y(), p.z());
        i2 += 3;
    }
    for i in 1..=nb_p2d {
        let p: Point2 = points2d[i];
        (*m.change_value(row, i2)?, *m.change_value(row, i2 + 1)?) = (p.x(), p.y());
        i2 += 2;
    }
    Ok(())
}

// The same layout into a vector (the tangent and curvature vectors Vt, Vc).
fn store_vectors(
    v: &mut Vector<'_>,
    vectors: &Array1<'_, Vector3>,
    vectors2d: &Array1<'_, Vector2>,
    [nb_p, nb_p2d]: [i32; 2],
) -> Result<(), Failure> {
    let mut i2: i32 = 1;
    for i in 1..=nb_p {
        let t: Vector3 = vectors[i];
        (
            *v.change_value(i2)?,
            *v.change_value(i2 + 1)?,
            *v.change_value(i2 + 2)?,
        ) = (t.x(), t.y(), t.z());
        i2 += 3;
    }
    for i in 1..=nb_p2d {
        let t: Vector2 = vectors2d[i];
        (*v.change_value(i2)?, *v.change_value(i2 + 1)?) = (t.x(), t.y());
        i2 += 2;
    }
    Ok(())
}

// CheckTangents (3D): reverses every 3D and 2D tangent when one 3D tangent opposes
// the chord from pt1 to pt2 (the 3D and 2D tangents are assumed to agree).
fn check_tangents_3d(
    pt1: &Array1<'_, Point3>,
    pt2: &Array1<'_, Point3>,
    tg3d: &mut Array1<'_, Vector3>,
    tg2d: &mut Array1<'_, Vector2>,
) {
    if pt1.lower() != pt2.lower() || pt1.upper() != pt2.upper() || tg3d.length() != pt1.length() {
        return;
    }
    let reversed: bool =
        (pt1.lower()..=pt1.upper()).any(|i| Vector3::between(pt1[i], pt2[i]).dot(tg3d[i]) < 0.0);
    if reversed {
        tg3d.iter_mut().for_each(Vector3::reverse);
        tg2d.iter_mut().for_each(Vector2::reverse);
    }
}

// CheckTangents (2D): reverses each 2D tangent that opposes its chord.
fn check_tangents_2d(
    pt1: &Array1<'_, Point2>,
    pt2: &Array1<'_, Point2>,
    tg2d: &mut Array1<'_, Vector2>,
) {
    if pt1.lower() != pt2.lower() || pt1.upper() != pt2.upper() {
        return;
    }
    for i in pt1.lower()..=pt1.upper() {
        if Vector2::between(pt1[i], pt2[i]).dot(tg2d[i]) < 0.0 {
            tg2d[i].reverse();
        }
    }
}

impl LeastSquare {
    /// `Init(SSP, FirstPoint, LastPoint)`.
    pub(super) fn init(
        &mut self,
        ssp: &impl LineTool,
        first_point: i32,
        last_point: i32,
    ) -> Result<(), Failure> {
        self.iscalculated = false;
        self.isready = true;
        self.myfirstp = first_point;
        self.mylastp = last_point;
        self.first_p = super::the_first_point(self.first_constraint, first_point);
        self.last_p = super::the_last_point(self.last_constraint, last_point);
        self.nb_p2d = ssp.nb_p2d()?;
        self.nb_p = ssp.nb_p3d()?;
        self.deg = self.nbpoles - 1;
        self.flatten_knots()?;
        let (mut vt, mut vc) = (self.vec1t.clone(), self.vec1c.clone());
        let mut constraint: Constraint = self.first_constraint;
        self.affect(ssp, first_point, &mut constraint, [&mut vt, &mut vc])?;
        (self.first_constraint, self.vec1t, self.vec1c) = (constraint, vt, vc);
        let (mut vt, mut vc) = (self.vec2t.clone(), self.vec2c.clone());
        let mut constraint: Constraint = self.last_constraint;
        self.affect(ssp, last_point, &mut constraint, [&mut vt, &mut vc])?;
        (self.last_constraint, self.vec2t, self.vec2c) = (constraint, vt, vc);
        self.read_points(ssp)?;
        self.fixed_poles()?;
        self.unknown_range()
    }

    // The B-spline degree and the flat knots, when there are multiplicities.
    fn flatten_knots(&mut self) -> Result<(), Failure> {
        let (Some(knots), Some(mults)) = (&self.knots, &self.mults) else {
            return Ok(());
        };
        self.deg = mults.iter().sum::<i32>() - self.nbpoles - 1;
        let mut k: i32 = 1;
        for i in knots.lower()..=knots.upper() {
            for _ in 1..=*mults
                .value(i)
                .map_err(|error| array_failure(error, "AppParCurves_LeastSquare"))?
            {
                *self.vflatknots.change_value(k)? = knots[i];
                k += 1;
            }
        }
        Ok(())
    }

    // mypoints: the coordinates of every multipoint FirstPoint..=LastPoint.
    fn read_points(&mut self, ssp: &impl LineTool) -> Result<(), Failure> {
        let (mut points, mut points2d) = (
            scratch::<Point3>(self.nb_p)?,
            scratch::<Point2>(self.nb_p2d)?,
        );
        for j in self.myfirstp..=self.mylastp {
            line_points(ssp, j, &mut points, &mut points2d)?;
            store_coordinates(
                &mut self.mypoints,
                j,
                &points,
                &points2d,
                [self.nb_p, self.nb_p2d],
            )?;
        }
        Ok(())
    }

    // The multipoint of row `row` of mypoints.
    fn multi_point(&self, row: i32) -> Result<MultiPoint, Failure> {
        let pole: MultiPoint = MultiPoint::with_counts(self.nb_p, self.nb_p2d)?;
        let value = |k: i32| self.mypoints.value(row, k).copied();
        let mut i2: i32 = 1;
        for i in 1..=self.nb_p {
            pole.set_point(i, Point3::new(value(i2)?, value(i2 + 1)?, value(i2 + 2)?))?;
            i2 += 3;
        }
        for i in 1..=self.nb_p2d {
            pole.set_point2d(i + self.nb_p, Point2::new(value(i2)?, value(i2 + 1)?))?;
            i2 += 2;
        }
        Ok(pole)
    }

    // Pole1 and PoleN from the constrained extremities. The last one is also taken when
    // the *first* constraint is a curvature point (the native condition).
    fn fixed_poles(&mut self) -> Result<(), Failure> {
        let (first, last) = (self.first_constraint, self.last_constraint);
        let mut pole1: MultiPoint = MultiPoint::with_counts(self.nb_p, self.nb_p2d)?;
        let mut pole_n: MultiPoint = MultiPoint::with_counts(self.nb_p, self.nb_p2d)?;
        if first >= Constraint::PassPoint {
            pole1 = self.multi_point(self.myfirstp)?;
            self.copy_row(self.myfirstp, 1)?;
        }
        if last == Constraint::PassPoint
            || last == Constraint::TangencyPoint
            || first == Constraint::CurvaturePoint
        {
            pole_n = self.multi_point(self.mylastp)?;
            self.copy_row(self.mylastp, self.nbpoles)?;
        }
        self.scu.set_value(1, &pole1)?;
        if last != Constraint::NoConstraint {
            self.scu.set_value(self.nbpoles, &pole_n)?;
        }
        Ok(())
    }

    // mypoles(pole, :) = mypoints(point, :).
    fn copy_row(&mut self, point: i32, pole: i32) -> Result<(), Failure> {
        for i in 1..=self.mypoles.col_number() {
            *self.mypoles.change_value(pole, i)? = *self.mypoints.value(point, i)?;
        }
        Ok(())
    }

    // resinit/resfin from the constraints, and the system sizes NA, Nlignes, Ninc;
    // isready is false when no pole is left unknown.
    fn unknown_range(&mut self) -> Result<(), Failure> {
        self.resinit = self.first_constraint.code() + 1;
        self.resfin = self.nbpoles - self.last_constraint.code();
        let nincx: i32 = self.resfin - self.resinit + 1;
        if nincx < 1 {
            self.isready = false;
            return Ok(());
        }
        let neq: i32 = self.last_p - self.first_p + 1;
        self.na = 3 * self.nb_p + 2 * self.nb_p2d;
        self.nlignes = self.na * neq;
        self.ninc = self.na * nincx;
        if self.first_constraint >= Constraint::TangencyPoint {
            self.ninc += 1;
        }
        if self.last_constraint >= Constraint::TangencyPoint {
            self.ninc += 1;
        }
        Ok(())
    }

    /// `Affect(SSP, Index, Cons, Vt, Vc)`: reads the curvatures (a missing one lowers
    /// the constraint to a tangency point) and the tangents (a missing one lowers it
    /// to a pass point) of multipoint `index`, oriented along the chord to the next
    /// (or from the previous) multipoint.
    fn affect(
        &self,
        ssp: &impl LineTool,
        index: i32,
        cons: &mut Constraint,
        [vt, vc]: [&mut Vector<'_>; 2],
    ) -> Result<(), Failure> {
        if *cons < Constraint::TangencyPoint {
            return Ok(());
        }
        let counts: [i32; 2] = [self.nb_p, self.nb_p2d];
        let (mut tab_v, mut tab_v2d) = (
            scratch::<Vector3>(self.nb_p)?,
            scratch::<Vector2>(self.nb_p2d)?,
        );
        if *cons == Constraint::CurvaturePoint {
            if self.read_vectors(ssp, index, (&mut tab_v, &mut tab_v2d), true)? {
                store_vectors(vc, &tab_v, &tab_v2d, counts)?;
            } else {
                *cons = Constraint::TangencyPoint;
            }
        }
        if !self.read_vectors(ssp, index, (&mut tab_v, &mut tab_v2d), false)? {
            *cons = Constraint::PassPoint;
            return Ok(());
        }
        self.orient_tangents(ssp, index, &mut tab_v, &mut tab_v2d)?;
        store_vectors(vt, &tab_v, &tab_v2d, counts)
    }

    // Tangency (or Curvature) through the overload the counts select.
    fn read_vectors(
        &self,
        ssp: &impl LineTool,
        index: i32,
        (tab_v, tab_v2d): (&mut Array1<'_, Vector3>, &mut Array1<'_, Vector2>),
        curvature: bool,
    ) -> Result<bool, Failure> {
        match (self.nb_p != 0, self.nb_p2d != 0, curvature) {
            (true, true, false) => ssp.tangency(index, tab_v, tab_v2d),
            (true, true, true) => ssp.curvature(index, tab_v, tab_v2d),
            (false, true, false) => ssp.tangency_2d(index, tab_v2d),
            (false, true, true) => ssp.curvature_2d(index, tab_v2d),
            (_, false, false) => ssp.tangency_3d(index, tab_v),
            (_, false, true) => ssp.curvature_3d(index, tab_v),
        }
    }

    // CheckTangents on the chord from multipoint `index` to the next one (from the
    // previous one at the last multipoint).
    fn orient_tangents(
        &self,
        ssp: &impl LineTool,
        index: i32,
        tab_v: &mut Array1<'_, Vector3>,
        tab_v2d: &mut Array1<'_, Vector2>,
    ) -> Result<(), Failure> {
        let chord: [i32; 2] = if index < ssp.last_point()? {
            [index, index + 1]
        } else {
            [index - 1, index]
        };
        let (mut p1, mut p2) = (scratch::<Point3>(self.nb_p)?, scratch::<Point3>(self.nb_p)?);
        if self.nb_p != 0 {
            ssp.value_3d(chord[0], &mut p1)?;
            ssp.value_3d(chord[1], &mut p2)?;
            check_tangents_3d(&p1, &p2, tab_v, tab_v2d);
        } else if self.nb_p2d != 0 {
            let (mut q1, mut q2) = (
                scratch::<Point2>(self.nb_p2d)?,
                scratch::<Point2>(self.nb_p2d)?,
            );
            ssp.value(chord[0], &mut p1, &mut q1)?;
            ssp.value(chord[1], &mut p2, &mut q2)?;
            check_tangents_2d(&q1, &q2, tab_v2d);
        }
        Ok(())
    }
}
