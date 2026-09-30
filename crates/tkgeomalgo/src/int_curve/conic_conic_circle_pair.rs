// Rust translation of IntCurve_IntConicConic_1.cxx (Perform for two circles) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntCurve_IntConicConic::Perform(gp_Circ2d, gp_Circ2d)`.
use super::conic_conic_circles::{
    ArcSolutions, circle_circle_geometric_intersection, project_on_c2_and_intersect_with_c2_domain,
};
use super::conic_conic_tool::{PI_P_PI, determine_transition_lc, normalize_on_circle_domain};
use super::{IntConicConic, PeriodicInterval};
use crate::int_imp_par_gen::determine_position;
use crate::int_res2d::{Domain, IntersectionPoint, IntersectionSegment, Position, Transition};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Point2, Vector2};

/// A circle domain as a periodic interval with its first bound moved into
/// `[0, 2PI)` and its length kept below one period.
fn periodic_domain(domain: &Domain) -> Result<PeriodicInterval, Failure> {
    let mut interval: PeriodicInterval = PeriodicInterval::from_domain(domain)?;
    let mut deltat: f64 = interval.bsup - interval.binf;
    if deltat >= PI_P_PI {
        // make deltat not including the upper limit
        deltat = libm_nextafter_below(PI_P_PI);
    }
    while interval.binf >= PI_P_PI {
        interval.binf -= PI_P_PI;
    }
    while interval.binf < 0.0 {
        interval.binf += PI_P_PI;
    }
    interval.bsup = interval.binf + deltat;
    Ok(interval)
}

// `std::nextafter(x, 0.)` for a positive finite x.
fn libm_nextafter_below(x: f64) -> f64 {
    f64::from_bits(x.to_bits() - 1)
}

/// The two circles, their domains, and the input domain of the second one (the
/// domain of the reversed circle is rebuilt when the circles turn opposite ways).
struct CirclePair<'a> {
    c1: Circle2,
    c2: Circle2,
    domain1: &'a Domain,
    domain2: Domain,
    input_domain2: &'a Domain,
    indirect: bool,
}

impl IntConicConic {
    /// `Perform(gp_Circ2d, D1, gp_Circ2d, D2, TolConf, Tol)`: the arcs of the first
    /// circle in the ring `R1 +/- TolConf` of the second (a tangency within `Tol`),
    /// mapped onto both domains; arcs shorter than `2 Tol` on both circles become
    /// points. A second circle turning the other way is reversed first.
    /// `inter.perform_circles(c1, &d1, c2, &d2, 1e-9, 1e-9)?`.
    pub fn perform_circles(
        &mut self,
        circle1: Circle2,
        domain_circ1: &Domain,
        circle2_in: Circle2,
        domain_circ2_in: &Domain,
        tol_conf: f64,
        tol: f64,
    ) -> Result<(), Failure> {
        let mut pair = CirclePair {
            c1: circle1,
            c2: circle2_in,
            domain1: domain_circ1,
            domain2: *domain_circ2_in,
            input_domain2: domain_circ2_in,
            indirect: false,
        };
        if circle1.is_direct() != circle2_in.is_direct() {
            pair.indirect = true;
            pair.c2 = circle2_in.reversed();
            pair.domain2 = reversed_domain(domain_circ2_in)?;
        }
        self.base.reset_fields();
        let mut arcs = circle_circle_geometric_intersection(&pair.c1, &pair.c2, tol_conf, tol)?;
        self.base.done = true;
        if arcs.nbsol == 0 {
            return Ok(());
        }
        let mut c1_domain: PeriodicInterval = periodic_domain(pair.domain1)?;
        let mut c2_domain: PeriodicInterval = periodic_domain(&pair.domain2)?;
        let ident_circles: bool = arcs.nbsol > 2;
        if ident_circles {
            // The circles coincide within Tol: the intervals may be longer than PI.
            arcs.int1.set_values(0.0, PI_P_PI);
            arcs.int2.set_null();
        }
        let mut solutions = ArcSolutions::default();
        let circles = (&pair.c1, &pair.c2);
        let mut part = c1_domain.first_intersection(&mut arcs.int1);
        project_on_c2_and_intersect_with_c2_domain(
            circles,
            &mut part,
            &mut c2_domain,
            &mut solutions,
            ident_circles,
        )?;
        let mut part = c1_domain.second_intersection(&mut arcs.int1);
        project_on_c2_and_intersect_with_c2_domain(
            circles,
            &mut part,
            &mut c2_domain,
            &mut solutions,
            ident_circles,
        )?;
        if arcs.nbsol == 2 {
            let mut part = c1_domain.first_intersection(&mut arcs.int2);
            project_on_c2_and_intersect_with_c2_domain(
                circles,
                &mut part,
                &mut c2_domain,
                &mut solutions,
                ident_circles,
            )?;
            let mut part = c1_domain.second_intersection(&mut arcs.int2);
            project_on_c2_and_intersect_with_c2_domain(
                circles,
                &mut part,
                &mut c2_domain,
                &mut solutions,
                ident_circles,
            )?;
        }
        let tol2: f64 = if tol < 1e-10 { 1e-10 } else { tol + tol };
        collapse_short_arcs(&pair, &mut solutions, tol2);
        for i in 0..solutions.on_c1.len() {
            self.add_circle_solution(&pair, &mut solutions, i, (arcs.nbsol, tol, tol2))?;
        }
        Ok(())
    }
}

// The domain of the reversed second circle: `2PI - u` swaps the bounds.
fn reversed_domain(domain: &Domain) -> Result<Domain, Failure> {
    let mut reversed: Domain = *domain;
    reversed.set_values(
        domain.last_point()?,
        PI_P_PI - domain.last_parameter()?,
        domain.last_tolerance()?,
        domain.first_point()?,
        PI_P_PI - domain.first_parameter()?,
        domain.first_tolerance()?,
    );
    reversed.set_equivalent_parameters(0.0, PI_P_PI)?;
    Ok(reversed)
}

// Arcs shorter than `tol2` on both circles become their mid points.
fn collapse_short_arcs(pair: &CirclePair<'_>, solutions: &mut ArcSolutions, tol2: f64) {
    let (r1, r2) = (pair.c1.radius(), pair.c2.radius());
    for (on_c1, on_c2) in solutions.on_c1.iter_mut().zip(solutions.on_c2.iter_mut()) {
        if (r1 * on_c1.length()) <= tol2 && (r2 * on_c2.length()) <= tol2 {
            let t: f64 = (on_c1.binf + on_c1.bsup) * 0.5;
            (on_c1.binf, on_c1.bsup) = (t, t);
            let t: f64 = (on_c2.binf + on_c2.bsup) * 0.5;
            (on_c2.binf, on_c2.bsup) = (t, t);
        }
    }
}

/// The first bounds of one arc pair on both circles, normalized on their domains.
struct ArcStart {
    c1inf: f64,
    c2inf: f64,
    c2sup: f64,
}

// A parameter normalized on a circle domain, kept (or clamped to the bound, which
// marks the arc out of range) when the normalisation leaves the domain.
fn clamp_on_domain(
    raw: f64,
    normalized: f64,
    domain: &Domain,
    out: &mut bool,
) -> Result<f64, Failure> {
    let mut value: f64 = normalized;
    if value < domain.first_parameter()? {
        if raw < domain.first_parameter()? {
            value = domain.first_parameter()?;
            *out = true;
        } else {
            value = raw;
        }
    }
    if value > domain.last_parameter()? {
        if raw > domain.last_parameter()? {
            value = domain.last_parameter()?;
            *out = true;
        } else {
            value = raw;
        }
    }
    Ok(value)
}

impl IntConicConic {
    // One solution arc: its first bound (skipped when clamped to a domain bound
    // farther than `tol2` on the two circles), then a point or a segment.
    fn add_circle_solution(
        &mut self,
        pair: &CirclePair<'_>,
        solutions: &mut ArcSolutions,
        i: usize,
        (nbsol, tol, tol2): (i32, f64, f64),
    ) -> Result<(), Failure> {
        let (r1, r2) = (pair.c1.radius(), pair.c2.radius());
        let is_opposite: bool =
            pair.c1.location().square_distance(pair.c2.location()) > (r1 * r1 + r2 * r2);
        let on_c2: PeriodicInterval = solutions.on_c2[i];
        let (c2inf_raw, c2sup) = if is_opposite {
            (on_c2.bsup, on_c2.binf)
        } else {
            (on_c2.binf, on_c2.bsup)
        };
        let c1tinf: f64 = solutions.on_c1[i].binf;
        let mut out_of_range: bool = false;
        let c1inf_n: f64 = normalize_on_circle_domain(c1tinf, pair.domain1)?;
        let c2inf_n: f64 = normalize_on_circle_domain(c2inf_raw, &pair.domain2)?;
        let c1inf: f64 = clamp_on_domain(c1tinf, c1inf_n, pair.domain1, &mut out_of_range)?;
        let c2inf: f64 = clamp_on_domain(c2inf_raw, c2inf_n, &pair.domain2, &mut out_of_range)?;
        if out_of_range {
            let p1: Point2 = elclib::circle_d2_2d(c1inf, pair.c1.axis(), r1).0;
            let p2: Point2 = elclib::circle_d2_2d(c2inf, pair.c2.axis(), r2).0;
            if p1.square_distance(p2) > tol2 * tol2 {
                // there are not any solutions in given parametric range.
                return Ok(());
            }
        }
        let start = ArcStart {
            c1inf,
            c2inf,
            c2sup,
        };
        self.add_circle_arc(pair, solutions, i, start, (nbsol, is_opposite, tol))
    }

    // The point (or segment) of an arc, with the transitions of both circles.
    fn add_circle_arc(
        &mut self,
        pair: &CirclePair<'_>,
        solutions: &mut ArcSolutions,
        i: usize,
        start: ArcStart,
        (nbsol, is_opposite, tol): (i32, bool, f64),
    ) -> Result<(), Failure> {
        let (p1a, t1a, t2a) = self.circle_bound(pair, start.c1inf, start.c2inf, tol)?;
        let u2a: f64 = pair.reported_c2(start.c2inf);
        let new_point1 = IntersectionPoint::new(p1a, start.c1inf, u2a, t1a, t2a, false);
        if !(solutions.on_c1[i].length() > 0.0 || solutions.on_c2[i].length() > 0.0) {
            self.base.append_point(new_point1);
            return Ok(());
        }
        let mut c1sup: f64 = normalize_on_circle_domain(solutions.on_c1[i].bsup, pair.domain1)?;
        if c1sup < start.c1inf {
            c1sup += PI_P_PI;
        }
        let mut c2sup: f64 = normalize_on_circle_domain(start.c2sup, &pair.domain2)?;
        let (p1b, t1b, t2b) = self.circle_bound(pair, c1sup, c2sup, tol)?;
        // The native code shifts C2inf or C2sup by a period here (C2inf is not read
        // again; for reversed circles only when they are not identical).
        if !is_opposite && !(pair.indirect && nbsol == 3) && c2sup < start.c2inf {
            c2sup += PI_P_PI;
        }
        let new_point2 =
            IntersectionPoint::new(p1b, c1sup, pair.reported_c2(c2sup), t1b, t2b, false);
        let oppos: bool = if pair.indirect {
            !is_opposite
        } else {
            is_opposite
        };
        self.base.append_segment(IntersectionSegment::bounded(
            new_point1, new_point2, oppos, false,
        ));
        Ok(())
    }

    // The point, positions and transitions at the parameters `u1` and `u2` of the
    // circles (the tangent of a reversed second circle is reversed back, and its
    // position is taken on the input domain at `2PI - u2`).
    fn circle_bound(
        &self,
        pair: &CirclePair<'_>,
        u1: f64,
        u2: f64,
        tol: f64,
    ) -> Result<(Point2, Transition, Transition), Failure> {
        let (p1, mut tan1, norm1) = elclib::circle_d2_2d(u1, pair.c1.axis(), pair.c1.radius());
        let (p2, mut tan2, norm2): (Point2, Vector2, Vector2) =
            elclib::circle_d2_2d(u2, pair.c2.axis(), pair.c2.radius());
        if pair.indirect {
            tan2.reverse();
        }
        let pos1: Position = determine_position(pair.domain1, p1, u1)?;
        let pos2: Position = if pair.indirect {
            determine_position(pair.input_domain2, p2, PI_P_PI - u2)?
        } else {
            determine_position(&pair.domain2, p2, u2)?
        };
        let (mut t1, mut t2) = (Transition::default(), Transition::default());
        determine_transition_lc(
            (pos1, &mut tan1, norm1, &mut t1),
            (pos2, &mut tan2, norm2, &mut t2),
            tol,
        )?;
        Ok((p1, t1, t2))
    }
}

impl CirclePair<'_> {
    // The parameter reported on the second circle: `2PI - u` on a reversed one.
    fn reported_c2(&self, u2: f64) -> f64 {
        if self.indirect { PI_P_PI - u2 } else { u2 }
    }
}
