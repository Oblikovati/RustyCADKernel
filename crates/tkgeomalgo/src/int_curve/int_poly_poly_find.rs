// Copyright (c) 1992-1999 Matra Datavision
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
// Rust translation of IntCurve_IntPolyPolyGen.gxx (findIntersect, GetIntersection) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntCurve_IntPolyPolyGen::findIntersect` and `GetIntersection`: the roots and
//! tangent zones of one polygon interference of two curves.
use super::int_poly_poly_gen::NBITER_MAX_POLYGON;
use super::int_poly_poly_zone::{
    CurveOf, PointPair, PolyCurve, orient_zone, point_transitions, position_on, zone_end,
    zone_parameter_range,
};
use super::{ExactIntersectionPoint, IntPolyPolyGen, PCurveTool, Polygon2dGen};
use crate::int_imp_par_gen::{IntersectorTypes, ParTool};
use crate::int_res2d::{Domain, IntersectionPoint, IntersectionSegment, Transition};
use crate::intf::{InterferencePolygon2d, Polygon2d, TangentZone};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::gp::Point2;
use std::cmp::Ordering;

/// A curve with its search domain.
type CurveOnDomain<'s, T> = (&'s CurveOf<T>, &'s Domain);

/// One curve of findIntersect: the curve, its domain and its polygon.
pub(super) type PolygonSide<'s, T> = (
    &'s CurveOf<T>,
    &'s Domain,
    &'s Polygon2dGen<<T as IntersectorTypes>::Par>,
);

impl<T: IntersectorTypes> IntPolyPolyGen<T>
where
    T::Par: PCurveTool,
{
    // `findIntersect(C1, D1, C2, D2, TolConf, Tol, NbIter, DeltaU, DeltaV, thePoly1,
    // thePoly2, isFullPolygon)`: false as soon as a section point of a partial
    // polygon has no exact root.
    pub(super) fn find_intersect(
        &mut self,
        (c1, d1, the_poly1): PolygonSide<'_, T>,
        (c2, d2, the_poly2): PolygonSide<'_, T>,
        (tol_conf, tol): (f64, f64),
        deltas: (i32, f64, f64),
        is_full_polygon: bool,
    ) -> Result<bool, Failure> {
        let inter_pp = InterferencePolygon2d::between(
            the_poly1 as &dyn Polygon2d,
            the_poly2 as &dyn Polygon2d,
        )?;
        let mut eip: ExactIntersectionPoint<'_, T::Par> =
            ExactIntersectionPoint::new(c1, c2, tol_conf)?;
        // To prevent exception in nbp=NbPoints().
        self.base.done = true;
        for sp in 1..=inter_pp.nb_section_points() {
            let spnt = inter_pp.pnt_value(sp)?;
            let (_, seg1, param1) = spnt.info_first_address();
            let (_, seg2, param2) = spnt.info_second_address();
            let mut seat1 = (seg1, param1);
            let mut seat2 = (seg2, param2);
            eip.perform_polygons(the_poly1, the_poly2, &mut seat1, &mut seat2)?;
            let an_error_occurred: bool = eip.an_error_occurred();
            if eip.nb_roots() == 0 && !is_full_polygon {
                return Ok(false);
            }
            if an_error_occurred {
                continue;
            }
            let mut root: (f64, f64) = eip.roots();
            let polys = (the_poly1, the_poly2);
            self.section_root(
                (c1, c2),
                polys,
                (seat1.0, seat2.0),
                &eip,
                &mut root,
                tol_conf,
            )?;
        }
        for tz in 1..=inter_pp.nb_tangent_zones() {
            let zone: &TangentZone = inter_pp.zone_value(tz)?;
            let sides = ((c1, d1), (c2, d2));
            self.find_zone(sides, (the_poly1, the_poly2), zone, (tol_conf, tol), deltas)?;
        }
        Ok(true)
    }

    // One section point after its exact search: the subdivision search when the
    // solver found no root farther than TolConf, then the point unless a result
    // has the same parameters within ten times EpsX.
    fn section_root(
        &mut self,
        (c1, c2): (&CurveOf<T>, &CurveOf<T>),
        (the_poly1, the_poly2): (&Polygon2dGen<T::Par>, &Polygon2dGen<T::Par>),
        (seg_index1, seg_index2): (i32, i32),
        eip: &ExactIntersectionPoint<'_, T::Par>,
        (u, v): &mut (f64, f64),
        tol_conf: f64,
    ) -> Result<(), Failure> {
        let mut pair: PointPair = PointPair::default();
        T::Par::d1(c1, *u, &mut pair.p1, &mut pair.tan1)?;
        T::Par::d1(c2, *v, &mut pair.p2, &mut pair.tan2)?;
        let mut dist: f64 = pair.p1.distance(pair.p2);
        if eip.nb_roots() == 0 && dist > tol_conf {
            let a_trans = Transition::new();
            let mut a_p_int = IntersectionPoint::new(pair.p1, *u, *v, a_trans, a_trans, false);
            let a_t1f: f64 = the_poly1.approx_param_on_curve(seg_index1, 0.0)?;
            let a_t1l: f64 = the_poly1.approx_param_on_curve(seg_index1, 1.0)?;
            let a_t2f: f64 = the_poly2.approx_param_on_curve(seg_index2, 0.0)?;
            let a_t2l: f64 = the_poly2.approx_param_on_curve(seg_index2, 1.0)?;
            let mut search = SubdivisionSearch {
                tol_conf,
                max_count: 16,
                count: 0,
                p_int: &mut a_p_int,
                dist: &mut dist,
            };
            search.run::<T>((c1, a_t1f, a_t1l), (c2, a_t2f, a_t2l))?;
            *u = a_p_int.param_on_first();
            *v = a_p_int.param_on_second();
            T::Par::d1(c1, *u, &mut pair.p1, &mut pair.tan1)?;
            T::Par::d1(c2, *v, &mut pair.p2, &mut pair.tan2)?;
            dist = pair.p1.distance(pair.p2);
        }
        // It is checked if the point (u,v) does not exist already.
        let eps_x1: f64 = 10.0 * T::Par::eps_x(c1);
        let eps_x2: f64 = 10.0 * T::Par::eps_x(c2);
        for p in 1..=self.base.nb_points()? {
            let point = self.base.point(p)?;
            if (*u - point.param_on_first()).abs() <= eps_x1
                && (*v - point.param_on_second()).abs() <= eps_x2
            {
                dist = tol_conf + 1.0;
                break;
            }
        }
        if dist > tol_conf {
            return Ok(());
        }
        let pos1 = position_on(&self.domain_on_curve1, pair.p1)?;
        let pos2 = position_on(&self.domain_on_curve2, pair.p2)?;
        let (mut trans1, mut trans2) = (Transition::new(), Transition::new());
        point_transitions::<T>(
            (c1, *u),
            (c2, *v),
            (pos1, pos2),
            &mut pair,
            (&mut trans1, &mut trans2),
            tol_conf,
        )?;
        let ip = IntersectionPoint::new(pair.p1, *u, *v, trans1, trans2, false);
        self.base.insert(ip)
    }

    // One tangent zone: a finer pass on the zone's parameter ranges (tolerances
    // swapped, as natively) while a deflection exceeds TolConf and fewer than 10
    // iterations ran, else a point or a segment between the zone ends.
    fn find_zone(
        &mut self,
        ((c1, d1), (c2, d2)): (CurveOnDomain<'_, T>, CurveOnDomain<'_, T>),
        (the_poly1, the_poly2): (&Polygon2dGen<T::Par>, &Polygon2dGen<T::Par>),
        zone: &TangentZone,
        (tol_conf, tol): (f64, f64),
        (nb_iter, delta_u, delta_v): (i32, f64, f64),
    ) -> Result<(), Failure> {
        let range: [f64; 4] = zone_parameter_range(zone, the_poly1, the_poly2)?;
        let ends: [f64; 4] = orient_zone::<T>(c1, c2, range)?;
        let [inf1, sup1, inf2, sup2] = range;
        if ((the_poly1.deflection_over_estimation() > tol_conf)
            || (the_poly2.deflection_over_estimation() > tol_conf))
            && (nb_iter < NBITER_MAX_POLYGON)
        {
            let recurs_d1 = Domain::bounded(
                T::Par::value(c1, inf1)?,
                inf1,
                tol_conf,
                T::Par::value(c1, sup1)?,
                sup1,
                tol_conf,
            );
            let recurs_d2 = Domain::bounded(
                T::Par::value(c2, inf2)?,
                inf2,
                tol_conf,
                T::Par::value(c2, sup2)?,
                sup2,
                tol_conf,
            );
            // The native call passes Tol as TolConf and TolConf as Tol.
            return self.perform_iter(
                c1,
                &recurs_d1,
                c2,
                &recurs_d2,
                tol,
                tol_conf,
                nb_iter + 1,
                delta_u,
                delta_v,
            );
        }
        let dom1 = self.domain_on_curve1;
        let dom2 = self.domain_on_curve2;
        let side1 = PolyCurve::<T> {
            curve: c1,
            domain: d1,
            on_curve: &dom1,
        };
        let side2 = PolyCurve::<T> {
            curve: c2,
            domain: d2,
            on_curve: &dom2,
        };
        match zone_segment::<T>((side1, side2), range, ends, tol_conf)? {
            ZoneResult::Point(point) => self.base.insert(point),
            ZoneResult::Segment(segment) => {
                self.base.append_segment(segment);
                Ok(())
            }
        }
    }
}

/// What a tangent zone gives: one point when its ends are within EpsX on a curve
/// (the "bad segment"), else a segment.
pub(super) enum ZoneResult {
    Point(IntersectionPoint),
    Segment(IntersectionSegment),
}

/// The ends of a tangent zone and their segment (the transitions of both ends
/// written into the same two transitions, as natively).
pub(super) fn zone_segment<T: IntersectorTypes>(
    (side1, side2): (PolyCurve<'_, T>, PolyCurve<'_, T>),
    [inf1, sup1, inf2, sup2]: [f64; 4],
    [u_inf, u_sup, v_inf, v_sup]: [f64; 4],
    tol_conf: f64,
) -> Result<ZoneResult, Failure> {
    let wide_on_first: bool = (inf1 - sup1).abs() > (inf2 - sup2).abs();
    let (mut trans1, mut trans2) = (Transition::new(), Transition::new());
    let mut pair: PointPair = PointPair::default();
    let (pt_seg1, poly_u_inf, poly_v_inf) = zone_end::<T>(
        side1,
        side2,
        (u_inf, v_inf),
        wide_on_first,
        &mut pair,
        (&mut trans1, &mut trans2),
        tol_conf,
    )?;
    if ((poly_u_inf - u_sup).abs() <= T::Par::eps_x(side1.curve))
        || ((poly_v_inf - v_sup).abs() <= T::Par::eps_x(side2.curve))
    {
        return Ok(ZoneResult::Point(pt_seg1));
    }
    let (pt_seg2, _, _) = zone_end::<T>(
        side1,
        side2,
        (u_sup, v_sup),
        wide_on_first,
        &mut pair,
        (&mut trans1, &mut trans2),
        tol_conf,
    )?;
    // `(Tan1.Dot(Tan2) > 0.0) ? false : true`: a NaN product is opposite.
    let oppos: bool = !matches!(
        pair.tan1.dot(pair.tan2).partial_cmp(&0.0),
        Some(Ordering::Greater)
    );
    let segment = if inf1 > sup1 {
        IntersectionSegment::bounded(pt_seg2, pt_seg1, oppos, false)
    } else {
        IntersectionSegment::bounded(pt_seg1, pt_seg2, oppos, false)
    };
    Ok(ZoneResult::Segment(segment))
}

/// `GetIntersection(theC1, theT1f, theT1l, theC2, theT2f, theT2l, theTolConf,
/// theMaxCount, thePInt, theDist, theCount)`: the recursive halving of both
/// parameter ranges whose boxes (enlarged by TolConf) overlap, down to ranges
/// shorter than their parameter tolerance or than 2 TolConf, or past `max_count`
/// levels; the nearest pair of the ends and middles gives the point.
struct SubdivisionSearch<'p> {
    tol_conf: f64,
    max_count: i32,
    count: i32,
    p_int: &'p mut IntersectionPoint,
    dist: &'p mut f64,
}

impl SubdivisionSearch<'_> {
    fn run<T: IntersectorTypes>(
        &mut self,
        (c1, t1f, t1l): (&CurveOf<T>, f64, f64),
        (c2, t2f, t2l): (&CurveOf<T>, f64, f64),
    ) -> Result<(), Failure>
    where
        T::Par: PCurveTool,
    {
        self.count += 1;
        let a_tol2: f64 = self.tol_conf * self.tol_conf;
        let a_p_tol1: f64 = native_max(
            100.0 * epsilon(native_max(t1f.abs(), t1l.abs())),
            precision::p_confusion(),
        );
        let a_p_tol2: f64 = native_max(
            100.0 * epsilon(native_max(t2f.abs(), t2l.abs())),
            precision::p_confusion(),
        );
        let a_p1f: Point2 = <T::Par as PCurveTool>::d0(c1, t1f)?;
        let a_p1l: Point2 = <T::Par as PCurveTool>::d0(c1, t1l)?;
        let mut a_b1 = BoundingBox2::default();
        a_b1.add_point(a_p1f);
        a_b1.add_point(a_p1l);
        a_b1.enlarge(self.tol_conf);
        let a_p2f: Point2 = <T::Par as PCurveTool>::d0(c2, t2f)?;
        let a_p2l: Point2 = <T::Par as PCurveTool>::d0(c2, t2l)?;
        let mut a_b2 = BoundingBox2::default();
        a_b2.add_point(a_p2f);
        a_b2.add_point(a_p2l);
        a_b2.enlarge(self.tol_conf);
        if a_b1.is_out_box(&a_b2) {
            self.count -= 1;
            return Ok(());
        }
        let is_small1: bool =
            (t1l - t1f) <= a_p_tol1 || a_p1f.square_distance(a_p1l) / 4.0 <= a_tol2;
        let is_small2: bool =
            (t2l - t2f) <= a_p_tol2 || a_p2f.square_distance(a_p2l) / 4.0 <= a_tol2;
        if (is_small1 && is_small2) || (self.count > self.max_count) {
            // Seems to be intersection: simple treatment of segment intersection.
            self.nearest_samples([t1f, t1l, t2f, t2l], [a_p1f, a_p1l, a_p2f, a_p2l]);
            self.count -= 1;
            return Ok(());
        }
        let a_t1m: f64 = (t1l + t1f) / 2.0;
        let a_t2m: f64 = (t2l + t2f) / 2.0;
        if is_small1 {
            self.run::<T>((c1, t1f, t1l), (c2, t2f, a_t2m))?;
            self.run::<T>((c1, t1f, t1l), (c2, a_t2m, t2l))?;
        } else if is_small2 {
            self.run::<T>((c1, t1f, a_t1m), (c2, t2f, t2l))?;
            self.run::<T>((c1, a_t1m, t1l), (c2, t2f, t2l))?;
        } else {
            self.run::<T>((c1, t1f, a_t1m), (c2, t2f, a_t2m))?;
            self.run::<T>((c1, t1f, a_t1m), (c2, a_t2m, t2l))?;
            self.run::<T>((c1, a_t1m, t1l), (c2, t2f, a_t2m))?;
            self.run::<T>((c1, a_t1m, t1l), (c2, a_t2m, t2l))?;
        }
        Ok(())
    }

    // The nearest pair among the ends and middles of both chords, kept when nearer
    // than the current distance.
    fn nearest_samples(
        &mut self,
        [t1f, t1l, t2f, t2l]: [f64; 4],
        [p1f, p1l, p2f, p2l]: [Point2; 4],
    ) {
        let a_pnts1: [Point2; 3] = [p1f, half_sum(p1f, p1l), p1l];
        let a_pnts2: [Point2; 3] = [p2f, half_sum(p2f, p2l), p2l];
        let (mut imin, mut jmin) = (0usize, 0usize);
        let mut dmin: f64 = f64::MAX;
        for (i, a) in a_pnts1.iter().enumerate() {
            for (j, b) in a_pnts2.iter().enumerate() {
                let (dx, dy) = (a.x() - b.x(), a.y() - b.y());
                let d: f64 = dx * dx + dy * dy;
                if d < dmin {
                    dmin = d;
                    imin = i;
                    jmin = j;
                }
            }
        }
        // imin and jmin stay -1 natively when every distance is NaN (out-of-bounds
        // reads if the distance still exceeds sqrt(RealLast)); the port reads the
        // first sample then.
        dmin = dmin.sqrt();
        if !matches!((*self.dist).partial_cmp(&dmin), Some(Ordering::Greater)) {
            return;
        }
        *self.dist = dmin;
        let t1: f64 = [t1f, (t1f + t1l) / 2.0, t1l][imin];
        let t2: f64 = [t2f, (t2f + t2l) / 2.0, t2l][jmin];
        let a_pint: Point2 = half_sum(a_pnts1[imin], a_pnts2[jmin]);
        let (a_trans1, a_trans2) = (Transition::new(), Transition::new());
        self.p_int
            .set_values(a_pint, t1, t2, a_trans1, a_trans2, false);
    }
}

// `(a.XY() + b.XY()) / 2.`
fn half_sum(a: Point2, b: Point2) -> Point2 {
    Point2::new((a.x() + b.x()) / 2.0, (a.y() + b.y()) / 2.0)
}

// `std::max(a, b)`: `b` only when `a < b`.
fn native_max(a: f64, b: f64) -> f64 {
    if a < b { b } else { a }
}
