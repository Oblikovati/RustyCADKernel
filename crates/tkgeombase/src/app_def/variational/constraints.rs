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
// Rust translation of ModelingData/TKGeomBase/AppDef/AppDef_Variational.cxx (Init, InitTthetaF, NotParallel)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The constraint tables of `AppDef_Variational`: the unit tangents and the
//! curvatures of the constrained points and the normal frames of their tangents.
use super::{ConstraintTables, Variational, construction};
use crate::app_par_curves::{Constraint, ConstraintCouple, LineTool};
use crate::native_failure::{array_failure, failure};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Vector2, Vector3};

/// The tangents (or curvatures) of one multipoint, 3D then 2D.
struct PointVectors {
    v3d: Array1<'static, Vector3>,
    v2d: Array1<'static, Vector2>,
}

// A table sized max(1, count); the native doubles and ints are uninitialized, the
// port's start at 0.
fn table<T: Default>(count: i32) -> Result<Array1<'static, T>, Failure> {
    Array1::with_bounds(1, count.max(1))
        .map_err(|error| array_failure(error, "AppDef_Variational::Init"))
}

fn store<T>(array: &mut Array1<'_, T>, index: i32, value: T) -> Result<(), Failure> {
    array
        .set_value(index, value)
        .map_err(|error| array_failure(error, "AppDef_Variational::Init"))
}

// The unit tangent of 3D point `jp3d`, whose curvature must be normal to it within
// Precision::Angular() (Standard_ConstructionError otherwise).
fn normal_tangent_3d(tangent: Vector3, curvature: Vector3, jp3d: i32) -> Result<Vector3, Failure> {
    let vt3d: Vector3 = tangent.normalized()?;
    if !curvature
        .normalized()?
        .is_normal(vt3d, precision::angular())?
    {
        return Err(construction(&format!(
            "3D curvature {jp3d} not normal to its tangent"
        )));
    }
    Ok(vt3d)
}

// The unit tangent of 2D point `jp2d`, at PI/2 of its curvature within
// Precision::Angular() (Standard_ConstructionError otherwise).
fn normal_tangent_2d(tangent: Vector2, curvature: Vector2, jp2d: i32) -> Result<Vector2, Failure> {
    let vt2d: Vector2 = tangent.normalized()?;
    if (curvature.angle(vt2d)?.abs() - std::f64::consts::FRAC_PI_2).abs() > precision::angular() {
        return Err(construction(&format!(
            "2D curvature {jp2d} not normal to its tangent"
        )));
    }
    Ok(vt2d)
}

// The table kind of a constraint: 0 pass, 1 tangency, 2 curvature; None for
// NoConstraint (not tabulated).
fn constraint_kind(constraint: Constraint) -> Option<i32> {
    match constraint {
        Constraint::NoConstraint => None,
        Constraint::PassPoint => Some(0),
        Constraint::TangencyPoint => Some(1),
        Constraint::CurvaturePoint => Some(2),
    }
}

/// `NotParallel(T, V)`: a V not parallel to T, from T by adding 1 to x, then y, then z.
fn not_parallel(t: Vector3) -> Option<Vector3> {
    let mut v: Vector3 = t;
    v.set_x(v.x() + 1.);
    if v.cross_magnitude(t) > 1.0e-12 {
        return Some(v);
    }
    v.set_y(v.y() + 1.);
    if v.cross_magnitude(t) > 1.0e-12 {
        return Some(v);
    }
    v.set_z(v.z() + 1.);
    (v.cross_magnitude(t) > 1.0e-12).then_some(v)
}

// F = (|T|^2 I - T T^t) theta for the curvature frames; the 2D formula has no z
// terms at all (adding the zero ones could flip the sign of a zero result).
fn curvature_frame(ndimen: i32, t: Vector3, theta: Vector3) -> Vector3 {
    let (xx, xy, yy) = (t.x().powf(2.), t.x() * t.y(), t.y().powf(2.));
    if ndimen == 2 {
        return Vector3::new(
            yy * theta.x() - xy * theta.y(),
            xx * theta.y() - xy * theta.x(),
            0.,
        );
    }
    let (xz, yz, zz) = (t.x() * t.z(), t.y() * t.z(), t.z().powf(2.));
    Vector3::new(
        (zz + yy) * theta.x() - xy * theta.y() - xz * theta.z(),
        (xx + zz) * theta.y() - xy * theta.x() - yz * theta.z(),
        (xx + yy) * theta.z() - xz * theta.x() - yz * theta.y(),
    )
}

impl Variational {
    /// `Init()`: the constraint tables of the current couples, then the
    /// over-constraint test and, when it passes, the smoothing criterion.
    pub(super) fn init(&mut self) -> Result<(), Failure> {
        let couples: Array1<'static, ConstraintCouple> = self.constraint_couples()?;
        let ntheta: i32 = 2 * self.nb_p2d + 6 * self.nb_p3d;
        self.tables = self.empty_tables(couples.length(), ntheta)?;
        let (mut index, mut jndex, mut cur_multy_point) = (1, 1, 1);
        for couple in couples.iter() {
            let Some(kind) = constraint_kind(couple.constraint()) else {
                // A NoConstraint couple takes no frame slot (natively curMultyPoint
                // goes down by ntheta, then up by ntheta at the end of the loop).
                continue;
            };
            store(&mut self.tables.types, index, couple.index())?;
            store(&mut self.tables.types, index + 1, kind)?;
            index += 2;
            self.constraint_point(couple, kind, &mut jndex, cur_multy_point)?;
            cur_multy_point += ntheta;
        }
        self.finish_init()
    }

    // A copy of the constraint couples; Standard_NullObject without them.
    fn constraint_couples(&self) -> Result<Array1<'static, ConstraintCouple>, Failure> {
        let couples = self.constraints.get().ok_or_else(|| {
            failure(
                "Standard_NullObject",
                "AppDef_Variational::Init: null constraints; expected a constraint array",
            )
        })?;
        Ok(couples.array1().clone())
    }

    // The constraint tables for `n` couples. myNbConstraints (every couple,
    // NoConstraint included) only sizes the tables.
    fn empty_tables(&self, n: i32, ntheta: i32) -> Result<ConstraintTables, Failure> {
        Ok(ConstraintTables {
            values: table(2 * self.dimension * n)?,
            types: table(2 * n)?,
            ttheta: table(ntheta * n)?,
            tfthet: table(ntheta * n)?,
            ..ConstraintTables::default()
        })
    }

    // OverConstraint Detection; the flag is only ever raised here.
    fn finish_init(&mut self) -> Result<(), Failure> {
        let max_seg: i32 = if self.options.with_cutting {
            self.options.max_segment
        } else {
            1
        };
        if self.over_constrained(self.options.max_degree, self.niv_cont, max_seg) {
            self.is_over_constr = true;
            self.is_created = false;
            return Ok(());
        }
        self.init_smooth_criterion()?;
        self.is_created = true;
        Ok(())
    }

    // The table entries of one pass (0), tangency (1) or curvature (2) point.
    fn constraint_point(
        &mut self,
        couple: &ConstraintCouple,
        kind: i32,
        jndex: &mut i32,
        cur: i32,
    ) -> Result<(), Failure> {
        let ipoint: i32 = couple.index();
        match kind {
            0 => {
                self.tables.nb_pass_points += 1;
                *jndex += 4 * self.nb_p2d + 6 * self.nb_p3d;
                Ok(())
            }
            1 => {
                self.tables.nb_tang_points += 1;
                let tangents: PointVectors = self.point_vectors(ipoint, false)?;
                self.tangent_entries(&tangents, jndex, cur, couple.constraint())
            }
            _ => {
                self.tables.nb_curv_points += 1;
                let tangents: PointVectors = self.point_vectors(ipoint, false)?;
                let curvatures: PointVectors = self.point_vectors(ipoint, true)?;
                self.curvature_entries((&tangents, &curvatures), jndex, cur, couple.constraint())
            }
        }
    }

    // The tangents (or curvatures) of `ipoint` through the overload of the dimensions;
    // Standard_ConstructionError when the multipoint has none.
    fn point_vectors(&self, ipoint: i32, curvature: bool) -> Result<PointVectors, Failure> {
        let mut vectors = PointVectors {
            v3d: table(self.nb_p3d)?,
            v2d: table(self.nb_p2d)?,
        };
        if !self.read_point_vectors(ipoint, curvature, &mut vectors)? {
            let what: &str = if curvature { "curvatures" } else { "tangents" };
            return Err(construction(&format!(
                "point {ipoint} has no {what}; expected them for its constraint"
            )));
        }
        Ok(vectors)
    }

    // The tangents (or curvatures) of `ipoint` read by the line overload of the
    // dimensions it has; false when the multipoint has none.
    fn read_point_vectors(
        &self,
        ipoint: i32,
        curvature: bool,
        vectors: &mut PointVectors,
    ) -> Result<bool, Failure> {
        let (v3d, v2d) = (&mut vectors.v3d, &mut vectors.v2d);
        Ok(match (self.nb_p3d != 0, self.nb_p2d != 0, curvature) {
            (false, true, false) => self.ssp.tangency_2d(ipoint, v2d)?,
            (true, false, false) => self.ssp.tangency_3d(ipoint, v3d)?,
            (true, true, false) => self.ssp.tangency(ipoint, v3d, v2d)?,
            (false, true, true) => self.ssp.curvature_2d(ipoint, v2d)?,
            (true, false, true) => self.ssp.curvature_3d(ipoint, v3d)?,
            (true, true, true) => self.ssp.curvature(ipoint, v3d, v2d)?,
            (false, false, _) => true,
        })
    }

    // Tangency: the unit tangents, 3 of 6 entries per 3D point and 2 of 4 per 2D one.
    fn tangent_entries(
        &mut self,
        tangents: &PointVectors,
        jndex: &mut i32,
        cur: i32,
        kind: Constraint,
    ) -> Result<(), Failure> {
        for jp3d in 1..=self.nb_p3d {
            let vt3d: Vector3 = tangents.v3d[jp3d].normalized()?;
            self.put(jndex, &[vt3d.x(), vt3d.y(), vt3d.z()])?;
            *jndex += 3;
            self.init_ttheta_f(3, kind, cur + (jp3d - 1) * 6, *jndex - 6)?;
        }
        for jp2d in 1..=self.nb_p2d {
            let vt2d: Vector2 = tangents.v2d[jp2d].normalized()?;
            self.put(jndex, &[vt2d.x(), vt2d.y()])?;
            *jndex += 2;
            self.init_ttheta_f(2, kind, cur + (jp2d - 1) * 2 + self.nb_p3d * 6, *jndex - 4)?;
        }
        Ok(())
    }

    // Curvature: the unit tangent then the curvature, which must be normal to it
    // within Precision::Angular() (Standard_ConstructionError otherwise).
    fn curvature_entries(
        &mut self,
        (tangents, curvatures): (&PointVectors, &PointVectors),
        jndex: &mut i32,
        cur: i32,
        kind: Constraint,
    ) -> Result<(), Failure> {
        for jp3d in 1..=self.nb_p3d {
            let vc3d: Vector3 = curvatures.v3d[jp3d];
            let vt3d: Vector3 = normal_tangent_3d(tangents.v3d[jp3d], vc3d, jp3d)?;
            self.put(
                jndex,
                &[vt3d.x(), vt3d.y(), vt3d.z(), vc3d.x(), vc3d.y(), vc3d.z()],
            )?;
            self.init_ttheta_f(3, kind, cur + (jp3d - 1) * 6, *jndex - 6)?;
        }
        for jp2d in 1..=self.nb_p2d {
            let vc2d: Vector2 = curvatures.v2d[jp2d];
            let vt2d: Vector2 = normal_tangent_2d(tangents.v2d[jp2d], vc2d, jp2d)?;
            self.put(jndex, &[vt2d.x(), vt2d.y(), vc2d.x(), vc2d.y()])?;
            self.init_ttheta_f(2, kind, cur + (jp2d - 1) * 2 + self.nb_p3d * 6, *jndex - 4)?;
        }
        Ok(())
    }

    // myTabConstraints(jndex++) = value, for each value.
    fn put(&mut self, jndex: &mut i32, values: &[f64]) -> Result<(), Failure> {
        for &value in values {
            store(&mut self.tables.values, *jndex, value)?;
            *jndex += 1;
        }
        Ok(())
    }

    /// `InitTthetaF(ndimen, typcon, begin, jndex)`: the unit normals theta1 (and
    /// theta2 in 3D) of the tangent at `myTabConstraints(jndex)`, and for a curvature
    /// point their images by the curvature projector; false for a dimension other
    /// than 2 or 3, or a 3D tangent every NotParallel attempt is parallel to.
    fn init_ttheta_f(
        &mut self,
        ndimen: i32,
        typcon: Constraint,
        begin: i32,
        jndex: i32,
    ) -> Result<bool, Failure> {
        if !(2..=3).contains(&ndimen) {
            return Ok(false);
        }
        if !matches!(
            typcon,
            Constraint::TangencyPoint | Constraint::CurvaturePoint
        ) {
            return Ok(true);
        }
        let z: f64 = if ndimen == 3 {
            self.tables.value(jndex + 2)?
        } else {
            0.
        };
        let t = Vector3::new(self.tables.value(jndex)?, self.tables.value(jndex + 1)?, z);
        let v: Vector3 = if ndimen == 2 {
            Vector3::new(0., 0., 1.)
        } else {
            let Some(v) = not_parallel(t) else {
                return Ok(false);
            };
            v
        };
        let theta1: Vector3 = (v ^ t).normalized()?;
        let mut frames: Vec<Vector3> = vec![theta1];
        if ndimen == 3 {
            frames.push((t ^ theta1).normalized()?);
        }
        self.store_frames(ndimen, begin, &frames, false)?;
        if typcon == Constraint::CurvaturePoint {
            let images: Vec<Vector3> = frames
                .iter()
                .map(|&theta| curvature_frame(ndimen, t, theta))
                .collect();
            self.store_frames(ndimen, begin, &images, true)?;
        }
        Ok(true)
    }

    // The frames at `begin`: x and y in 2D, x, y, z of each frame in 3D.
    fn store_frames(
        &mut self,
        ndimen: i32,
        begin: i32,
        frames: &[Vector3],
        tfthet: bool,
    ) -> Result<(), Failure> {
        let target: &mut Array1<'static, f64> = if tfthet {
            &mut self.tables.tfthet
        } else {
            &mut self.tables.ttheta
        };
        let values: Vec<f64> = if ndimen == 2 {
            vec![frames[0].x(), frames[0].y()]
        } else {
            frames.iter().flat_map(|f| [f.x(), f.y(), f.z()]).collect()
        };
        for (value, index) in values.into_iter().zip(begin..) {
            store(target, index, value)?;
        }
        Ok(())
    }
}
