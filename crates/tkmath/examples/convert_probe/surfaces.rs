//! Elementary surface conversions of tools/oracle/tkmath_convert_probe.cxx.
use super::{arr, arr_i, attempt, frame3, grid_pts, grid_reals};
use crate::bspl_probe::{In, o, oi, op, report};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::convert::ElementarySurfaceToBSplineSurface as Surface;
use rustycad_tkmath::gp::{Cone3, Cylinder3, Sphere3, Torus3};

fn surface_out(c: &Surface) {
    for value in [
        c.u_degree(),
        c.v_degree(),
        c.nb_u_poles(),
        c.nb_v_poles(),
        c.nb_u_knots(),
        c.nb_v_knots(),
    ] {
        oi(value);
    }
    oi(i32::from(c.is_u_periodic()));
    oi(i32::from(c.is_v_periodic()));
    grid_pts(c.poles());
    grid_reals(c.weights());
    arr(c.u_knots());
    arr(c.v_knots());
    arr_i(c.u_multiplicities());
    arr_i(c.v_multiplicities());
    let (nu, nv) = (c.nb_u_poles(), c.nb_v_poles());
    for (i, j) in [(0, 1), (1, 0), (1, 1), (nu, nv), (nu + 1, nv), (nu, nv + 1)] {
        report(c.pole(i, j), |p| op(&p));
        report(c.weight(i, j), o);
    }
    for k in [0, 1, c.nb_u_knots(), c.nb_u_knots() + 1] {
        report(c.u_knot(k), o);
        report(c.u_multiplicity(k), oi);
    }
    for k in [0, 1, c.nb_v_knots(), c.nb_v_knots() + 1] {
        report(c.v_knot(k), o);
        report(c.v_multiplicity(k), oi);
    }
}

struct Request {
    kind: i32,
    mode: i32,
    frame: [f64; 10],
    a: f64,
    b: f64,
    p: [f64; 4],
    u_trim: bool,
}

fn convert(q: &Request) -> Result<Option<Surface>, Failure> {
    let ax = frame3(&q.frame)?;
    let [p1, p2, p3, p4] = q.p;
    Ok(match (q.kind, q.mode) {
        (0, 0) => Some(Surface::from_sphere(&Sphere3::new(ax, q.a)?)?),
        (0, 1) => Some(Surface::from_sphere_patch(
            &Sphere3::new(ax, q.a)?,
            p1,
            p2,
            p3,
            p4,
        )?),
        (0, 2) => Some(Surface::from_sphere_band(
            &Sphere3::new(ax, q.a)?,
            p1,
            p2,
            q.u_trim,
        )?),
        (1, 0) => Some(Surface::from_torus(&Torus3::new(ax, q.a, q.b)?)?),
        (1, 1) => Some(Surface::from_torus_patch(
            &Torus3::new(ax, q.a, q.b)?,
            p1,
            p2,
            p3,
            p4,
        )?),
        (1, 2) => Some(Surface::from_torus_band(
            &Torus3::new(ax, q.a, q.b)?,
            p1,
            p2,
            q.u_trim,
        )?),
        (2, 0) => Some(Surface::from_cylinder(&Cylinder3::new(ax, q.a)?, p1, p2)?),
        (2, 1) => Some(Surface::from_cylinder_patch(
            &Cylinder3::new(ax, q.a)?,
            p1,
            p2,
            p3,
            p4,
        )?),
        (3, 0) => Some(Surface::from_cone(&Cone3::new(ax, q.b, q.a)?, p1, p2)?),
        (3, 1) => Some(Surface::from_cone_patch(
            &Cone3::new(ax, q.b, q.a)?,
            p1,
            p2,
            p3,
            p4,
        )?),
        // The corpus sends no other (kind, mode) pair.
        _ => None,
    })
}

// surface: kind(0 sphere,1 torus,2 cylinder,3 cone) mode(0 whole/periodic,1 patch,2 band) frame3 a b p1 p2 p3 p4 utrim
pub fn surface_case(values: &[f64]) {
    let mut f = In::new(values);
    let (kind, mode) = (f.i(), f.i());
    let frame: [f64; 10] = std::array::from_fn(|_| f.r());
    let (a, b) = (f.r(), f.r());
    let p: [f64; 4] = std::array::from_fn(|_| f.r());
    let q = Request {
        kind,
        mode,
        frame,
        a,
        b,
        p,
        u_trim: f.b(),
    };
    attempt(|| {
        if let Some(surface) = convert(&q)? {
            surface_out(&surface);
        }
        Ok(())
    });
    println!();
}
