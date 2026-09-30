// Rust translation of the sphere case of Function_SetUVBounds (ProjLib_ComputeApprox.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The window of a curve on a sphere. A circle counts how often it crosses the seam
//! half-plane (0 or 2: a closed pcurve centred on the circle centre; 1: an open one
//! starting at its first point); other curves use the unwrapping scan.
use super::function::{Projection, UvBounds, compute_step};
use super::uv_bounds::{AngleTrack, Samples, natural_or_centred, next_scan_parameter};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Axis3, Circle3, Plane3, Point3, Sphere3, Transform3, xoy};
use std::f64::consts::PI;

// The tolerance of the seam and pole tests.
const TOL: f64 = 1.0e-10;

/// The sphere case of `Function_SetUVBounds`.
pub(super) fn sphere_bounds(
    p: &Projection<'_>,
    s: &Samples,
    b: &mut UvBounds,
) -> Result<(), Failure> {
    b.v_couture = false;
    let sphere: Sphere3 = p.surface.sphere()?;
    if p.curve.get_type()? == CurveType::Circle {
        return circle_bounds(p, &sphere, s, b);
    }
    let (u1, _) = elslib::parameters(&sphere, s.p1);
    let step: f64 = compute_step(p.curve, sphere.radius())?;
    let mut track: AngleTrack = AngleTrack::new(u1, s.w1);
    let plim: f64 = s.w2 + 0.1 * step;
    let mut par: f64 = s.w1 + step;
    while par <= plim {
        let (u, _) = elslib::parameters(&sphere, p.curve.value(par)?);
        track.push(par, u);
        par = next_scan_parameter(par, step, [s.w1, s.w2])?;
    }
    track.widen(s.w1, s.w2);
    (b.u1, b.u2, b.u_couture) = natural_or_centred(&track);
    b.v_couture = false;
    Ok(())
}

/// The number of intersections of the circle (in the sphere frame) with the seam
/// half-plane `y = 0, x > 0`, from its plane `A x + B y + C z + D = 0`.
// The negated comparisons keep the native positive tests, so a NaN takes the native
// branch.
#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn seam_crossings(a: f64, c: f64, d: f64, r: f64) -> i32 {
    if c.abs() < TOL {
        if !(a.abs() > TOL) || !((d / a) < 0.) {
            return 0;
        }
        if (r - (d / a).abs()) > TOL {
            return 2;
        }
        return if (r - (d / a).abs()).abs() < TOL {
            1
        } else {
            0
        };
    }
    let mut delta: f64 = r * r * (a * a + c * c) - d * d;
    delta *= c * c;
    if delta.abs() < TOL * TOL {
        return if a * d > 0. { 1 } else { 0 };
    }
    if !(delta > 0.) {
        return 0;
    }
    delta = delta.sqrt();
    let mut count: i32 = 0;
    if -a * d + delta > TOL {
        count += 1;
    }
    if -a * d - delta > TOL {
        count += 1;
    }
    count
}

// A U at the seam (0 or 2 PI) is flipped to the other side when the point at
// `probe` lies more than PI away.
fn seam_fixed_u(
    p: &Projection<'_>,
    sphere: &Sphere3,
    mut u: f64,
    probe: f64,
) -> Result<f64, Failure> {
    let eps: f64 = 10. * epsilon(1.);
    if u.abs() < eps {
        let (ud, _) = elslib::parameters(sphere, p.curve.value(probe)?);
        if (u - ud).abs() > PI {
            u = 2. * PI;
        }
    } else if (2. * PI - u).abs() < eps {
        let (ud, _) = elslib::parameters(sphere, p.curve.value(probe)?);
        if (u - ud).abs() > PI {
            u = 0.;
        }
    }
    Ok(u)
}

// The U of the circle centre (in the sphere frame) on the unit sphere.
fn centre_u(circle: &Circle3) -> f64 {
    let (u, _) = elslib::sphere_parameters(Axis3::from_axis2(xoy()), 1., circle.location());
    u
}

fn circle_bounds(
    p: &Projection<'_>,
    sphere: &Sphere3,
    s: &Samples,
    b: &mut UvBounds,
) -> Result<(), Failure> {
    b.u_couture = true;
    let mut circle: Circle3 = p.curve.circle()?;
    let trsf: Transform3 = Transform3::transformation_to(sphere.position());
    circle.transform(trsf)?;
    let r: f64 = sphere.radius();
    let plane: Plane3 = Plane3::from_position(Axis3::from_axis2(circle.position()));
    let [a, _, c, d] = plane.coefficients()?;
    let nb_solutions: i32 = seam_crossings(a, c, d, r);
    let dt: f64 = precision::p_confusion().max(0.01 * (s.w2 - s.w1));
    let (u1, _) = elslib::parameters(sphere, s.p1);
    let u1: f64 = seam_fixed_u(p, sphere, u1, s.w1 + dt)?;
    let (u2_end, _) = elslib::parameters(sphere, s.p2);
    // The corrected U2 is overwritten below natively; only its evaluations remain.
    seam_fixed_u(p, sphere, u2_end, s.w2 - dt)?;
    let _ = elslib::parameters(sphere, s.p);
    let (u2, _) = elslib::parameters(sphere, p.curve.value(s.w1 + PI / 8.)?);
    (b.u1, b.u2) = if nb_solutions == 1 {
        one_crossing_window(u1, u2)
    } else {
        let u: f64 = centre_u(&circle);
        (u - PI, u + PI)
    };
    pole_window(&circle, &trsf, s.p1, r, [c, d], b);
    // box+sphere: the V window is left open.
    (b.v1, b.v2) = (-1.0e+100, 1.0e+100);
    Ok(())
}

// One crossing: the period starting (or ending) at the first point, in the sense
// given by the point a PI/8 further.
fn one_crossing_window(u1: f64, u2: f64) -> (f64, f64) {
    if (u1 - u2).abs() > PI {
        // Crossing the seam.
        if u1 > PI {
            (u1, u1 + 2. * PI)
        } else {
            (u1 - 2. * PI, u1)
        }
    } else if u1 > u2 {
        (u1 - 2. * PI, u1)
    } else {
        (u1, u1 + 2. * PI)
    }
}

// A circle through a pole (its plane tangent to the sphere): the V seam is crossed,
// unless the first point is itself on the sphere.
fn pole_window(
    circle: &Circle3,
    trsf: &Transform3,
    p1: Point3,
    r: f64,
    cd: [f64; 2],
    b: &mut UvBounds,
) {
    let (c, d) = (cd[0], cd[1]);
    if (c == 0.) || ((d / c).abs() - r).abs() > 1.0e-10 {
        b.v_couture = false;
        return;
    }
    b.v_couture = true;
    b.u_couture = true;
    (b.v1, b.v2) = if d / c < 0. {
        (-PI / 2., 3. * PI / 2.)
    } else {
        (-3. * PI / 2., PI / 2.)
    };
    // If P1.Z() equals +/- R we are at the vertex: not significant.
    let pp: Point3 = p1.transformed(*trsf);
    if (pp.x() * pp.x() + pp.y() * pp.y() + pp.z() * pp.z() - r * r).abs() < TOL {
        let u: f64 = centre_u(circle);
        (b.u1, b.u2) = (u - PI, u + PI);
        b.v_couture = false;
    }
}
