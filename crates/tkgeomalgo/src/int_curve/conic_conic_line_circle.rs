// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of IntCurve_IntConicConic_1.cxx (LineCircleGeometricIntersection,
// LineEllipseGeometricIntersection and the line-circle and line-ellipse Perform) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntCurve_IntConicConic::Perform` for a line and a circle or an ellipse.
use super::conic_conic_line_conic::{
    ClosedConic, LineConicContext, collapse_line_conic_arcs, drop_split_arc, line_conic_solutions,
    max_tolerance,
};
use super::conic_conic_tool::PI_P_PI;
use super::{IntConicConic, PeriodicInterval};
use crate::int_res2d::Domain;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::extrema::ExtElC2d;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Ellipse2, Line2, Point2, Transform2, Vector2};
use std::f64::consts::PI;

/// `PIsur2`.
const PI_SUR_2: f64 = 0.5 * PI;

impl ClosedConic for Circle2 {
    fn point_at(&self, u: f64) -> Point2 {
        elclib::circle_value_2d(u, self.axis(), self.radius())
    }
    fn d1_at(&self, u: f64) -> (Point2, Vector2) {
        elclib::circle_d1_2d(u, self.axis(), self.radius())
    }
    fn d2_at(&self, u: f64) -> (Point2, Vector2, Vector2) {
        elclib::circle_d2_2d(u, self.axis(), self.radius())
    }
    fn length_radius(&self) -> f64 {
        self.radius()
    }
    const NATIVE_ELLIPSE_SLIPS: bool = false;
}

impl ClosedConic for Ellipse2 {
    fn point_at(&self, u: f64) -> Point2 {
        elclib::value(u, self)
    }
    fn d1_at(&self, u: f64) -> (Point2, Vector2) {
        elclib::ellipse_d1_2d(u, self.axis(), self.major_radius(), self.minor_radius())
    }
    fn d2_at(&self, u: f64) -> (Point2, Vector2, Vector2) {
        elclib::ellipse_d2_2d(u, self.axis(), self.major_radius(), self.minor_radius())
    }
    fn length_radius(&self) -> f64 {
        self.minor_radius()
    }
    const NATIVE_ELLIPSE_SLIPS: bool = true;
}

/// The arcs of the circle cut by the line: `(CInt1, CInt2, nbsol)`.
type ConicArcs = (PeriodicInterval, PeriodicInterval, i32);

/// `LineCircleGeometricIntersection(Line, Circle, Tol, TolTang, CInt1, CInt2, nbsol)`:
/// the arcs of the circle in the tube around the line (the native `Patch !!!` uses
/// `R` for both the inner and the outer radius); a tangent point within `TolTang`.
fn line_circle_geometric_intersection(
    line: &Line2,
    circle: &Circle2,
    tol: f64,
    tol_tang: f64,
) -> Result<ConicArcs, Failure> {
    let d_o1o2: f64 = line.distance(circle.location());
    let r: f64 = circle.radius();
    let rm_tol: f64 = r - tol;
    let (mut binf1, mut bsup1, mut binf2, mut bsup2) = (0.0, 0.0, 0.0, 0.0);
    let nbsol: i32;
    if d_o1o2 > (r + tol) {
        if d_o1o2 > (r + tol_tang) {
            return Ok((PeriodicInterval::default(), PeriodicInterval::default(), 0));
        }
        nbsol = 1;
    } else {
        let mut b2_sol: bool = false;
        if r > d_o1o2 + tol_tang {
            let a_tol2: f64 = tol * tol;
            let a_x2: f64 = 4. * (r * r - d_o1o2 * d_o1o2);
            if a_x2 > a_tol2 {
                b2_sol = !b2_sol;
            }
        }
        if d_o1o2 > rm_tol && !b2_sol {
            // dy = 0 (Patch !!!): atan2(0, dx) is 0 or PI.
            let d_alpha1: f64 = 0.0_f64.sqrt().atan2(d_o1o2);
            (binf1, bsup1) = (-d_alpha1, d_alpha1);
            nbsol = 1;
        } else {
            let mut dy: f64 = r * r - d_o1o2 * d_o1o2;
            dy = if dy >= 0.0 { dy.sqrt() } else { 0.0 };
            let d_alpha1: f64 = dy.atan2(d_o1o2);
            (binf1, bsup2, binf2, bsup1) = (-d_alpha1, d_alpha1, d_alpha1, -d_alpha1);
            if (d_alpha1 * r) < tol.max_native(tol_tang) {
                bsup1 = bsup2;
                nbsol = 1;
            } else {
                nbsol = 2;
            }
        }
    }
    place_line_circle_arcs(line, circle, nbsol, [binf1, bsup1, binf2, bsup2])
}

// `std::max(a, b)`.
trait NativeMax {
    fn max_native(self, other: f64) -> f64;
}
impl NativeMax for f64 {
    fn max_native(self, other: f64) -> f64 {
        if self < other { other } else { self }
    }
}

// The arcs moved from the frame of the line's normal to the circle's frame; an arc
// across 2PI is split in two (Modified by Sergey KHROMOV - Thu Oct 26 2000).
fn place_line_circle_arcs(
    line: &Line2,
    circle: &Circle2,
    mut nbsol: i32,
    [mut binf1, mut bsup1, mut binf2, mut bsup2]: [f64; 4],
) -> Result<ConicArcs, Failure> {
    // gp_Dir2d::Angle (not the gp_Vec2d one of the circle-circle case).
    let mut d_angle1: f64 = circle.x_axis().direction().angle(line.direction());
    let (a, b, c) = line.coefficients();
    let d: f64 = a * circle.location().x() + b * circle.location().y() + c;
    if d > 0.0 {
        d_angle1 += PI_SUR_2;
    } else {
        d_angle1 -= PI_SUR_2;
    }
    if d_angle1 < 0.0 {
        d_angle1 += PI_P_PI;
    } else if d_angle1 > PI_P_PI {
        d_angle1 -= PI_P_PI;
    }
    binf1 += d_angle1;
    bsup1 += d_angle1;
    if !circle.is_direct() {
        (binf1, bsup1) = (-bsup1, -binf1);
    }
    let (mut c_int1, mut c_int2) = (PeriodicInterval::default(), PeriodicInterval::default());
    set_arc(&mut c_int1, binf1, bsup1);
    if nbsol == 2 {
        binf2 += d_angle1;
        bsup2 += d_angle1;
        if !circle.is_direct() {
            (binf2, bsup2) = (-bsup2, -binf2);
        }
        set_arc(&mut c_int2, binf2, bsup2);
    } else if c_int1.bsup > PI_P_PI && c_int1.binf < PI_P_PI {
        nbsol = 2;
        let (binf2, bsup2) = (c_int1.binf, PI_P_PI);
        let bsup1: f64 = c_int1.bsup - PI_P_PI;
        set_arc(&mut c_int1, 0., bsup1);
        set_arc(&mut c_int2, binf2, bsup2);
    }
    Ok((c_int1, c_int2, nbsol))
}

// `SetValues(a, b)`, complemented when longer than PI.
fn set_arc(arc: &mut PeriodicInterval, a: f64, b: f64) {
    arc.set_values(a, b);
    if arc.length() > PI {
        arc.complement();
    }
}

/// `LineEllipseGeometricIntersection(Line, Ellipse, Tol, TolTang, EInt1, EInt2,
/// nbsol)`: the intersection points in the ellipse's frame (a vertical line when its
/// direction is within 2 epsilon of the Y axis); a missing line within `TolTang`
/// of the ellipse gives its nearest point (Extrema_ExtElC2d). The arcs are points.
fn line_ellipse_geometric_intersection(
    line: &Line2,
    ellipse: &Ellipse2,
    tol_tang: f64,
) -> Result<ConicArcs, Failure> {
    let mut a_tr: Transform2 = Transform2::default();
    a_tr.set_transformation_to(ellipse.axis().x_axis());
    let a_tellipse: Ellipse2 = ellipse.transformed(a_tr);
    let a_tline: Line2 = line.transformed(a_tr);
    let a_dy: f64 = a_tline.position().direction().y();
    let is_vert: bool = a_dy.abs() > 1. - 2. * epsilon(1.);
    let (a, b) = (a_tellipse.major_radius(), a_tellipse.minor_radius());
    let (a2, b2) = (a * a, b * b);
    let eps0: f64 = if b / a < 1.0e-5 { 1.0e-6 } else { 1.0e-12 };
    let (an_a, mut a_b, mut a_c) = a_tline.coefficients();
    if is_vert {
        a_c += a_b * a_tline.position().location().y();
        a_b = 0.;
    }
    let (x1, y1, x2, y2, nbsol): (f64, f64, f64, f64, i32);
    if a_b.abs() > eps0 {
        let m: f64 = -an_a / a_b;
        let m2: f64 = m * m;
        let c: f64 = -a_c / a_b;
        let c2: f64 = c * c;
        let mut d: f64 = a2 * m2 + b2 - c2;
        if d < 0. {
            return nearest_ellipse_point(&a_tline, &a_tellipse, tol_tang);
        }
        d = d.sqrt();
        let n: f64 = a2 * m2 + b2;
        let k: f64 = a * b * d / n;
        let l: f64 = -a2 * m * c / n;
        (x1, x2) = (l + k, l - k);
        (y1, y2) = (m * x1 + c, m * (l - k) + c);
        nbsol = 2;
    } else {
        x1 = -a_c / an_a;
        if x1.abs() > a + tol_tang {
            return Ok((PeriodicInterval::default(), PeriodicInterval::default(), 0));
        } else if x1.abs() >= a - epsilon(1. + a) {
            (y1, x2, y2, nbsol) = (0., 0., 0., 1);
        } else {
            y1 = b * (1. - x1 * x1 / a2).sqrt();
            (x2, y2, nbsol) = (x1, -y1, 2);
        }
    }
    ellipse_point_arcs(
        &a_tellipse,
        (Point2::new(x1, y1), Point2::new(x2, y2)),
        nbsol,
    )
}

// `Epsilon(x)`: the gap between |x| and the next double.
fn epsilon(x: f64) -> f64 {
    rustycad_tkernel::standard::real::epsilon(x)
}

// The degenerate arcs at the parameters of the intersection points, in order.
fn ellipse_point_arcs(
    ellipse: &Ellipse2,
    (p1, p2): (Point2, Point2),
    nbsol: i32,
) -> Result<ConicArcs, Failure> {
    let mut pe1: f64 = elclib::parameter(ellipse, p1)?;
    let (mut e_int1, mut e_int2) = (PeriodicInterval::default(), PeriodicInterval::default());
    if nbsol > 1 {
        let mut pe2: f64 = elclib::parameter(ellipse, p2)?;
        if pe2 < pe1 {
            (pe1, pe2) = (pe2, pe1);
        }
        e_int2.set_values(pe2, pe2);
    }
    e_int1.set_values(pe1, pe1);
    Ok((e_int1, e_int2, nbsol))
}

// The nearest point of the ellipse to a line that misses it, when within `TolTang`.
fn nearest_ellipse_point(
    line: &Line2,
    ellipse: &Ellipse2,
    tol_tang: f64,
) -> Result<ConicArcs, Failure> {
    let an_ext: ExtElC2d = ExtElC2d::line_ellipse(*line, *ellipse)?;
    let (mut imin, mut dmin) = (0, f64::MAX);
    for i in 1..=an_ext.nb_ext()? {
        if an_ext.square_distance(i)? < dmin {
            dmin = an_ext.square_distance(i)?;
            imin = i;
        }
    }
    let mut e_int1 = PeriodicInterval::default();
    if imin > 0 && dmin <= tol_tang * tol_tang {
        let (_, a_p2) = an_ext.points(imin)?;
        let pe1: f64 = a_p2.parameter();
        e_int1.set_values(pe1, pe1);
        return Ok((e_int1, PeriodicInterval::default(), 1));
    }
    Ok((e_int1, PeriodicInterval::default(), 0))
}

impl IntConicConic {
    /// `Perform(gp_Lin2d, DL, gp_Circ2d, DC, TolConf, Tol)`: the circle domain is
    /// not required to be closed (the native check is commented out).
    /// `inter.perform_line_circle(l, &dl, c, &dc, 1e-9, 1e-9)?`.
    pub fn perform_line_circle(
        &mut self,
        line: Line2,
        lig_domain: &Domain,
        circle: Circle2,
        circ_domain: &Domain,
        tol_conf: f64,
        tol: f64,
    ) -> Result<(), Failure> {
        self.base.reset_fields();
        let (mut c_int1, c_int2, mut nbsol) =
            line_circle_geometric_intersection(&line, &circle, tol_conf, tol)?;
        self.base.done = true;
        if nbsol == 0 {
            return Ok(());
        }
        drop_split_arc(&mut nbsol, &mut c_int1, &c_int2, circ_domain)?;
        self.perform_line_closed_conic(
            &line,
            lig_domain,
            &circle,
            circ_domain,
            (c_int1, c_int2, nbsol),
            (tol_conf, tol),
        )
    }

    /// `Perform(gp_Lin2d, DL, gp_Elips2d, DE, TolConf, Tol)`: intersection points
    /// only (arcs reduced to points), with the native slips of the second bound of an
    /// arc kept (see [`LineConicContext`]).
    /// `inter.perform_line_ellipse(l, &dl, e, &de, 1e-9, 1e-9)?`.
    pub fn perform_line_ellipse(
        &mut self,
        l: Line2,
        dl: &Domain,
        e: Ellipse2,
        de: &Domain,
        tol_conf: f64,
        tol: f64,
    ) -> Result<(), Failure> {
        self.base.reset_fields();
        let (mut e_int1, e_int2, mut nbsol) = line_ellipse_geometric_intersection(&l, &e, tol)?;
        self.base.done = true;
        if nbsol == 0 {
            return Ok(());
        }
        drop_split_arc(&mut nbsol, &mut e_int1, &e_int2, de)?;
        self.perform_line_closed_conic(&l, dl, &e, de, (e_int1, e_int2, nbsol), (tol_conf, tol))
    }

    // The part of both overloads after the geometric arcs.
    fn perform_line_closed_conic<C: ClosedConic>(
        &mut self,
        line: &Line2,
        line_domain: &Domain,
        conic: &C,
        conic_domain: &Domain,
        arcs: ConicArcs,
        (tol_conf, tol): (f64, f64),
    ) -> Result<(), Failure> {
        let mut solutions = line_conic_solutions(conic, line, arcs, (line_domain, conic_domain))?;
        let max_tol: f64 = max_tolerance(tol_conf, tol);
        collapse_line_conic_arcs(&mut solutions, conic.length_radius(), max_tol);
        if solutions.on_conic.is_empty() {
            return Ok(());
        }
        let (_, tan1) = conic.d1_at(solutions.on_conic[0].binf);
        let (_, tan2) = elclib::line_d1_2d(solutions.on_line[0].binf, line.position());
        let ctx = LineConicContext {
            conic,
            line,
            line_domain,
            conic_domain,
            is_opposite: tan1.dot(tan2) < 0.0,
            max_tol,
            tol,
            ellipse_quirks: C::NATIVE_ELLIPSE_SLIPS,
        };
        self.add_line_conic_solutions(&ctx, &mut solutions)
    }
}
