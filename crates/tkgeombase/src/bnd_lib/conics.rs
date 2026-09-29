// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of the circle and ellipse members of BndLib.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BndLib::Add` for circles and ellipses. The 3D arcs read the accumulated box back
//! (its finite part, gap removed) and widen it with the coordinate extrema inside the
//! arc; the 2D arcs use the `Compute` polygon.
use super::compute::{ArcFrame, compute};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkmath::bnd::{BoundingBox2, BoundingBox3};
use rustycad_tkmath::bvh::{std_max, std_min};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{self, Axis2, Circle2, Circle3, Ellipse2, Ellipse3, Point2, Point3, Xy};
use std::f64::consts::PI;

/// `((min value, its parameter), (max value, its parameter))` of one coordinate.
type Extremum = ((f64, f64), (f64, f64));

// A 3D circle or ellipse for the BndLib arc analysis.
struct Conic3 {
    major: f64,
    minor: f64,
    origin: [f64; 3],
    x_dir: [f64; 3],
    y_dir: [f64; 3],
    position: Axis2,
    is_circle: bool,
}

impl Conic3 {
    fn of_circle(circle: &Circle3) -> Self {
        let position: Axis2 = circle.position();
        Self::new(position, (circle.radius(), circle.radius()), true)
    }
    fn of_ellipse(ellipse: &Ellipse3) -> Self {
        Self::new(
            ellipse.position(),
            (ellipse.major_radius(), ellipse.minor_radius()),
            false,
        )
    }
    fn new(position: Axis2, (major, minor): (f64, f64), is_circle: bool) -> Self {
        Self {
            major,
            minor,
            origin: *position.location().coordinates(),
            x_dir: *position.x_direction().coordinates(),
            y_dir: *position.y_direction().coordinates(),
            position,
            is_circle,
        }
    }
    fn value(&self, u: f64) -> Point3 {
        if self.is_circle {
            elclib::circle_value(u, self.position, self.major)
        } else {
            elclib::ellipse_value(u, self.position, self.major, self.minor)
        }
    }
    // Extremal parameters and values (min first) of coordinate k: the circle tangent is
    // `Yk / Xk`, the ellipse one `(MinR Yk) / (MajR Xk)`.
    fn extremum(&self, k: usize) -> Extremum {
        let (xk, yk) = (self.x_dir[k], self.y_dir[k]);
        let mut t_min: f64 = PI / 2.0;
        if xk.abs() > gp::resolution() {
            let ratio: f64 = if self.is_circle {
                yk / xk
            } else {
                (self.minor * yk) / (self.major * xk)
            };
            t_min = elclib::in_period(ratio.atan(), 0.0, 2.0 * PI);
        }
        let t_max: f64 = if t_min <= PI { t_min + PI } else { t_min - PI };
        let value = |t: f64| self.major * t.cos() * xk + self.minor * t.sin() * yk + self.origin[k];
        let (v_min, v_max) = (value(t_min), value(t_max));
        if v_min > v_max {
            return ((v_max, t_max), (v_min, t_min));
        }
        ((v_min, t_min), (v_max, t_max))
    }
}

// `Add(gp_Circ/gp_Elips, U1, U2, Tol, B)`.
fn add_conic_arc(
    conic: &Conic3,
    u1: f64,
    u2: f64,
    tolerance: f64,
    bounds: &mut BoundingBox3,
) -> Result<(), Failure> {
    let period: f64 = 2.0 * PI - epsilon(2.0 * PI);
    let (t1, t2) = if u2 - u1 > period {
        (0.0, 2.0 * PI)
    } else {
        elclib::adjust_periodic(0.0, 2.0 * PI, epsilon(1.0), u1, u2)
    };
    let extrema: [Extremum; 3] = [conic.extremum(0), conic.extremum(1), conic.extremum(2)];
    if t2 - t1 >= period {
        let min: [f64; 3] = extrema.map(|((v, _), _)| v);
        let max: [f64; 3] = extrema.map(|(_, (v, _))| v);
        bounds.update_bounds(min, max);
    } else {
        widen_with_arc(conic, (t1, t2), &extrema, bounds)?;
    }
    bounds.enlarge(tolerance);
    Ok(())
}

// The arc branch: the two ends, then the accumulated finite part (gap removed) widened by
// each extremum whose parameter lies in the arc.
fn widen_with_arc(
    conic: &Conic3,
    (t1, t2): (f64, f64),
    extrema: &[Extremum; 3],
    bounds: &mut BoundingBox3,
) -> Result<(), Failure> {
    bounds.add_point(conic.value(t1));
    bounds.add_point(conic.value(t2));
    let [x_min, y_min, z_min, x_max, y_max, z_max] = bounds.finite_part().get()?;
    let gap: f64 = bounds.gap();
    let mut min: [f64; 3] = [x_min + gap, y_min + gap, z_min + gap];
    let mut max: [f64; 3] = [x_max - gap, y_max - gap, z_max - gap];
    let inside = |t: f64| {
        let tk: f64 = elclib::in_period(t, t1, t1 + 2.0 * PI);
        tk >= t1 && tk <= t2
    };
    for (k, &((v_min, t_min), (v_max, t_max))) in extrema.iter().enumerate() {
        if inside(t_min) {
            min[k] = std_min(v_min, min[k]);
        }
        if inside(t_max) {
            max[k] = std_max(v_max, max[k]);
        }
    }
    bounds.update_bounds(min, max);
    Ok(())
}

/// `BndLib::Add(gp_Circ, Tol, B)`: `Add(C, 0, 2Pi, Tol, B)`; `add_circ(&circle, 0.0, &mut bounds)?`.
pub fn add_circ(
    circle: &Circle3,
    tolerance: f64,
    bounds: &mut BoundingBox3,
) -> Result<(), Failure> {
    add_circ_arc(circle, 0.0, 2.0 * PI, tolerance, bounds)
}

/// `BndLib::Add(gp_Circ, U1, U2, Tol, B)`: a span beyond `2Pi - Epsilon(2Pi)` updates `B`
/// with the coordinate extremes; an arc adds its ends and widens the box `B` then holds
/// (the gap removed, as natively) by the extrema inside the arc;
/// `add_circ_arc(&circle, 0.0, 1.5, 0.0, &mut bounds)?`.
pub fn add_circ_arc(
    circle: &Circle3,
    u1: f64,
    u2: f64,
    tolerance: f64,
    bounds: &mut BoundingBox3,
) -> Result<(), Failure> {
    add_conic_arc(&Conic3::of_circle(circle), u1, u2, tolerance, bounds)
}

/// `BndLib::Add(gp_Elips, Tol, B)`; `add_elips(&ellipse, 0.0, &mut bounds)?`.
pub fn add_elips(
    ellipse: &Ellipse3,
    tolerance: f64,
    bounds: &mut BoundingBox3,
) -> Result<(), Failure> {
    add_elips_arc(ellipse, 0.0, 2.0 * PI, tolerance, bounds)
}

/// `BndLib::Add(gp_Elips, U1, U2, Tol, B)`: as [`add_circ_arc`] for an ellipse;
/// `add_elips_arc(&ellipse, 0.0, 1.5, 0.0, &mut bounds)?`.
pub fn add_elips_arc(
    ellipse: &Ellipse3,
    u1: f64,
    u2: f64,
    tolerance: f64,
    bounds: &mut BoundingBox3,
) -> Result<(), Failure> {
    add_conic_arc(&Conic3::of_ellipse(ellipse), u1, u2, tolerance, bounds)
}

// The four corners `O +/- Ra Xd +/- Rb Yd` of the 2D full boxes, in `signs` order.
fn add_corners(
    bounds: &mut BoundingBox2,
    origin: Xy,
    (x_part, y_part): (Xy, Xy),
    signs: [(f64, f64); 4],
) {
    for (sx, sy) in signs {
        let with_x: Xy = if sx > 0.0 {
            origin + x_part
        } else {
            origin - x_part
        };
        let point: Xy = if sy > 0.0 {
            with_x + y_part
        } else {
            with_x - y_part
        };
        bounds.add_point(Point2::from(point));
    }
}

/// `BndLib::Add(gp_Circ2d, Tol, B)`: the four corners `O -/+ R Xd -/+ R Yd`;
/// `add_circ2d(&circle, 0.0, &mut bounds)`.
pub fn add_circ2d(circle: &Circle2, tolerance: f64, bounds: &mut BoundingBox2) {
    let frame = circle.position();
    let r: f64 = circle.radius();
    let parts: (Xy, Xy) = (frame.x_direction().xy() * r, frame.y_direction().xy() * r);
    add_corners(
        bounds,
        frame.location().xy(),
        parts,
        [(-1.0, -1.0), (-1.0, 1.0), (1.0, -1.0), (1.0, 1.0)],
    );
    bounds.enlarge(tolerance);
}

/// `BndLib::Add(gp_Circ2d, P1, P2, Tol, B)`: the `Compute` polygon of the arc;
/// `add_circ2d_arc(&circle, 0.0, 1.5, 0.0, &mut bounds)`.
pub fn add_circ2d_arc(
    circle: &Circle2,
    p1: f64,
    p2: f64,
    tolerance: f64,
    bounds: &mut BoundingBox2,
) {
    let frame = circle.position();
    let arc = ArcFrame {
        ra: circle.radius(),
        rb: circle.radius(),
        x_dir: Point2::from(frame.x_direction().xy()),
        y_dir: Point2::from(frame.y_direction().xy()),
        origin: frame.location(),
    };
    compute(bounds, p1, p2, &arc);
    bounds.enlarge(tolerance);
}

/// `BndLib::Add(gp_Elips2d, Tol, B)`: the corners `O +/- Ra Xd +/- Rb Yd` in the native
/// order (+ +, - +, - -, + -); `add_elips2d(&ellipse, 0.0, &mut bounds)`.
pub fn add_elips2d(ellipse: &Ellipse2, tolerance: f64, bounds: &mut BoundingBox2) {
    let frame = ellipse.axis();
    let parts: (Xy, Xy) = (
        frame.x_direction().xy() * ellipse.major_radius(),
        frame.y_direction().xy() * ellipse.minor_radius(),
    );
    add_corners(
        bounds,
        frame.location().xy(),
        parts,
        [(1.0, 1.0), (-1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)],
    );
    bounds.enlarge(tolerance);
}

/// `BndLib::Add(gp_Elips2d, P1, P2, Tol, B)`: the `Compute` polygon of the arc;
/// `add_elips2d_arc(&ellipse, 0.0, 1.5, 0.0, &mut bounds)`.
pub fn add_elips2d_arc(
    ellipse: &Ellipse2,
    p1: f64,
    p2: f64,
    tolerance: f64,
    bounds: &mut BoundingBox2,
) {
    let frame = ellipse.axis();
    let arc = ArcFrame {
        ra: ellipse.major_radius(),
        rb: ellipse.minor_radius(),
        x_dir: Point2::from(frame.x_direction().xy()),
        y_dir: Point2::from(frame.y_direction().xy()),
        origin: frame.location(),
    };
    compute(bounds, p1, p2, &arc);
    bounds.enlarge(tolerance);
}
