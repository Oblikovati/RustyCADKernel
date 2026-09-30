// Rust translation of IsParallel, AdjustToDomain, SeekPointOnBoundary, PutToBoundary and
// SeekAdditionalPoints of ModelingAlgorithms/TKGeomAlgo/IntWalk/IntWalk_PWalking.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The post-processing of a computed line: its ends put on the surface boundaries
//! when they are near them, and additional points inserted in a short line.
use super::IntWalkPWalking;
use super::minimize::domain_borders;
use crate::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, real};
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkmath::gp::{Point3, Xyz};

/// `IsParallel`: whether the line is parallel to the U and to the V boundaries of one
/// surface (its V, respectively U, parameter swings less than `toler` on at most 23
/// sampled points); a line of fewer than 3 points is parallel to both.
fn is_parallel(
    line: &IntSurfLineOn2S,
    check_surf1: bool,
    toler: f64,
) -> Result<(bool, bool), Failure> {
    let nb_line_pnts: i32 = line.nb_points();
    let mut nb_points: i32 = nb_line_pnts;
    if nb_points > 23 {
        nb_points = 23;
    } else if nb_points < 3 {
        return Ok((true, true));
    }
    let step: f64 = f64::from(line.nb_points()) / f64::from(nb_points);
    let mut n_point: f64 = 1.0;
    let (mut umin, mut umax, mut vmin, mut vmax) = (
        real::real_last(),
        real::real_first(),
        real::real_last(),
        real::real_first(),
    );
    for _ in 1..=nb_points {
        if n_point > f64::from(nb_line_pnts) {
            n_point = f64::from(nb_line_pnts);
        }
        let point = line.value(real::real_to_int(n_point))?;
        let (u, v) = point.parameters_on_surface(check_surf1);
        (umin, umax) = (
            if u < umin { u } else { umin },
            if u > umax { u } else { umax },
        );
        (vmin, vmax) = (
            if v < vmin { v } else { vmin },
            if v > vmax { v } else { vmax },
        );
        n_point += step;
    }
    Ok(((vmax - vmin) < toler, (umax - umin) < toler))
}

/// `AdjustToDomain`: the parameters beyond a bound by more than PConfusion go onto
/// it; returns whether one moved.
fn adjust_to_domain(params: &mut [f64], low: &[f64], upp: &[f64]) -> bool {
    let mut moved: bool = false;
    for i in 0..params.len() {
        if (params[i] - low[i]) < -precision::p_confusion() {
            params[i] = low[i];
            moved = true;
        }
        if (params[i] - upp[i]) > precision::p_confusion() {
            params[i] = upp[i];
            moved = true;
        }
    }
    moved
}

/// The end parameters moved onto a near boundary (within `tol` but beyond the
/// confusion) in the directions the line is not parallel to; None when none moved.
fn boundary_target(
    (u1, v1, u2, v2): (f64, f64, f64, f64),
    (low, upp): (&[f64; 4], &[f64; 4]),
    parallel: [bool; 4],
    tol: f64,
) -> Option<[f64; 4]> {
    let tol_min: f64 = precision::confusion();
    let mut params: [f64; 4] = [u1, v1, u2, v2];
    let mut need: bool = false;
    // U1 unless V1-parallel, U2 unless V2-parallel, V1 unless U1-parallel, V2 unless U2-parallel.
    for (k, skip) in [
        (0, parallel[1]),
        (2, parallel[3]),
        (1, parallel[0]),
        (3, parallel[2]),
    ] {
        if skip {
            continue;
        }
        let delta: f64 = params[k] - low[k];
        if tol_min < delta && delta < tol {
            (params[k], need) = (low[k], true);
            continue;
        }
        let delta: f64 = upp[k] - params[k];
        if tol_min < delta && delta < tol {
            (params[k], need) = (upp[k], true);
        }
    }
    need.then_some(params)
}

impl IntWalkPWalking {
    /// `SeekPointOnBoundary(theASurf1, theASurf2, theU1, theV1, theU2, theV2,
    /// isTheFirst)`: the intersection point near the given boundary parameters (by
    /// gradient and Newton minimizations, or the singular point), inserted before the
    /// first (or after the last) point of the line; the points that would make the
    /// line turn back are removed first. Returns whether the point was inserted.
    pub fn seek_point_on_boundary(
        &mut self,
        surf1: &Adaptor3dSurfaceHandle,
        surf2: &Adaptor3dSurfaceHandle,
        uv: [f64; 4],
        is_the_first: bool,
    ) -> Result<bool, Failure> {
        let (low, upp) = domain_borders(surf1, surf2)?;
        let res1: f64 = std_max(
            precision::p_confusion() / surf1.u_resolution(1.0)?,
            precision::p_confusion() / surf1.v_resolution(1.0)?,
        );
        let res2: f64 = std_max(
            precision::p_confusion() / surf2.u_resolution(1.0)?,
            precision::p_confusion() / surf2.v_resolution(1.0)?,
        );
        let tol: f64 = std_max(precision::confusion(), std_max(res1, res2));
        let mut pnt: [f64; 4] = uv;
        let mut singular: [f64; 4] = uv;
        let status: bool = self.minimize_on_boundary(surf1, surf2, &mut pnt, (&low, &upp))?;
        let singular_status: bool =
            self.handle_single_singular_point(surf1, surf2, tol, &mut singular)?;
        if singular_status {
            pnt = singular;
        }
        if !status && !singular_status {
            return Ok(false);
        }
        let p1: Point3 = surf1.value(pnt[0], pnt[1])?;
        let p2: Point3 = surf2.value(pnt[2], pnt[3])?;
        let pint = Point3::from((p1.xyz() + p2.xyz()) * 0.5);
        if pint.square_distance(&p1) > tol * tol {
            return Ok(false);
        }
        let mut ip = IntSurfPntOn2S::default();
        ip.set_value_all(pint, pnt[0], pnt[1], pnt[2], pnt[3]);
        if is_the_first {
            return self.insert_first(&ip, pint);
        }
        self.insert_last(&ip, pint)
    }
    /// The alternate minimizations of `SeekPointOnBoundary` (at most 20 rounds): each
    /// one stops the search when it succeeds without leaving the domain.
    fn minimize_on_boundary(
        &self,
        surf1: &Adaptor3dSurfaceHandle,
        surf2: &Adaptor3dSurfaceHandle,
        pnt: &mut [f64; 4],
        (low, upp): (&[f64; 4], &[f64; 4]),
    ) -> Result<bool, Failure> {
        let mut nb_iter: i32 = 20;
        loop {
            nb_iter -= 1;
            let mut status: bool = self.distance_minimize_by_gradient(surf1, surf2, pnt, None)?;
            if status && !adjust_to_domain(pnt, low, upp) {
                return Ok(true);
            }
            let (head, tail) = pnt.split_at_mut(2);
            status = self.distance_minimize_by_extrema(
                surf1,
                surf2.value(tail[0], tail[1])?,
                split2(head),
                None,
            )?;
            if status && !adjust_to_domain(head, &low[..2], &upp[..2]) {
                return Ok(true);
            }
            status = self.distance_minimize_by_extrema(
                surf2,
                surf1.value(head[0], head[1])?,
                split2(tail),
                None,
            )?;
            if status && !adjust_to_domain(tail, &low[2..], &upp[2..]) {
                return Ok(true);
            }
            if status || nb_iter <= 0 {
                return Ok(status);
            }
        }
    }
    /// The insertion before the first point: the first points that make the line
    /// turn back at the new point are removed, and a first point confused with it.
    fn insert_first(&mut self, ip: &IntSurfPntOn2S, pint: Point3) -> Result<bool, Failure> {
        loop {
            let line = self.line_ref()?;
            if line.nb_points() <= 1 {
                break;
            }
            let Some((p1, p2)) = distinct_pair(line, pint, true)? else {
                return Ok(false);
            };
            let (dir01, dir12): (Xyz, Xyz) = (p1.xyz() - pint.xyz(), p2.xyz() - p1.xyz());
            if dir01.dot(dir12) > 0.0 {
                break;
            }
            self.remove_a_point(1)?;
        }
        if self.line_ref()?.value(1)?.value().square_distance(&pint)
            <= precision::square_confusion()
        {
            self.remove_a_point(1)?;
        }
        self.line_ref()?.insert_before(1, ip)?;
        Ok(true)
    }
    /// The insertion after the last point, symmetric of [`Self::insert_first`].
    fn insert_last(&mut self, ip: &IntSurfPntOn2S, pint: Point3) -> Result<bool, Failure> {
        loop {
            let line = self.line_ref()?;
            let nb_pnts: i32 = line.nb_points();
            if nb_pnts <= 1 {
                break;
            }
            let Some((curr, prev)) = distinct_pair(line, pint, false)? else {
                return Ok(false);
            };
            let (dir_pc, dir_cn): (Xyz, Xyz) = (curr.xyz() - prev.xyz(), pint.xyz() - curr.xyz());
            if dir_pc.dot(dir_cn) > 0.0 {
                break;
            }
            self.remove_a_point(nb_pnts)?;
        }
        let nb_pnts: i32 = self.line_ref()?.nb_points();
        if self
            .line_ref()?
            .value(nb_pnts)?
            .value()
            .square_distance(&pint)
            <= precision::square_confusion()
        {
            self.remove_a_point(nb_pnts)?;
        }
        self.line_ref()?.add(ip);
        Ok(true)
    }
    /// `PutToBoundary(theASurf1, theASurf2)`: the first and the last point of the line
    /// are moved onto a near boundary (within a thousandth of the smallest range,
    /// at most 1e-3) in the directions the line is not parallel to. Returns whether
    /// the last attempt inserted a point.
    pub fn put_to_boundary(
        &mut self,
        surf1: &Adaptor3dSurfaceHandle,
        surf2: &Adaptor3dSurfaceHandle,
    ) -> Result<bool, Failure> {
        let (low, upp) = domain_borders(surf1, surf2)?;
        let mut tol: f64 = 1.0;
        tol = std_min(tol, upp[0] - low[0]);
        tol = std_min(tol, upp[2] - low[2]);
        tol = std_min(tol, upp[1] - low[1]);
        tol = std_min(tol, upp[3] - low[3]) * 1.0e-3;
        if tol <= 2.0 * precision::confusion() {
            return Ok(false);
        }
        let line = self.line_ref()?;
        let (u1p, v1p) = is_parallel(line, true, tol)?;
        let (u2p, v2p) = is_parallel(line, false, tol)?;
        let parallel: [bool; 4] = [u1p, v1p, u2p, v2p];
        let mut has_been_added: bool = false;
        let first = line.value(1)?.parameters();
        if let Some(target) = boundary_target(first, (&low, &upp), parallel, tol) {
            has_been_added = self.seek_point_on_boundary(surf1, surf2, target, true)?;
        }
        let line = self.line_ref()?;
        let last = line.value(line.nb_points())?.parameters();
        if let Some(target) = boundary_target(last, (&low, &upp), parallel, tol) {
            has_been_added = self.seek_point_on_boundary(surf1, surf2, target, false)?;
        }
        Ok(has_been_added)
    }
    /// `SeekAdditionalPoints(theASurf1, theASurf2, theMinNbPoints)`: inserts the exact
    /// intersection points in the middle of the segments of a line of at most
    /// `min_nb_points` points until it has that many; returns whether one was inserted
    /// (true at once when the line is long enough).
    pub fn seek_additional_points(
        &mut self,
        surf1: &Adaptor3dSurfaceHandle,
        surf2: &Adaptor3dSurfaceHandle,
        min_nb_points: i32,
    ) -> Result<bool, Failure> {
        let mut nb_points: i32 = self.line_ref()?.nb_points();
        if nb_points > min_nb_points {
            return Ok(true);
        }
        let (low, upp) = domain_borders(surf1, surf2)?;
        let mut is_precise: bool = false;
        let mut nb_points_prev: i32 = 0;
        while nb_points < min_nb_points && nb_points != nb_points_prev {
            nb_points_prev = nb_points;
            let mut fp: i32 = 1;
            while fp < nb_points {
                let mut lp: i32 = fp + 1;
                let mut pnt: [f64; 4] = self.segment_middle(fp, lp, (&low, &upp))?;
                if self.minimize_middle(surf1, surf2, &mut pnt)? {
                    if let Some(ip) = exact_point(surf1, surf2, &pnt)? {
                        self.line_ref()?.insert_before(lp, &ip)?;
                        is_precise = true;
                        nb_points += 1;
                        if nb_points >= min_nb_points {
                            break;
                        }
                    } else {
                        lp -= 1;
                    }
                }
                fp = lp + 1;
            }
        }
        Ok(is_precise)
    }
    /// The middle of the parameters of points `fp` and `lp`, clamped to the domain.
    fn segment_middle(
        &self,
        fp: i32,
        lp: i32,
        (low, upp): (&[f64; 4], &[f64; 4]),
    ) -> Result<[f64; 4], Failure> {
        let line = self.line_ref()?;
        let (a1, b1, c1, d1) = line.value(fp)?.parameters();
        let (a2, b2, c2, d2) = line.value(lp)?.parameters();
        let (first, last) = ([a1, b1, c1, d1], [a2, b2, c2, d2]);
        let mut pnt: [f64; 4] = [0.0; 4];
        for k in 0..4 {
            pnt[k] = 0.5 * (first[k] + last[k]);
            if pnt[k] < low[k] {
                pnt[k] = low[k];
            }
            if pnt[k] > upp[k] {
                pnt[k] = upp[k];
            }
        }
        Ok(pnt)
    }
    /// The alternate minimizations of `SeekAdditionalPoints` (at most 5 rounds, the
    /// first success stops them).
    fn minimize_middle(
        &self,
        surf1: &Adaptor3dSurfaceHandle,
        surf2: &Adaptor3dSurfaceHandle,
        pnt: &mut [f64; 4],
    ) -> Result<bool, Failure> {
        let mut nb_iter: i32 = 5;
        loop {
            if self.distance_minimize_by_gradient(surf1, surf2, pnt, None)? {
                return Ok(true);
            }
            let (head, tail) = pnt.split_at_mut(2);
            if self.distance_minimize_by_extrema(
                surf1,
                surf2.value(tail[0], tail[1])?,
                split2(head),
                None,
            )? {
                return Ok(true);
            }
            if self.distance_minimize_by_extrema(
                surf2,
                surf1.value(head[0], head[1])?,
                split2(tail),
                None,
            )? {
                return Ok(true);
            }
            nb_iter -= 1;
            if nb_iter <= 0 {
                return Ok(false);
            }
        }
    }
}

/// The two parameters of one surface as the in-out pair of the Newton search.
fn split2(pair: &mut [f64]) -> (&mut f64, &mut f64) {
    let (u, v) = pair.split_at_mut(1);
    (&mut u[0], &mut v[0])
}

/// The first two distinct points from the start (or from the end) of the line that
/// are away from `pint`; None when there are none.
fn distinct_pair(
    line: &IntSurfLineOn2S,
    pint: Point3,
    from_start: bool,
) -> Result<Option<(Point3, Point3)>, Failure> {
    let nb_pnts: i32 = line.nb_points();
    let order: Vec<i32> = if from_start {
        (1..=nb_pnts).collect()
    } else {
        (1..=nb_pnts).rev().collect()
    };
    let mut index: usize = 0;
    let mut first: Point3 = Point3::default();
    while index < order.len() {
        first = line.value(order[index])?.value();
        if first.square_distance(&pint) > precision::square_confusion() {
            break;
        }
        index += 1;
    }
    index += 1;
    while index < order.len() {
        let second: Point3 = line.value(order[index])?.value();
        if first.square_distance(&second) > precision::square_confusion() {
            return Ok(Some((first, second)));
        }
        index += 1;
    }
    Ok(None)
}

/// The middle of the surface points at `pnt` when both are within 1e-14 (squared) of it.
fn exact_point(
    surf1: &Adaptor3dSurfaceHandle,
    surf2: &Adaptor3dSurfaceHandle,
    pnt: &[f64; 4],
) -> Result<Option<IntSurfPntOn2S>, Failure> {
    let p1: Point3 = surf1.value(pnt[0], pnt[1])?;
    let p2: Point3 = surf2.value(pnt[2], pnt[3])?;
    let pint = Point3::from((p1.xyz() + p2.xyz()) * 0.5);
    if !(pint.square_distance(&p1) < 1.0e-14 && pint.square_distance(&p2) < 1.0e-14) {
        return Ok(None);
    }
    let mut ip = IntSurfPntOn2S::default();
    ip.set_value_all(pint, pnt[0], pnt[1], pnt[2], pnt[3]);
    Ok(Some(ip))
}
