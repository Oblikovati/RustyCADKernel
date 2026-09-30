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
// Rust translation of the "max flow" branch of ComputeSurfInit (a closed G1 contour) of
// ModelingAlgorithms/TKGeomAlgo/GeomPlate/GeomPlate_BuildPlateSurface.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The initial plane of a closed G1 contour: normal to the best vector of the
//! constraint normals when they lie in one half space.
use super::GeomPlateBuildPlateSurface;
use crate::geom_plate::aij::GeomPlateAij;
use crate::geom_plate::build_average_plane::GeomPlateBuildAveragePlane;
use rustycad_tkernel::ncollection::{HArray1, Sequence};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkmath::gp::{Point3, Vector3};

/// `LinTol` and `AngTol` of ComputeSurfInit.
pub(super) const LIN_TOL: f64 = 0.001;
pub(super) const ANG_TOL: f64 = 0.001;
/// The samples per curve and the step between normal samples.
const NB_POINT: i32 = 20;
const DISCR: i32 = NB_POINT / 4;

/// The sampled points and the normals of the half space search.
struct NormalCloud {
    pts: HArray1<Point3>,
    pnum: i32,
    vecs: Sequence<Vector3>,
    aset: Sequence<GeomPlateAij>,
}

impl NormalCloud {
    fn push(&mut self, p: Point3) -> Result<(), Failure> {
        self.pnum += 1;
        self.pts
            .change_array1()
            .set_value(self.pnum, p)
            .map_err(errors::array_bounds)
    }

    /// Whether `normal` differs from every kept normal.
    fn is_new(&self, normal: Vector3, others: &Sequence<Vector3>) -> Result<bool, Failure> {
        for list in [&self.vecs, others] {
            for v in list.iter() {
                if v.is_equal(normal, LIN_TOL, ANG_TOL)? {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    fn half_space(&mut self, new_vecs: &Sequence<Vector3>) -> Result<bool, Failure> {
        GeomPlateBuildAveragePlane::half_space(
            new_vecs,
            &mut self.vecs,
            &mut self.aset,
            LIN_TOL,
            ANG_TOL,
        )
    }
}

impl GeomPlateBuildPlateSurface {
    /// The start of curve `i` in its sense and the signed length of its range.
    pub(super) fn sensed_range(&self, i: i32) -> Result<(f64, f64), Failure> {
        let lin = self.lin(i)?;
        let (mut uinit, ufinal) = (lin.first_parameter()?, lin.last_parameter()?);
        let mut uif: f64 = ufinal - uinit;
        if self.sense_at(i)? == 1 {
            uinit = ufinal;
            uif = -uif;
        }
        Ok((uinit, uif))
    }

    /// Samples curve `i` (21 points, a normal every fifth for G1 curves) and adds its
    /// new normals to the half space; `last_vec` is the last normal of the previous
    /// curve. False when the half space breaks.
    fn sample_curve_normals(
        &self,
        i: i32,
        cloud: &mut NormalCloud,
        last_vec: &mut Vector3,
    ) -> Result<bool, Failure> {
        let lin = self.lin(i)?;
        let order: i32 = lin.order();
        let (uinit, uif) = self.sensed_range(i)?;
        let mut new_vecs: Sequence<Vector3> = Sequence::default();
        let mut normal = Vector3::new(0.0, 0.0, 0.0);
        let mut to_reverse: bool = false;
        if i > 1 && order >= 1 {
            let d1 = lin.d1(uinit)?;
            normal = d1.d1u ^ d1.d1v;
            to_reverse = last_vec.is_opposite(normal, ANG_TOL)?;
        }
        for j in 0..=NB_POINT {
            let inter: f64 = f64::from(j) * uif / f64::from(NB_POINT);
            if order < 1 || j % DISCR != 0 {
                cloud.push(lin.d0(uinit + inter)?)?;
                continue;
            }
            let d1 = lin.d1(uinit + inter)?;
            cloud.push(d1.point)?;
            normal = d1.d1u ^ d1.d1v;
            normal.normalize()?;
            if to_reverse {
                normal.reverse();
            }
            if cloud.is_new(normal, &new_vecs)? {
                new_vecs.append(normal);
            }
        }
        if order < 1 {
            return Ok(true);
        }
        let is_half_space: bool = cloud.half_space(&new_vecs)?;
        if is_half_space {
            *last_vec = normal;
        }
        Ok(is_half_space)
    }

    /// Samples point constraint `i` and adds its normal (or the reversed one) to the
    /// half space. False when the half space breaks.
    fn sample_point_normal(&self, i: i32, cloud: &mut NormalCloud) -> Result<bool, Failure> {
        let pnt = self.pnt(i)?;
        if pnt.order() < 1 {
            cloud.push(pnt.d0())?;
            return Ok(true);
        }
        let (p, v1, v2) = pnt.d1();
        cloud.push(p)?;
        let mut normal: Vector3 = v1 ^ v2;
        normal.normalize()?;
        if !cloud.is_new(normal, &Sequence::default())? {
            return Ok(true);
        }
        let mut new_vecs: Sequence<Vector3> = Sequence::default();
        new_vecs.append(normal);
        if cloud.half_space(&new_vecs)? {
            return Ok(true);
        }
        new_vecs.change_value(1)?.reverse();
        cloud.half_space(&new_vecs)
    }

    /// The initial plane of a closed G1 contour: normal to the best vector of the
    /// normals sampled on the curves and points, enlarged to three times the box of
    /// the points. False (nothing set) when the normals leave a half space.
    pub(super) fn surf_init_from_normals(&mut self) -> Result<bool, Failure> {
        let size: i32 = (NB_POINT + 1) * self.nt_lin_cont() + self.nt_pnt_cont();
        let mut cloud = NormalCloud {
            pts: HArray1::with_bounds(1, size).map_err(errors::array_bounds)?,
            pnum: 0,
            vecs: Sequence::default(),
            aset: Sequence::default(),
        };
        let mut last_vec = Vector3::new(0.0, 0.0, 0.0);
        for i in 1..=self.nt_lin_cont() {
            if !self.sample_curve_normals(i, &mut cloud, &mut last_vec)? {
                return Ok(false);
            }
        }
        for i in 1..=self.nt_pnt_cont() {
            if !self.sample_point_normal(i, &mut cloud)? {
                return Ok(false);
            }
        }
        // Remove the null normals (inside the cone of the others).
        let kept: Vec<Vector3> = cloud
            .vecs
            .iter()
            .copied()
            .filter(|v| v.square_magnitude() != 0.0)
            .collect();
        let mut vecs: Sequence<Vector3> = Sequence::default();
        kept.into_iter().for_each(|v| {
            vecs.append(v);
        });
        let bap = GeomPlateBuildAveragePlane::with_normals(&vecs, &Handle::new(cloud.pts))?;
        self.set_surf_init_from_plane(&bap)?;
        Ok(true)
    }
}
