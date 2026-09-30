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
// Rust translation of the static helpers IsInsideIn2d, IsInsideIn3d, IsSurfPlaneLike and
// DeleteByTube of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_WLineTool.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The tube criteria of `ComputePurgedWLine` (step III): a point is deleted when it
//! lies near the lines of the previous two points in both parametric spaces and in 3d.
use super::purge::{DELETE, PointsHash, STORE, VERTEX, fill_points_hash, make_new_wline};
use crate::int_patch::{IntPatchPointLine, IntPatchWLine};
use crate::int_surf::IntSurfPntOn2S;
use crate::intf::{std_max, std_min};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3, Xy};

/// Below this number of points the line is evened out (see `DeleteByTube`).
const MIN_NB_BAD_DISTR: i32 = 15;
/// The number of points of a line approximated by one Bezier curve.
pub(super) const NB_SINGLE_BEZIER: i32 = 30;

/// `IsInsideIn2d`: whether `next_pnt` lies in the tube of square radius
/// `square_max_dist` around the line of `base_pnt` and `base_vec`.
fn is_inside_in_2d(
    base_pnt: Point2,
    base_vec: Vector2,
    next_pnt: Point2,
    square_max_dist: f64,
) -> bool {
    let vec2d: Vector2 = Vector2::between(base_pnt, next_pnt);
    // d*d = (basevec^(nextpnt-basepnt))**2 / basevec**2
    let sq_mag: f64 = base_vec.square_magnitude();
    if sq_mag < precision::square_computational() {
        return false;
    }
    let cross: f64 = vec2d.crossed(base_vec);
    let square_dist: f64 = cross * cross / sq_mag;
    square_dist <= square_max_dist
}

/// `IsInsideIn3d`: the same test in 3d.
fn is_inside_in_3d(
    base_pnt: Point3,
    base_vec: Vector3,
    next_pnt: Point3,
    square_max_dist: f64,
) -> bool {
    let vec: Vector3 = Vector3::between(base_pnt, next_pnt);
    // d*d = (basevec^(nextpnt-basepnt))**2 / basevec**2
    let sq_mag: f64 = base_vec.square_magnitude();
    if sq_mag < precision::square_computational() {
        return false;
    }
    let square_dist: f64 = vec.cross_square_magnitude(base_vec) / sq_mag;
    square_dist <= square_max_dist
}

/// `IsSurfPlaneLike`: a plane, or a B-spline of degree 1 in both directions.
fn is_surf_plane_like(s: &Adaptor3dSurfaceHandle) -> Result<bool, Failure> {
    if s.get_type()? == SurfaceType::Plane {
        return Ok(true);
    }
    if s.get_type()? == SurfaceType::BSplineSurface && s.u_degree()? == 1 && s.v_degree()? == 1 {
        return Ok(true);
    }
    Ok(false)
}

/// The pipe of the tube test: the three last parameters on both surfaces (`UonS1`...)
/// and the base point, vector and step in 3d.
struct Pipe {
    uv1: [(f64, f64); 3],
    uv2: [(f64, f64); 3],
    base_pnt1: Point2,
    base_pnt2: Point2,
    base_vec1: Vector2,
    base_vec2: Vector2,
    base_3d_pnt: Point3,
    base_3d_vec: Vector3,
    prev_step: f64,
}

fn vec_of(a: (f64, f64), b: (f64, f64)) -> Vector2 {
    Vector2::new(b.0 - a.0, b.1 - a.1)
}

impl Pipe {
    /// The pipe of the first two points of the line.
    fn new(p1: &IntSurfPntOn2S, p2: &IntSurfPntOn2S) -> Self {
        let (a1, a2) = (p1.parameters_on_s1(), p2.parameters_on_s1());
        let (b1, b2) = (p1.parameters_on_s2(), p2.parameters_on_s2());
        let base_3d_vec = Vector3::between(p1.value(), p2.value());
        Self {
            uv1: [a1, a2, (0.0, 0.0)],
            uv2: [b1, b2, (0.0, 0.0)],
            base_pnt1: Point2::new(a1.0, a1.1),
            base_pnt2: Point2::new(b1.0, b1.1),
            base_vec1: vec_of(a1, a2),
            base_vec2: vec_of(b1, b2),
            base_3d_pnt: p1.value(),
            base_3d_vec,
            prev_step: base_3d_vec.square_magnitude(),
        }
    }

    /// The steps of the last two segments in both parametric spaces are close to each
    /// other (an uneven parametrization on one space keeps the point); `None` when the
    /// last step is null in one space (the point is kept, the pipe unchanged).
    fn even_steps(&self) -> Option<bool> {
        let s1 = [
            xy_of(self.uv1[0], self.uv1[1]),
            xy_of(self.uv1[1], self.uv1[2]),
        ];
        let s2 = [
            xy_of(self.uv2[0], self.uv2[1]),
            xy_of(self.uv2[1], self.uv2[2]),
        ];
        let sq_mod_s1: f64 = s1[1].square_modulus();
        let sq_mod_s2: f64 = s2[1].square_modulus();
        if sq_mod_s1 < precision::square_computational()
            || sq_mod_s2 < precision::square_computational()
        {
            return None;
        }
        let step_on_s1: f64 = s1[0].square_modulus() / sq_mod_s1;
        let step_on_s2: f64 = s2[0].square_modulus() / sq_mod_s2;
        // Check very rare case when wline fluctuates nearly one point and some of them may
        // be equal. Middle point will be deleted when such situation occurs.
        // bugs moddata_2 bug469.
        const LIMIT_COEFF: f64 = 0.99 * 0.99;
        Some(std_min(step_on_s1, step_on_s2) >= LIMIT_COEFF * std_max(step_on_s1, step_on_s2))
    }

    /// The pipe moves to the last two points `i - 1` and `i`.
    fn advance(&mut self, prev: Point3, cur: Point3) {
        self.uv1[0] = self.uv1[1];
        self.uv2[0] = self.uv2[1];
        self.uv1[1] = self.uv1[2];
        self.uv2[1] = self.uv2[2];
        self.base_pnt1 = Point2::new(self.uv1[0].0, self.uv1[0].1);
        self.base_pnt2 = Point2::new(self.uv2[0].0, self.uv2[0].1);
        self.base_vec1 = vec_of(self.uv1[0], self.uv1[1]);
        self.base_vec2 = vec_of(self.uv2[0], self.uv2[1]);
        self.base_3d_pnt = prev;
        self.base_3d_vec = Vector3::between(prev, cur);
        self.prev_step = self.base_3d_vec.square_magnitude();
    }
}

fn xy_of(a: (f64, f64), b: (f64, f64)) -> Xy {
    Xy::new(b.0 - a.0, b.1 - a.1)
}

/// The tolerances of the tube (squared) and the plane-plane flag.
struct TubeTolerances {
    tol1: f64,
    tol2: f64,
    tol3d: f64,
    is_plane_plane: bool,
}

impl TubeTolerances {
    /// The base tolerance `Precision::Approximation()` scaled to each parametric space.
    fn new(s1: &Adaptor3dSurfaceHandle, s2: &Adaptor3dSurfaceHandle) -> Result<Self, Failure> {
        let base: f64 = precision::approximation();
        let res_s1: f64 = std_min(s1.u_resolution(base)?, s1.v_resolution(base)?);
        let res_s2: f64 = std_min(s2.u_resolution(base)?, s2.v_resolution(base)?);
        Ok(Self {
            tol1: res_s1 * res_s1,
            tol2: res_s2 * res_s2,
            tol3d: base * base,
            is_plane_plane: is_surf_plane_like(s1)? && is_surf_plane_like(s2)?,
        })
    }

    /// Whether the steps before and after the point `pnt3d` are close in 3d (always, for
    /// two plane-like surfaces).
    fn is_even_3d(&self, pipe: &Pipe, pnt3d: Point3) -> bool {
        const MAX_SQR_RATIO: f64 = 15.0 * 15.0;
        let curr_step: f64 = pipe.base_3d_pnt.square_distance(&pnt3d);
        let mut sqr_ratio: f64 = 0.0;
        if !self.is_plane_plane {
            sqr_ratio = if curr_step > precision::square_computational() {
                pipe.prev_step / curr_step
            } else {
                MAX_SQR_RATIO
            };
            if sqr_ratio < 1.0 {
                sqr_ratio = 1.0 / sqr_ratio;
            }
        }
        sqr_ratio < MAX_SQR_RATIO
    }
}

/// Whether the point `pnt` (after a stored point `i - 1`) lies in the three tubes.
fn is_in_tubes(tols: &TubeTolerances, pipe: &Pipe, pnt: &IntSurfPntOn2S) -> bool {
    let (u1, v1) = pnt.parameters_on_s1();
    let (u2, v2) = pnt.parameters_on_s2();
    is_inside_in_2d(
        pipe.base_pnt1,
        pipe.base_vec1,
        Point2::new(u1, v1),
        tols.tol1,
    ) && is_inside_in_2d(
        pipe.base_pnt2,
        pipe.base_vec2,
        Point2::new(u2, v2),
        tols.tol2,
    ) && is_inside_in_3d(pipe.base_3d_pnt, pipe.base_3d_vec, pnt.value(), tols.tol3d)
}

/// Runs the tube test on the points `3..=NbPnts` and returns the number of kept points.
fn mark_by_tube(
    wline: &IntPatchWLine,
    tols: &TubeTolerances,
    hash: &mut PointsHash,
) -> Result<i32, Failure> {
    let mut pipe = Pipe::new(&wline.point(1)?, &wline.point(2)?);
    let mut nb_pnt: i32 = 0;
    for i in 3..=wline.nb_pnts()? {
        let pnt: IntSurfPntOn2S = wline.point(i)?;
        pipe.uv1[2] = pnt.parameters_on_s1();
        pipe.uv2[2] = pnt.parameters_on_s2();
        let mut is_delete_state = false;
        if hash.get(i - 1)? != VERTEX && is_in_tubes(tols, &pipe, &pnt) {
            // Handle possible uneven parametrization on one of 2d subspaces: delete the
            // point only when the expected lengths are close to each other.
            match pipe.even_steps() {
                None => continue,
                Some(true) if tols.is_even_3d(&pipe, pnt.value()) => {
                    is_delete_state = true;
                    hash.set(i - 1, DELETE)?;
                    // Change middle point.
                    pipe.uv1[1] = pipe.uv1[2];
                    pipe.uv2[1] = pipe.uv2[2];
                }
                Some(_) => {}
            }
        }
        if !is_delete_state {
            pipe.advance(wline.point(i - 1)?.value(), pnt.value());
            nb_pnt += 1;
        }
    }
    Ok(nb_pnt)
}

/// The workarounds of the small lines and of the bad distributions of points.
fn fix_small_counts(nb_pnts: i32, nb_pnt: i32, hash: &mut PointsHash) -> Result<(), Failure> {
    // Workaround to handle case of small amount of points after purge.
    // Test "boolean boptuc_complex B5" and similar.
    // This is possible since there are at least two points.
    if hash.get(1)? == VERTEX && hash.get(2)? == VERTEX && nb_pnt <= 3 {
        // Delete first.
        hash.set(1, DELETE)?;
    }
    if hash.get(nb_pnts - 1)? == VERTEX && hash.get(nb_pnts)? == VERTEX && nb_pnt <= 3 {
        // Delete last.
        hash.set(nb_pnts, DELETE)?;
    }
    // Purge when too small amount of points left.
    if nb_pnt <= 2 {
        for i in 1..=hash.upper() {
            if hash.get(i)? != VERTEX {
                hash.set(i, DELETE)?;
            }
        }
    }
    // Handle possible bad distribution of points, which will be converted into one single
    // bezier curve (less than 30 points). Make distribution more even: max step will be
    // nearly to 0.1 of param distance.
    if nb_pnt + 2 > MIN_NB_BAD_DISTR && nb_pnt + 2 < NB_SINGLE_BEZIER {
        for idx in 1..=8 {
            let hash_idx: i32 = idx * nb_pnts / 9;
            // Vertex must be stored as VERTEX (HASH = -1).
            if hash.get(hash_idx)? != VERTEX {
                hash.set(hash_idx, STORE)?;
            }
        }
    }
    Ok(())
}

/// `DeleteByTube`: the points that lie in the tubes of their neighbours are removed
/// (step III); a line of at most 2 points is returned as is.
pub(super) fn delete_by_tube(
    wline: &Handle<IntPatchWLine>,
    s1: &Adaptor3dSurfaceHandle,
    s2: &Adaptor3dSurfaceHandle,
) -> Result<Handle<IntPatchWLine>, Failure> {
    // Workaround to handle case of small amount points after purge.
    // Test "boolean boptuc_complex B5" and similar.
    let nb_pnts: i32 = wline.nb_pnts()?;
    if nb_pnts <= 2 {
        return Ok(wline.clone());
    }
    let mut hash: PointsHash = fill_points_hash(wline)?;
    let tols = TubeTolerances::new(s1, s2)?;
    let nb_pnt: i32 = mark_by_tube(wline, &tols, &mut hash)?;
    fix_small_counts(nb_pnts, nb_pnt, &mut hash)?;
    make_new_wline(wline, &mut hash, false)
}
