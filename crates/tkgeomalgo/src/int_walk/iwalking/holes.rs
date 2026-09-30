// Rust translation of FillPntsInHoles, IsValidEndPoint, RemoveTwoEndPoints and IsPointOnLine of
// ModelingAlgorithms/TKGeomAlgo/IntWalk/IntWalk_IWalking.gxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The holes between lonely line ends (a start point of a closed line between two
//! ends) and the tests of a point on the computed lines.
use super::{IWalkingTypes, IntWalkIWalking, TheIWFunction, TheIWLine};
use crate::int_surf::{IntSurfInteriorPoint, IntSurfPntOn2S};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, real};
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Xyz};
use rustycad_tkmath::math::{FunctionSetRoot, Vector};
use std::collections::BTreeSet;

/// What `FillPntsInHoles` does with the nearest end of an end.
enum HoleVerdict {
    /// The two ends are close: both are removed from the lonely ends.
    Close,
    /// A point was found in the hole between them.
    Filled,
    /// No valid point between them: the next nearest end is tried.
    Bad,
}

impl<T: IWalkingTypes> IntWalkIWalking<T> {
    /// The end of line `|irang|`: the last point when `irang` is positive, else the first.
    fn line_end(&self, irang: i32) -> Result<IntSurfPntOn2S, Failure> {
        let line = self.line_at(irang.abs())?;
        if irang > 0 {
            line.value(line.nb_points())
        } else {
            line.value(1)
        }
    }
    /// `FillPntsInHoles`: pairs each lonely end of `copy_seq_alone` with its nearest
    /// lonely end, and finds a point of the function between them (an interior point
    /// of `pnts_in_holes`, recorded with the two ends in `PointLineLine`); close ends
    /// and ends without partner are removed from the lonely ends.
    pub(super) fn fill_pnts_in_holes(
        &mut self,
        sp: &mut T::Function,
        copy_seq_alone: &mut [i32],
        pnts_in_holes: &mut Vec<IntSurfInteriorPoint>,
    ) -> Result<(), Failure> {
        self.point_line_line.clear();
        let mut seq_to_remove: Vec<i32> = Vec::new();
        let mut bad_solutions: BTreeSet<i32> = BTreeSet::new();
        let mut i: usize = 0;
        while i + 1 < copy_seq_alone.len() {
            let irang1: i32 = copy_seq_alone[i];
            if irang1 != 0
                && !self.fill_one_end(
                    sp,
                    (copy_seq_alone, i),
                    (&mut seq_to_remove, &mut bad_solutions),
                    pnts_in_holes,
                )?
            {
                continue;
            }
            i += 1;
        }
        for removed in seq_to_remove {
            if let Some(at) = self.seq_alone.iter().position(|&alone| alone == removed) {
                self.seq_alone.remove(at);
            }
        }
        Ok(())
    }
    /// One lonely end `copy[i]`: returns false when its nearest partner was bad and
    /// the end must be tried again (the native `i--`).
    fn fill_one_end(
        &mut self,
        sp: &mut T::Function,
        (copy, i): (&mut [i32], usize),
        (seq_to_remove, bad_solutions): (&mut Vec<i32>, &mut BTreeSet<i32>),
        pnts_in_holes: &mut Vec<IntSurfInteriorPoint>,
    ) -> Result<bool, Failure> {
        let irang1: i32 = copy[i];
        let p2d1: Point2 = self.line_end(irang1)?.value_on_surface(self.reversed);
        let (min_rang, min_index) = self.nearest_end(copy, i, p2d1, bad_solutions)?;
        if min_rang == 0 {
            seq_to_remove.push(irang1);
            bad_solutions.clear();
            return Ok(true);
        }
        if irang1.abs() == min_rang.abs() && self.line_at(irang1.abs())?.nb_points() == 2 {
            seq_to_remove.extend([irang1, min_rang]);
            (copy[i], copy[min_index]) = (0, 0);
            bad_solutions.clear();
            return Ok(true);
        }
        let verdict: HoleVerdict = self.hole_between(sp, (irang1, min_rang), pnts_in_holes)?;
        if let HoleVerdict::Bad = verdict {
            bad_solutions.insert(min_rang);
            return Ok(false);
        }
        (copy[i], copy[min_index]) = (0, 0);
        if let HoleVerdict::Close = verdict {
            seq_to_remove.extend([irang1, min_rang]);
        }
        bad_solutions.clear();
        Ok(true)
    }
    /// The lonely end after `copy[i]`, not bad, nearest to `p2d1` in the parameters
    /// (the first of equal distances): its value and index, `(0, 0)` when none.
    fn nearest_end(
        &self,
        copy: &[i32],
        i: usize,
        p2d1: Point2,
        bad_solutions: &BTreeSet<i32>,
    ) -> Result<(i32, usize), Failure> {
        let (mut min_sq_dist, mut min_rang, mut min_index) = (real::real_last(), 0, 0);
        for (j, &irang2) in copy.iter().enumerate().skip(i + 1) {
            if irang2 == 0 || bad_solutions.contains(&irang2) {
                continue;
            }
            let p2d2: Point2 = self.line_end(irang2)?.value_on_surface(self.reversed);
            let sq_dist: f64 = p2d1.square_distance(p2d2);
            if sq_dist < min_sq_dist {
                (min_sq_dist, min_rang, min_index) = (sq_dist, irang2, j);
            }
        }
        Ok((min_rang, min_index))
    }
    /// The hole between the ends `irang1` and `min_rang`: close ends, or the root of
    /// the function from their middle, valid when it lies between them and on no line.
    fn hole_between(
        &mut self,
        sp: &mut T::Function,
        (irang1, min_rang): (i32, i32),
        pnts_in_holes: &mut Vec<IntSurfInteriorPoint>,
    ) -> Result<HoleVerdict, Failure> {
        let (alone1, alone2) = (self.line_end(irang1)?, self.line_end(min_rang)?);
        let (p2d1, p2d2) = (
            alone1.value_on_surface(self.reversed),
            alone2.value_on_surface(self.reversed),
        );
        let min_sq_dist_3d: f64 = alone1.value().square_distance(&alone2.value());
        if min_sq_dist_3d <= self.epsilon
            || ((p2d1.x() - p2d2.x()).abs() <= self.tolerance[1]
                && (p2d1.y() - p2d2.y()).abs() <= self.tolerance[2])
        {
            return Ok(HoleVerdict::Close);
        }
        let Some(uv) = self.solve_middle(sp, p2d1, p2d2)? else {
            return Ok(HoleVerdict::Bad);
        };
        let pmid = Point2::new(uv[1], uv[2]);
        if !self.is_between_and_free(pmid, p2d1, p2d2)? {
            return Ok(HoleVerdict::Bad);
        }
        let direction2d = sp.direction2d()?;
        let direction2d: Vector2 = Vector2::new(direction2d.x(), direction2d.y());
        let point = IntSurfInteriorPoint::with_value(
            sp.point(),
            uv[1],
            uv[2],
            sp.direction3d()?,
            direction2d,
        );
        pnts_in_holes.push(point);
        self.point_line_line
            .insert(pnts_in_holes.len() as i32, vec![irang1, min_rang]);
        Ok(HoleVerdict::Filled)
    }
    /// The root of the function from the middle of the two ends, when the solver
    /// converges to a non-tangent point within the function's tolerance.
    fn solve_middle(
        &self,
        sp: &mut T::Function,
        p2d1: Point2,
        p2d2: Point2,
    ) -> Result<Option<Vector<'static>>, Failure> {
        let (born_inf, born_sup) = self.domain_bounds()?;
        let mut uvap: Vector<'static> = Vector::with_bounds(1, 2)?;
        (uvap[1], uvap[2]) = ((p2d1.x() + p2d2.x()) / 2., (p2d1.y() + p2d2.y()) / 2.);
        let mut rsnld = FunctionSetRoot::new(&*sp, &self.tolerance, 100)?;
        let solved = rsnld.perform_bounded(sp, &uvap, &born_inf, &born_sup, false);
        sp.take_failure()?;
        solved?;
        if !(rsnld.is_done() && sp.root().abs() <= sp.tolerance() && !sp.is_tangent()?) {
            return Ok(None);
        }
        let mut uv: Vector<'static> = Vector::with_bounds(1, 2)?;
        rsnld.copy_root(&mut uv)?;
        Ok(Some(uv))
    }
    /// Whether `pmid` lies strictly between the ends (by the scalar products with
    /// their chord) and on none of the computed lines.
    fn is_between_and_free(
        &self,
        pmid: Point2,
        p2d1: Point2,
        p2d2: Point2,
    ) -> Result<bool, Failure> {
        let p1p2: Vector2 = Vector2::between(p2d1, p2d2);
        let scal_prod1: f64 = p1p2.dot(Vector2::between(p2d1, pmid));
        let scal_prod2: f64 = p1p2.dot(Vector2::between(p2d2, pmid));
        if !(scal_prod1 > 0. && scal_prod2 < 0.0) {
            return Ok(false);
        }
        for iline in 1..=self.lines.len() as i32 {
            if self.is_point_on_line_2d(pmid, iline)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
    /// The domain `(Um, Vm)` and `(UM, VM)` as solver bounds.
    pub(super) fn domain_bounds(&self) -> Result<(Vector<'static>, Vector<'static>), Failure> {
        let mut born_inf: Vector<'static> = Vector::with_bounds(1, 2)?;
        let mut born_sup: Vector<'static> = Vector::with_bounds(1, 2)?;
        (born_inf[1], born_sup[1], born_inf[2], born_sup[2]) =
            (self.um, self.um_last, self.vm, self.vm_last);
        Ok((born_inf, born_sup))
    }
    /// `IsValidEndPoint(IndOfPoint, IndOfLine)`: true without hole points; otherwise
    /// whether line end `ind_of_line` is one of the two ends of hole point
    /// `ind_of_point` (it is then taken from the pair). Standard_NoSuchObject when the
    /// point is not a hole point.
    pub(super) fn is_valid_end_point(
        &mut self,
        ind_of_point: i32,
        ind_of_line: i32,
    ) -> Result<bool, Failure> {
        if self.point_line_line.is_empty() {
            return Ok(true);
        }
        let ends: &mut Vec<i32> = self.point_line_line.get_mut(&ind_of_point).ok_or_else(|| {
            errors::no_such_object(&format!(
                "IntWalk_IWalking::IsValidEndPoint: point {ind_of_point} is not a hole point"
            ))
        })?;
        let Some(at) = ends.iter().position(|&end| end == ind_of_line) else {
            return Ok(false);
        };
        ends.remove(at);
        Ok(true)
    }
    /// `RemoveTwoEndPoints(IndOfPoint)`: the first and last ends recorded with hole
    /// point `ind_of_point` are no longer lonely. Standard_NoSuchObject when both were
    /// taken from its pair (the native `First` of an empty list).
    pub(super) fn remove_two_end_points(&mut self, ind_of_point: i32) -> Result<(), Failure> {
        let Some(ends) = self.point_line_line.get(&ind_of_point) else {
            return Ok(());
        };
        let empty = || errors::no_such_object("IntWalk_IWalking::RemoveTwoEndPoints: no end left");
        let (line1, line2) = (
            *ends.first().ok_or_else(empty)?,
            *ends.last().ok_or_else(empty)?,
        );
        self.seq_alone
            .retain(|&alone| alone != line1 && alone != line2);
        Ok(())
    }
    /// `IsPointOnLine(theP2d, Irang)`: whether `p2d` is a point of line `|irang|` (within
    /// the resolutions) or lies between two of its consecutive points.
    pub(super) fn is_point_on_line_2d(&self, p2d: Point2, irang: i32) -> Result<bool, Failure> {
        let line = self.line_at(irang.abs())?;
        let nb_points: i32 = line.nb_points();
        for i in 1..=nb_points {
            let p2d1: Point2 = line.value(i)?.value_on_surface(self.reversed);
            if (p2d1.x() - p2d.x()).abs() <= self.tolerance[1]
                && (p2d1.y() - p2d.y()).abs() <= self.tolerance[2]
            {
                return Ok(true);
            }
            if i < nb_points {
                let p2d2: Point2 = line.value(i + 1)?.value_on_surface(self.reversed);
                if Vector2::between(p2d, p2d1).dot(Vector2::between(p2d, p2d2)) < 0.0 {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
    /// `IsPointOnLine(thePOn2S, InfBounds, SupBounds, theSolver, theFunc)`: projects the
    /// point on the nearest segment of each computed line, refines the projection
    /// with the solver, and accepts the point when it is nearer to the refined point
    /// than twice the refinement move.
    pub(super) fn is_point_on_line(
        &self,
        pon2s: &IntSurfPntOn2S,
        (inf_bounds, sup_bounds): (&Vector<'_>, &Vector<'_>),
        solver: &mut FunctionSetRoot,
        func: &mut T::Function,
    ) -> Result<bool, Failure> {
        let p3d: Point3 = pon2s.value();
        for handle in &self.lines {
            let lon2s = Self::line_of(handle)?.line();
            let Some(line) = lon2s.get() else { continue };
            if line.is_out_box(p3d) {
                continue;
            }
            let Some((umin, vmin, min_sq_dist)) = self.nearest_segment_point(line, p3d)? else {
                return Ok(true);
            };
            if min_sq_dist > precision::infinite() {
                continue;
            }
            if Self::refined_near(p3d, (umin, vmin), (inf_bounds, sup_bounds), solver, func)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
    /// The parameters of the projection of `p3d` on the nearest segment of `line`
    /// (`(0, 0)` when no segment projects); `None` when the projection is at a segment
    /// end (within Epsilon(1)): the point is then on the line.
    fn nearest_segment_point(
        &self,
        line: &crate::int_surf::IntSurfLineOn2S,
        p3d: Point3,
    ) -> Result<Option<(f64, f64, f64)>, Failure> {
        let eps: f64 = real::epsilon(1.);
        let (mut umin, mut vmin, mut min_sq_dist) = (0.0, 0.0, real::real_last());
        for index in 1..line.nb_points() {
            let (a, b) = (line.value(index)?, line.value(index + 1)?);
            let p1p: Xyz = p3d.xyz() - a.value().xyz();
            let p1p2: Xyz = b.value().xyz() - a.value().xyz();
            let sq12: f64 = p1p2.square_modulus();
            if sq12 < rustycad_tkmath::gp::resolution() {
                continue;
            }
            let dp: f64 = p1p.dot(p1p2);
            if dp < 0.0 || dp > sq12 {
                continue;
            }
            let sq_d: f64 = p1p.cross_square_magnitude(p1p2) / sq12;
            if sq_d < min_sq_dist {
                min_sq_dist = sq_d;
                let l1: f64 = dp / sq12;
                let l2: f64 = 1.0 - l1;
                if l1 < eps || l2 < eps {
                    return Ok(None);
                }
                let (u1, v1) = a.parameters_on_surface(self.reversed);
                let (u2, v2) = b.parameters_on_surface(self.reversed);
                (umin, vmin) = (l1 * u2 + l2 * u1, l1 * v2 + l2 * v1);
            }
        }
        Ok(Some((umin, vmin, min_sq_dist)))
    }
    /// The refinement of `IsPointOnLine`: the solver from `(umin, vmin)`; the point is
    /// on the line when it is nearer to the refined point than twice the move.
    fn refined_near(
        p3d: Point3,
        (umin, vmin): (f64, f64),
        (inf_bounds, sup_bounds): (&Vector<'_>, &Vector<'_>),
        solver: &mut FunctionSetRoot,
        func: &mut T::Function,
    ) -> Result<bool, Failure> {
        let mut prms: Vector<'static> = Vector::with_bounds(1, 2)?;
        (prms[1], prms[2]) = (umin, vmin);
        let solved = solver.perform_bounded(func, &prms, inf_bounds, sup_bounds, false);
        func.take_failure()?;
        solved?;
        if !solver.is_done() {
            return Ok(false);
        }
        solver.copy_root(&mut prms)?;
        let pa: Point3 = func.p_surface_value(umin, vmin)?;
        let pb: Point3 = func.p_surface_value(prms[1], prms[2])?;
        Ok(pb.square_distance(&p3d) < 4.0 * pa.square_distance(&pb))
    }
}
