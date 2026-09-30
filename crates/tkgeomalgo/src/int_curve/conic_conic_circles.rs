// Rust translation of IntCurve_IntConicConic_1.cxx (CircleCircleGeometricIntersection,
// ProjectOnC2AndIntersectWithC2Domain) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The geometric part of the circle-circle intersection: the arcs of the first
//! circle within `Tol` of the second one, and their images on the second circle
//! clipped by its domain.
use super::PeriodicInterval;
use super::conic_conic_tool::PI_P_PI;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{self, Circle2, Vector2};
use std::f64::consts::PI;

/// The result of `CircleCircleGeometricIntersection`: `nbsol` is 0, 1, 2 or 3
/// (identical circles) with the arcs of the first circle.
pub(super) struct CircleArcs {
    pub(super) nbsol: i32,
    pub(super) int1: PeriodicInterval,
    pub(super) int2: PeriodicInterval,
}

// The half angle of the arc of C1 (radius `r` from its centre) cut by a circle of
// radius `r2` at distance `d` (in the frame x1 = O1O2).
fn half_arc(r: f64, d: f64, r2: f64) -> (f64, f64) {
    let rr: f64 = r * r;
    let dx: f64 = (rr + d * d - r2 * r2) / (d + d);
    let mut dy: f64 = rr - dx * dx;
    dy = if dy >= 0.0 { dy.sqrt() } else { 0.0 };
    (dy.atan2(dx), dy)
}

/// `CircleCircleGeometricIntersection(C1, C2, Tol, TolTang, C1_Res1, C1_Res2,
/// nbsol)`: the arcs of `c1` where `c2` enters the ring `R1 +/- Tol`; one tangent
/// point within `TolTang`; 3 when the circles coincide within `Tol`.
pub(super) fn circle_circle_geometric_intersection(
    c1: &Circle2,
    c2: &Circle2,
    tol: f64,
    tol_tang: f64,
) -> Result<CircleArcs, Failure> {
    let d_o1o2: f64 = c1.location().distance(c2.location());
    let (r1, r2) = (c1.radius(), c2.radius());
    let abs_r1m_r2: f64 = (r1 - r2).abs();
    let mut arcs = CircleArcs {
        nbsol: 0,
        int1: PeriodicInterval::default(),
        int2: PeriodicInterval::default(),
    };
    let (mut binf1, mut bsup1, mut binf2, mut bsup2) = (0.0, 0.0, 0.0, 0.0);
    if d_o1o2 > (r1 + r2 + tol) {
        if d_o1o2 > (r1 + r2 + tol_tang) {
            return Ok(arcs);
        }
        arcs.nbsol = 1;
    } else if d_o1o2 <= tol && abs_r1m_r2 <= tol {
        arcs.nbsol = 3;
        return Ok(arcs);
    } else if d_o1o2 > (r1 + r2) - tol {
        // C2 cuts only C1+ (radius R1 + Tol): one arc.
        let (d_alpha1, _) = half_arc(r1 + tol, d_o1o2, r2);
        (binf1, bsup1) = (-d_alpha1, d_alpha1);
        arcs.nbsol = 1;
    } else if d_o1o2 > abs_r1m_r2 - tol {
        // Two arcs between C2 and C1- and C1+; only the sign of dx changes when
        // dO1O2 < max(R1, R2).
        let (alpha_plus, _) = half_arc(r1 + tol, d_o1o2, r2);
        let (alpha_minus, dy) = half_arc(r1 - tol, d_o1o2, r2);
        [binf1, bsup1, binf2, bsup2] = [-alpha_plus, -alpha_minus, alpha_minus, alpha_plus];
        arcs.nbsol = merge_arcs(dy, [&mut binf1, &mut bsup1, &mut binf2, &mut bsup2]);
    } else if (d_o1o2 > abs_r1m_r2 - tol_tang) && (abs_r1m_r2 - tol_tang) > 0.0 {
        arcs.nbsol = 1;
    } else {
        return Ok(arcs);
    }
    place_arcs(c1, c2, &mut arcs, [binf1, bsup1, binf2, bsup2])?;
    Ok(arcs)
}

// The two arcs as one when their inner bounds coincide (`dy == 0`) or overlap;
// returns the number of arcs.
fn merge_arcs(dy: f64, [binf1, bsup1, binf2, bsup2]: [&mut f64; 4]) -> i32 {
    if dy == 0.0 {
        *bsup1 = *bsup2;
        return 1;
    }
    if *binf1 > *bsup1 {
        std::mem::swap(binf1, bsup1);
    }
    if *binf2 > *bsup2 {
        std::mem::swap(binf2, bsup2);
    }
    let overlap: bool =
        ((*binf1 <= *bsup2) && (*binf1 >= *binf2)) || ((*bsup1 <= *bsup2) && (*bsup1 >= *binf2));
    if !overlap {
        return 2;
    }
    if *binf1 > *binf2 {
        *binf1 = *binf2;
    }
    if *binf1 > *bsup2 {
        *binf1 = *bsup2;
    }
    if *bsup1 < *binf2 {
        *bsup1 = *binf2;
    }
    if *bsup1 < *bsup2 {
        *bsup1 = *bsup2;
    }
    1
}

// Moves the arcs from the frame (O1, O1O2) to the frame of C1; by construction no
// arc exceeds PI, so a longer one is complemented.
fn place_arcs(
    c1: &Circle2,
    c2: &Circle2,
    arcs: &mut CircleArcs,
    [binf1, bsup1, binf2, bsup2]: [f64; 4],
) -> Result<(), Failure> {
    let axe1: Vector2 = Vector2::from(c1.x_axis().direction());
    let axe_o1o2: Vector2 = Vector2::between(c1.location(), c2.location());
    let mut d_angle1: f64 = if axe_o1o2.magnitude() <= gp::resolution() {
        axe1.angle(Vector2::from(c2.x_axis().direction()))?
    } else {
        axe1.angle(axe_o1o2)?
    };
    if !c1.is_direct() {
        d_angle1 = -d_angle1;
    }
    arcs.int1.set_values(binf1 + d_angle1, bsup1 + d_angle1);
    if arcs.int1.length() > PI {
        arcs.int1.complement();
    }
    if arcs.nbsol != 2 {
        arcs.int2.set_null();
        return Ok(());
    }
    arcs.int2.set_values(binf2 + d_angle1, bsup2 + d_angle1);
    if arcs.int2.length() > PI {
        arcs.int2.complement();
    }
    Ok(())
}

/// The solution arcs on both circles. The native code stores them in two arrays of
/// 4 ("numerical problems may give 4") and would write past them; the port keeps
/// every arc.
#[derive(Default)]
pub(super) struct ArcSolutions {
    pub(super) on_c1: Vec<PeriodicInterval>,
    pub(super) on_c2: Vec<PeriodicInterval>,
}

// The parameter on `to` of the point of parameter `u` on `from`.
fn transfer(u: f64, from: &Circle2, to: &Circle2) -> Result<f64, Failure> {
    let point = elclib::circle_value_2d(u, from.axis(), from.radius());
    elclib::circle_parameter_2d(to.axis(), point)
}

// An interval on a circle from its transferred bounds: shorter than PI unless the
// circles are identical, where it is only ordered and moved below 2PI.
fn transferred_interval(inf: f64, sup: f64, ident_circles: bool) -> PeriodicInterval {
    let mut interval = PeriodicInterval::new(inf, sup);
    if !ident_circles {
        if interval.length() > PI {
            interval.complement();
        }
        return interval;
    }
    if interval.bsup <= interval.binf {
        interval.bsup += PI_P_PI;
    }
    if interval.binf >= PI_P_PI {
        interval.binf -= PI_P_PI;
        interval.bsup -= PI_P_PI;
    }
    interval
}

/// `ProjectOnC2AndIntersectWithC2Domain(Circle1, Circle2, C1DomainAndRes, DomainC2,
/// SolutionC1, SolutionC2, NbSolTotal, IdentCircles)`: the arc of `circle1` mapped
/// onto `circle2`, intersected with its domain (both periodic intersections), and
/// mapped back.
pub(super) fn project_on_c2_and_intersect_with_c2_domain(
    circles: (&Circle2, &Circle2),
    c1_domain_and_res: &mut PeriodicInterval,
    domain_c2: &mut PeriodicInterval,
    solutions: &mut ArcSolutions,
    ident_circles: bool,
) -> Result<(), Failure> {
    if c1_domain_and_res.is_null() {
        return Ok(());
    }
    let (circle1, circle2) = circles;
    let mut c2inf: f64 = transfer(c1_domain_and_res.binf, circle1, circle2)?;
    let c2sup: f64 = transfer(c1_domain_and_res.bsup, circle1, circle2)?;
    let mut c2_inter = PeriodicInterval::new(c2inf, c2sup);
    if !ident_circles {
        if c2_inter.length() > PI {
            c2_inter.complement();
        }
    } else {
        // The native code also moves C2sup by a period, but then overwrites Bsup
        // with the length of the arc on C1, so only C2inf matters.
        if c2inf >= PI_P_PI {
            c2inf -= PI_P_PI;
        }
        c2_inter.binf = c2inf;
        c2_inter.bsup = c2inf + c1_domain_and_res.bsup - c1_domain_and_res.binf;
    }
    for i in 0..2 {
        let mut part: PeriodicInterval = if i == 0 {
            domain_c2.first_intersection(&mut c2_inter)
        } else {
            domain_c2.second_intersection(&mut c2_inter)
        };
        if part.is_null() {
            continue;
        }
        let c1inf: f64 = transfer(part.binf, circle2, circle1)?;
        let c1sup: f64 = transfer(part.bsup, circle2, circle1)?;
        solutions
            .on_c1
            .push(transferred_interval(c1inf, c1sup, ident_circles));
        solutions.on_c2.push(part);
    }
    Ok(())
}
