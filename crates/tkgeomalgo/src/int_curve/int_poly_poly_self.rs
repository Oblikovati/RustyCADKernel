// Rust translation of IntCurve_IntPolyPolyGen.gxx (the self intersection) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The self intersection pass of `IntCurve_IntPolyPolyGen`.
use super::int_poly_poly_find::{ZoneResult, zone_segment};
use super::int_poly_poly_zone::{
    CurveOf, PointPair, PolyCurve, orient_zone, point_transitions, position_on,
    zone_parameter_range,
};
use super::{ExactIntersectionPoint, IntPolyPolyGen, PCurveTool, Polygon2dGen};
use crate::int_imp_par_gen::{IntersectorTypes, ParTool};
use crate::int_res2d::{Domain, IntersectionPoint, Transition};
use crate::intf::{InterferencePolygon2d, Polygon2d};
use rustycad_tkernel::standard::Failure;

impl<T: IntersectorTypes> IntPolyPolyGen<T>
where
    T::Par: PCurveTool,
{
    /// `Perform(Curve1, Domain1, TolConf, Tol, NbIter, DeltaU, DeltaV)` (protected):
    /// the self intersection on a polygon of twice the samples (3/2 of that times
    /// the iteration after the first; none past iteration 3, or 2 beyond 100
    /// samples). Only section points of non-adjacent chords are refined, once per
    /// chord pair; a root at equal parameters is dropped. The deltas are unused.
    /// `inter.perform_self_iter(&adaptor, &domain, 1e-9, 1e-9, 0, 1.0, 1.0)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn perform_self_iter(
        &mut self,
        c1: &CurveOf<T>,
        d1: &Domain,
        tol_conf: f64,
        tol: f64,
        nb_iter: i32,
        _delta_u: f64,
        _delta_v: f64,
    ) -> Result<(), Failure> {
        self.base.done = false;
        let mut nbsamples: i32 =
            T::Par::nb_samples(c1, d1.first_parameter()?, d1.last_parameter()?)?;
        if nb_iter > 3 || (nb_iter > 2 && nbsamples > 100) {
            return Ok(());
        }
        // Two times more points than on a normal curve: self-intersecting curves
        // often give polygons rather far from the curve.
        nbsamples *= 2;
        if nb_iter > 0 {
            nbsamples = (3 * (nbsamples * nb_iter)) / 2;
        }
        let mut poly1: Polygon2dGen<T::Par> = Polygon2dGen::new(c1, nbsamples, d1, tol)?;
        if !poly1.auto_intersection_is_possible()? {
            self.base.done = true;
            return Ok(());
        }
        // Detection of tangency zones.
        if poly1.deflection_over_estimation() < tol_conf {
            poly1.set_deflection_over_estimation(tol_conf);
        }
        let inter_pp = InterferencePolygon2d::of_self(&poly1 as &dyn Polygon2d)?;
        let mut eip: ExactIntersectionPoint<'_, T::Par> =
            ExactIntersectionPoint::new(c1, c1, tol_conf)?;
        for sp in sorted_section_points(&inter_pp)? {
            let spnt = inter_pp.pnt_value(sp)?;
            let (_, seg1, param1) = spnt.info_first_address();
            let (_, seg2, param2) = spnt.info_second_address();
            if (seg1 - seg2).abs() <= 1 {
                continue;
            }
            let (mut seat1, mut seat2) = ((seg1, param1), (seg2, param2));
            eip.perform_polygons(&poly1, &poly1, &mut seat1, &mut seat2)?;
            if eip.nb_roots() >= 1 {
                self.self_root(c1, eip.roots(), tol_conf)?;
            }
        }
        let dom1 = self.domain_on_curve1;
        let dom2 = self.domain_on_curve2;
        let side1 = PolyCurve::<T> {
            curve: c1,
            domain: d1,
            on_curve: &dom1,
        };
        let side2 = PolyCurve::<T> {
            curve: c1,
            domain: d1,
            on_curve: &dom2,
        };
        for tz in 1..=inter_pp.nb_tangent_zones() {
            let range: [f64; 4] = zone_parameter_range(inter_pp.zone_value(tz)?, &poly1, &poly1)?;
            let ends: [f64; 4] = orient_zone::<T>(c1, c1, range)?;
            // Ends within EpsX are a "bad segment", which gives nothing here.
            if let ZoneResult::Segment(segment) =
                zone_segment::<T>((side1, side2), range, ends, tol_conf)?
            {
                self.base.append_segment(segment);
            }
        }
        self.base.done = true;
        Ok(())
    }

    // One exact root `(u, v)` of the self intersection: kept when the curve points
    // are within TolConf, the parameters are not within ten times EpsX of each other
    // and no result has the same parameters.
    fn self_root(
        &mut self,
        c1: &CurveOf<T>,
        (u, v): (f64, f64),
        tol_conf: f64,
    ) -> Result<(), Failure> {
        let mut pair: PointPair = PointPair::default();
        T::Par::d1(c1, u, &mut pair.p1, &mut pair.tan1)?;
        T::Par::d1(c1, v, &mut pair.p2, &mut pair.tan2)?;
        let mut dist: f64 = pair.p1.distance(pair.p2);
        let eps_x1: f64 = 10.0 * T::Par::eps_x(c1);
        if (u - v).abs() <= eps_x1 {
            // Not valid: the solver converged to the trivial solution U = V.
            dist = tol_conf + 1.0;
        }
        self.base.done = true;
        for p in 1..=self.base.nb_points()? {
            let point = self.base.point(p)?;
            if (u - point.param_on_first()).abs() <= eps_x1
                && (v - point.param_on_second()).abs() <= eps_x1
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
            (c1, u),
            (c1, v),
            (pos1, pos2),
            &mut pair,
            (&mut trans1, &mut trans2),
            tol_conf,
        )?;
        let ip = IntersectionPoint::new(pair.p1, u, v, trans1, trans2, false);
        self.base.insert(ip)
    }
}

// The section point numbers in the order of their (first, second) chord indices
// (the native bubble sort, which is stable), keeping the last of equal pairs.
fn sorted_section_points(inter_pp: &InterferencePolygon2d) -> Result<Vec<i32>, Failure> {
    let nbsp: i32 = inter_pp.nb_section_points();
    let mut segs: Vec<(i32, i32)> = Vec::with_capacity(nbsp as usize);
    for i in 1..=nbsp {
        let spnt = inter_pp.pnt_value(i)?;
        segs.push((spnt.info_first_address().1, spnt.info_second_address().1));
    }
    let key = |i: i32| segs[(i - 1) as usize];
    let mut tri_index: Vec<i32> = (1..=nbsp).collect();
    loop {
        let mut triok: bool = true;
        for tr in 0..tri_index.len().saturating_sub(1) {
            let (s1, s2) = key(tri_index[tr]);
            let (s_1, s_2) = key(tri_index[tr + 1]);
            if s1 > s_1 || (s1 == s_1 && s2 > s_2) {
                tri_index.swap(tr, tr + 1);
                triok = false;
            }
        }
        if triok {
            break;
        }
    }
    // "supression des doublons": a pair equal to the next one is dropped.
    let mut kept: Vec<i32> = Vec::with_capacity(tri_index.len());
    for (i, &index) in tri_index.iter().enumerate() {
        let duplicate: bool = tri_index
            .get(i + 1)
            .is_some_and(|&next| key(index) == key(next));
        if !duplicate {
            kept.push(index);
        }
    }
    Ok(kept)
}
