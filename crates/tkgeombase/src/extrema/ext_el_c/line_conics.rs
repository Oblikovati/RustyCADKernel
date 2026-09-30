// Rust translation of the line-conic constructors of Extrema_ExtElC.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The line-conic extrema of Extrema_ExtElC. Each solves `P1P2.D = 0`,
//! `P1P2.T = 0` for the conic parameter in the conic's frame, then projects the
//! conic point on the line (`u1 = O1P2.D`).
use super::ExtElC;
use super::trig_roots::{SortedTrigRoots, refine_dir};
use crate::extrema::ExtElC2d;
use crate::extrema::ext_p_el_c::polynomial_roots;
use crate::extrema::pon::POnCurv;
use crate::int_ana2d::AnaIntersection;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elclib::{self, ElementaryCurve};
use rustycad_tkmath::gp::{
    self, Axis22d, Circle2, Circle3, Direction2, Direction3, Ellipse3, Hyperbola3, Line2, Line3,
    Parabola3, Point2, Point3, Vector3, Xyz,
};
use rustycad_tkmath::math::DirectPolynomialRoots;
use std::f64::consts::PI;

/// The dot products `(Dx, Dy)` of the line direction with the conic X and Y axes
/// (the coefficients read these, not the normalized `D`) and
/// `V = (O2O1.D) D - O2O1`, with `D` normalized by `SetCoord` and O2O1 in the frame.
struct ConicFrame {
    dx: f64,
    dy: f64,
    v: Xyz,
}

// The frame of a conic of centre `o2` and axes `(x2, y2, z2)` seen from the line.
fn conic_frame(line: &Line3, o2: Point3, axes: [Direction3; 3]) -> Result<ConicFrame, Failure> {
    let d: Direction3 = line.direction();
    let [dx, dy, dz] = axes.map(|axis| d.dot(axis));
    let mut direction: Direction3 = d;
    direction.set_coordinates(dx, dy, dz)?;
    let mut o2o1: Vector3 = Vector3::between(o2, line.location());
    let [x2, y2, z2] = axes.map(Vector3::from);
    o2o1.set_coordinates(o2o1.dot(x2), o2o1.dot(y2), o2o1.dot(z2));
    let v: Xyz = direction.xyz() * o2o1.dot(Vector3::from(direction)) - *o2o1.xyz();
    Ok(ConicFrame { dx, dy, v })
}

// Zeroes a coefficient within `eps` of 0 (`A >= -eps && A <= eps`).
fn clamp_small(value: f64, eps: f64) -> f64 {
    if value >= -eps && value <= eps {
        0.
    } else {
        value
    }
}

impl ExtElC {
    // Stores the extremum of the conic parameter `u2`: `P2 = C2(u2)`,
    // `u1 = O1P2.D1`, `P1 = C1(u1)`.
    fn push_conic_parameter<C>(&mut self, line: &Line3, conic: &C, u2: f64)
    where
        C: ElementaryCurve<Point = Point3>,
    {
        let p2: Point3 = conic.value(u2);
        let u1: f64 = Vector3::between(line.location(), p2).dot(Vector3::from(line.direction()));
        let p1: Point3 = elclib::value(u1, line);
        let sq_dist: f64 = p1.square_distance(&p2);
        self.result
            .push(sq_dist, POnCurv::new(u1, p1), POnCurv::new(u2, p2));
    }

    /// `Extrema_ExtElC(gp_Lin, gp_Circ, Tol)`: a line in (or parallel to) the circle
    /// plane is solved in 2D with Extrema_ExtElC2d and IntAna2d; otherwise the roots
    /// of the trigonometric equation of the circle parameter. An identity equation
    /// (the line on the axis) is parallel with the square radius. The tolerance is
    /// unused; `ExtElC::line_circle(line, circle, 1e-10)?`.
    pub fn line_circle(line: Line3, circle: Circle3, _tol: f64) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        if extrema.planar_line_circle_extrema(&line, &circle)? {
            return Ok(extrema);
        }
        let axes: [Direction3; 3] = conic_axes(&circle);
        let d: Direction3 = line.direction();
        let mut direction: Direction3 = d;
        let [dx, dy, dz] = axes.map(|axis| d.dot(axis));
        direction.set_coordinates(dx, dy, dz)?;
        refine_dir(&mut direction)?;
        let (dx, dy) = (direction.x(), direction.y());
        let o2o1: Vector3 = circle_offset(line.location(), circle.location(), axes)?;
        let v: Xyz = direction.xyz() * o2o1.dot(Vector3::from(direction)) - *o2o1.xyz();
        let radius: f64 = circle.radius();
        let a5: f64 = radius * dx * dy;
        let coefficients: [f64; 5] = [
            -2. * a5,
            0.5 * radius * (dx * dx - dy * dy),
            v.y(),
            -v.x(),
            a5,
        ]
        .map(|value| clamp_small(value, 1.0e-12));
        let roots = SortedTrigRoots::new(coefficients, 0., PI + PI)?;
        if !roots.is_done() {
            return Ok(extrema);
        }
        if roots.infinite_roots()? {
            extrema.result.set_parallel(radius * radius);
            return Ok(extrema);
        }
        for index in 1..=roots.nb_solutions()? {
            extrema.push_conic_parameter(&line, &circle, roots.value(index)?);
        }
        extrema.result.done = true;
        Ok(extrema)
    }
}

// The x, y and z axes of a conic frame.
fn conic_axes<C: ConicAxes>(conic: &C) -> [Direction3; 3] {
    conic.frame_axes()
}

/// The `XAxis()`, `YAxis()` and `Axis()` directions of a 3D conic.
trait ConicAxes {
    fn frame_axes(&self) -> [Direction3; 3];
}

macro_rules! conic_axes {
    ($($conic:ty),*) => {$(
        impl ConicAxes for $conic {
            fn frame_axes(&self) -> [Direction3; 3] {
                [
                    self.x_axis().direction(),
                    self.y_axis().direction(),
                    self.axis().direction(),
                ]
            }
        }
    )*};
}
conic_axes!(Circle3, Ellipse3, Hyperbola3, Parabola3);

// O2O1 in the circle frame: normalized and refined when longer than
// gp::Resolution(), then scaled back.
fn circle_offset(o1: Point3, o2: Point3, axes: [Direction3; 3]) -> Result<Vector3, Failure> {
    let mut o2o1: Vector3 = Vector3::between(o2, o1);
    let [x2, y2, z2] = axes.map(Vector3::from);
    let length: f64 = o2o1.magnitude();
    if length <= gp::resolution() {
        o2o1.set_coordinates(o2o1.dot(x2), o2o1.dot(y2), o2o1.dot(z2));
        return Ok(o2o1);
    }
    o2o1 *= 1. / length;
    let mut unit: Direction3 = Direction3::new(1., 0., 0.)?;
    unit.set_coordinates(o2o1.dot(x2), o2o1.dot(y2), o2o1.dot(z2))?;
    refine_dir(&mut unit)?;
    o2o1.set_xyz(unit.xyz() * length);
    Ok(o2o1)
}

impl ExtElC {
    /// `PlanarLineCircleExtrema` (protected): when the line is normal to the circle
    /// axis within Precision::Angular(), the extrema and intersections of their
    /// projections in the circle plane (Extrema_ExtElC2d and IntAna2d); false
    /// otherwise.
    fn planar_line_circle_extrema(
        &mut self,
        line: &Line3,
        circle: &Circle3,
    ) -> Result<bool, Failure> {
        let (dir_c, dir_l) = (circle.axis().direction(), line.direction());
        if dir_c.dot(dir_l).abs() > precision::angular() {
            return Ok(false);
        }
        let (line2d, circle2d) = planar_projection(line, circle)?;
        let extrema2d = ExtElC2d::line_circle(line2d, circle2d, precision::confusion())?;
        let inters = AnaIntersection::line_circle(line2d, circle2d)?;
        self.result.done = extrema2d.is_done() || inters.is_done();
        if !self.result.done {
            return Ok(true);
        }
        let nb_extr: i32 = extrema2d.nb_ext()?;
        for id in 1..=nb_extr + inters.nb_points()? {
            let (lin_par, circ_par) = if id <= nb_extr {
                let (on_line, on_circle) = extrema2d.points(id)?;
                (on_line.parameter(), on_circle.parameter())
            } else {
                let point = inters.point(id - nb_extr)?;
                (point.param_on_first(), point.param_on_second()?)
            };
            let on_l: Point3 = elclib::line_value(lin_par, line.position());
            let on_c: Point3 = elclib::circle_value(circ_par, circle.position(), circle.radius());
            let sq_dist: f64 = on_l.square_distance(&on_c);
            self.result.push(
                sq_dist,
                POnCurv::new(lin_par, on_l),
                POnCurv::new(circ_par, on_c),
            );
        }
        Ok(true)
    }
}

// The line and the circle in the circle plane, the circle centred at the origin of
// the standard 2D frame.
fn planar_projection(line: &Line3, circle: &Circle3) -> Result<(Line2, Circle2), Failure> {
    let c_loc: Xyz = circle.location().xyz();
    let (dcx, dcy) = (
        circle.position().x_direction().xyz(),
        circle.position().y_direction().xyz(),
    );
    let l_dir: Xyz = line.direction().xyz();
    let vec_cl: Xyz = line.location().xyz() - c_loc;
    let frame = Axis22d::with_directions(
        Point2::new(0.0, 0.0),
        Direction2::new(1.0, 0.0)?,
        Direction2::new(0.0, 1.0)?,
    )?;
    let circle2d = Circle2::new(frame, circle.radius())?;
    let pl: Point2 = Point2::new(vec_cl.dot(dcx), vec_cl.dot(dcy));
    let dl: Direction2 = Direction2::new(l_dir.dot(dcx), l_dir.dot(dcy))?;
    Ok((Line2::new(pl, dl), circle2d))
}

impl ExtElC {
    /// `Extrema_ExtElC(gp_Lin, gp_Elips)`: the trigonometric equation of the ellipse
    /// parameter; an identity equation is parallel, with the square distance from
    /// the line to the ellipse point at parameter 0;
    /// `ExtElC::line_ellipse(line, ellipse)?`.
    pub fn line_ellipse(line: Line3, ellipse: Ellipse3) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        let frame = conic_frame(&line, ellipse.location(), conic_axes(&ellipse))?;
        let (dx, dy) = (frame.dx, frame.dy);
        let (maj_r, min_r) = (ellipse.major_radius(), ellipse.minor_radius());
        let a5: f64 = maj_r * min_r * dx * dy;
        let (r2, rr2) = (maj_r * maj_r, min_r * min_r);
        let coefficients: [f64; 5] = [
            clamp_small(-2. * a5, 1.0e-12),
            clamp_small((r2 * dx * dx - rr2 * dy * dy - r2 + rr2) / 2.0, 1.0e-12),
            clamp_small(min_r * frame.v.y(), 1.0e-12),
            clamp_small(-maj_r * frame.v.x(), 1.0e-12),
            clamp_small(a5, 1.0e-12),
        ];
        let roots = SortedTrigRoots::new(coefficients, 0., PI + PI)?;
        if !roots.is_done() {
            return Ok(extrema);
        }
        if roots.infinite_roots()? {
            let point: Point3 = elclib::ellipse_value(0., ellipse.position(), maj_r, min_r);
            extrema.result.set_parallel(line.square_distance(point));
            return Ok(extrema);
        }
        for index in 1..=roots.nb_solutions()? {
            extrema.push_conic_parameter(&line, &ellipse, roots.value(index)?);
        }
        extrema.result.done = true;
        Ok(extrema)
    }

    /// `Extrema_ExtElC(gp_Lin, gp_Hypr)`: the logarithms of the positive roots `v` of
    /// a quartic (`Chu = (v² + 1)/2v`, `Shu = (v² - 1)/2v`);
    /// `ExtElC::line_hyperbola(line, hyperbola)?`.
    pub fn line_hyperbola(line: Line3, hyperbola: Hyperbola3) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        let frame = conic_frame(&line, hyperbola.location(), conic_axes(&hyperbola))?;
        let (dx, dy) = (frame.dx, frame.dy);
        let (vx, vy) = (frame.v.x(), frame.v.y());
        let (r, rr) = (hyperbola.major_radius(), hyperbola.minor_radius());
        let a: f64 = -2. * r * rr * dx * dy;
        let b: f64 = -r * r * dx * dx - rr * rr * dy * dy + r * r + rr * rr;
        let a2: f64 = 2. * r * vx + 2. * rr * vy;
        let a4: f64 = -2. * r * vx + 2. * rr * vy;
        let roots = DirectPolynomialRoots::quartic(a + b, a2, 0.0, a4, a - b);
        if !roots.is_done() {
            return Ok(extrema);
        }
        for v in polynomial_roots(&roots)? {
            if v > 0.0 {
                extrema.push_conic_parameter(&line, &hyperbola, v.ln());
            }
        }
        extrema.result.done = true;
        Ok(extrema)
    }

    /// `Extrema_ExtElC(gp_Lin, gp_Parab)`: the roots of a cubic in the parabola
    /// parameter; `ExtElC::line_parabola(line, parabola)?`.
    pub fn line_parabola(line: Line3, parabola: Parabola3) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        let frame = conic_frame(&line, parabola.location(), conic_axes(&parabola))?;
        let (dx, dy) = (frame.dx, frame.dy);
        let p: f64 = parabola.parameter();
        let roots = DirectPolynomialRoots::cubic(
            (1. - dx * dx) / (2.0 * p * p),
            -3.0 * dx * dy / (2.0 * p),
            1. - dy * dy + frame.v.x() / p,
            frame.v.y(),
        );
        if !roots.is_done() {
            return Ok(extrema);
        }
        for u2 in polynomial_roots(&roots)? {
            extrema.push_conic_parameter(&line, &parabola, u2);
        }
        extrema.result.done = true;
        Ok(extrema)
    }
}
