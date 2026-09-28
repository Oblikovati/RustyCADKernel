//! Counterparts of tkmath_bspl_misc_probe.cxx: point moves, resolution, function
//! products, extension, knot merging and continuity intervals.
use super::curves::Curve;
use super::eval::{dimension, flat_of};
use super::{In, o, oa, ob, oi, oia, opa, report};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bsplclib::{self, CurvePoint, EvaluatorFunction, FunctionTarget};
use rustycad_tkmath::gp::{Point2, Point3};

fn move_points<P: CurvePoint>(
    c: &Curve,
    poles: &Array1<'static, P>,
    u: f64,
    d: &[f64; 9],
    i: [i32; 4],
    tol: f64,
) {
    let [i1, i2, start, end] = i;
    let (displ, delta, dd) = (
        P::vector_from(&d[0..3]),
        P::vector_from(&d[3..6]),
        P::vector_from(&d[6..9]),
    );
    let mut np: Array1<'static, P> = poles.clone();
    report(
        bsplclib::move_point(u, displ, i1, i2, c.deg, poles, c.w(), &c.k, &mut np),
        |(first, last)| {
            oi(first);
            oi(last);
            opa(&np);
        },
    );
    let mut np: Array1<'static, P> = poles.clone();
    let mut status: i32 = -7;
    report(
        bsplclib::move_point_and_tangent(
            u,
            delta,
            dd,
            tol,
            c.deg,
            start,
            end,
            poles,
            c.w(),
            &c.k,
            &mut np,
            &mut status,
        ),
        |()| {
            oi(status);
            opa(&np);
        },
    );
}

// move: curve(knots = flat) u displ delta dd (3 each) i1 i2 tol start end
pub fn move_case(v: &[f64]) {
    let mut f = In::new(v);
    let c = Curve::read(&mut f);
    let u: f64 = f.r();
    let mut d: [f64; 9] = [0.0; 9];
    d.iter_mut().for_each(|x| *x = f.r());
    let (i1, i2, tol, start, end) = (f.i(), f.i(), f.r(), f.i(), f.i());
    match c.kind {
        2 => move_points::<Point2>(&c, &c.p2, u, &d, [i1, i2, start, end], tol),
        _ => move_points::<Point3>(&c, &c.p3, u, &d, [i1, i2, start, end], tol),
    }
    let flat: Vec<f64> = flat_of(&c);
    let mut np: Vec<f64> = vec![91.0; flat.len()];
    let mut status: i32 = -7;
    let dim: usize = c.kind as usize;
    let result = bsplclib::move_point_and_tangent_flat(
        u,
        c.kind,
        &d[3..3 + dim],
        &d[6..6 + dim],
        tol,
        c.deg,
        start,
        end,
        &flat,
        c.w(),
        &c.k,
        &mut np,
        &mut status,
    );
    report(result, |()| {
        oi(status);
        ob(&np);
    });
    println!();
}

// resolution: curve(knots = flat) numpoles tol
pub fn resolution_case(v: &[f64]) {
    let mut f = In::new(v);
    let c = Curve::read(&mut f);
    let (num, tol) = (f.i(), f.r());
    let flat: Vec<f64> = flat_of(&c);
    report(
        bsplclib::resolution_flat(&flat, dimension(&c), num, c.w(), &c.k, c.deg, tol),
        o,
    );
    match c.kind {
        2 => report(bsplclib::resolution(&c.p2, c.w(), num, &c.k, c.deg, tol), o),
        3 => report(bsplclib::resolution(&c.p3, c.w(), num, &c.k, c.deg, tol), o),
        _ => {}
    }
    println!();
}

struct ProbeFunction {
    c: [f64; 3],
    limit: f64,
}

impl EvaluatorFunction for ProbeFunction {
    fn evaluate(
        &self,
        _request: i32,
        _start_end: &[f64; 2],
        t: f64,
        result: &mut f64,
        error: &mut i32,
    ) {
        if t > self.limit {
            *error = 1;
            return;
        }
        *result = self.c[0] + t * (self.c[1] + t * self.c[2]);
        *error = 0;
    }
}

// function: curve(knots = bspline flat) newdeg newflat c0 c1 c2 limit reparam
pub fn function_case(v: &[f64]) {
    let mut f = In::new(v);
    let c = Curve::read(&mut f);
    let (nd, nf) = (f.i(), f.reals());
    let function = ProbeFunction {
        c: [f.r(), f.r(), f.r()],
        limit: f.r(),
    };
    let reparam: bool = f.b();
    let target = FunctionTarget {
        bspline_degree: c.deg,
        bspline_flat_knots: &c.k,
        flat_knots: &nf,
        new_degree: nd,
    };
    let dim: i32 = dimension(&c);
    let flat: Vec<f64> = flat_of(&c);
    let nnew: i32 = nf.len() as i32 - nd - 1;
    let mut np: Vec<f64> = vec![91.0; (nnew * dim).max(1) as usize];
    let mut status: i32 = -7;
    let result = if reparam {
        bsplclib::function_reparameterise_flat(&function, &target, dim, &flat, &mut np, &mut status)
    } else {
        bsplclib::function_multiply_flat(&function, &target, dim, &flat, &mut np, &mut status)
    };
    report(result, |()| {
        oi(status);
        ob(&np);
    });
    typed_function(&c, &function, &target, nnew, reparam);
    println!();
}

fn typed_function(
    c: &Curve,
    function: &ProbeFunction,
    target: &FunctionTarget<'_, '_>,
    nnew: i32,
    reparam: bool,
) {
    let mut status: i32 = -7;
    if c.kind == 1 && c.dim == 1 {
        let mut np: Array1<'static, f64> = Array1::with_bounds(1, nnew).unwrap();
        np.init(91.0);
        let result = if reparam {
            bsplclib::function_reparameterise_scalar(function, target, &c.f, &mut np, &mut status)
        } else {
            bsplclib::function_multiply_scalar(function, target, &c.f, &mut np, &mut status)
        };
        report(result, |()| {
            oi(status);
            oa(&np);
        });
    } else if c.kind == 2 {
        report(
            typed_points(function, target, &c.p2, nnew, reparam),
            |()| {},
        );
    } else if c.kind == 3 {
        report(
            typed_points(function, target, &c.p3, nnew, reparam),
            |()| {},
        );
    }
}

fn typed_points<P: CurvePoint>(
    function: &ProbeFunction,
    target: &FunctionTarget<'_, '_>,
    poles: &Array1<'_, P>,
    nnew: i32,
    reparam: bool,
) -> Result<(), Failure> {
    let mut np: Array1<'static, P> = Array1::with_bounds(1, nnew).unwrap();
    np.init(P::point_from(&[91.0, 91.0, 91.0]));
    let mut status: i32 = -7;
    if reparam {
        bsplclib::function_reparameterise(function, target, poles, &mut np, &mut status)?;
    } else {
        bsplclib::function_multiply(function, target, poles, &mut np, &mut status)?;
    }
    oi(status);
    opa(&np);
    Ok(())
}

// tang: curve(kind 1, knots = flat) point continuity after c1
pub fn tang_case(v: &[f64]) {
    let mut f = In::new(v);
    let c = Curve::read(&mut f);
    let (point, cont, after, c1) = (f.reals(), f.i(), f.b(), f.r());
    let num: i32 = c.f.len() as i32 / c.dim;
    let mut kr: Vec<f64> = vec![91.0; c.k.len() + 2 * (c.deg + 1) as usize];
    let mut pr: Vec<f64> = vec![91.0; ((num + c.deg) * c.dim) as usize];
    let result = bsplclib::tang_extend_to_constraint(
        &c.k,
        c1,
        num,
        c.f.as_slice(),
        c.dim,
        c.deg,
        &point,
        cont,
        after,
        &mut kr,
        &mut pr,
    );
    report(result, |(nbp, nbk)| {
        oi(nbp);
        oi(nbk);
        ob(&kr[..nbk as usize]);
        ob(&pr[..(nbp * c.dim) as usize]);
    });
    println!();
}

// merge: tol start end d1 k1 m1 d2 k2 m2
pub fn merge_case(v: &[f64]) {
    let mut f = In::new(v);
    let (tol, a, b, d1) = (f.r(), f.r(), f.r(), f.i());
    let (k1, m1, d2, k2, m2) = (f.reals(), f.ints(), f.i(), f.reals(), f.ints());
    report(
        bsplclib::merge_bspline_knots(tol, a, b, d1, &k1, &m1, d2, &k2, &m2),
        |(num, knots, mults)| {
            oi(num);
            oa(&knots);
            oia(&mults);
        },
    );
    println!();
}

// intervals: deg periodic continuity first last tol knots mults
pub fn intervals_case(v: &[f64]) {
    let mut f = In::new(v);
    let (deg, per, cont, first, last, tol) = (f.i(), f.b(), f.i(), f.r(), f.r(), f.r());
    let (k, m) = (f.reals(), f.ints());
    let mut intervals: Array1<'static, f64> = Array1::with_bounds(1, 1).unwrap();
    intervals.init(91.0);
    let result =
        bsplclib::intervals(&k, &m, deg, per, cont, first, last, tol, None).and_then(|count| {
            oi(count);
            bsplclib::intervals(
                &k,
                &m,
                deg,
                per,
                cont,
                first,
                last,
                tol,
                Some(&mut intervals),
            )
        });
    report(result, |count| {
        oi(count);
        oa(&intervals);
    });
    println!();
}
