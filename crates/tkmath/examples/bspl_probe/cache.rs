//! Counterparts of the cache and Bezier cases of tkmath_bspl_eval_probe.cxx.
use super::curves::Curve;
use super::{In, fail, o, oa, oi, op, opa, ov, report};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bsplclib::{self, Cache, CacheSpan, CurveD3, CurvePoint};

fn print_d3<P: CurvePoint>((p, v1, v2, v3): CurveD3<P>) {
    op(&p);
    ov::<P>(&v1);
    ov::<P>(&v2);
    ov::<P>(&v3);
}

fn print_levels<P: CurvePoint>(
    d0: P,
    d1: (P, P::Vector),
    d2: (P, P::Vector, P::Vector),
    d3: CurveD3<P>,
) {
    op(&d0);
    op(&d1.0);
    ov::<P>(&d1.1);
    op(&d2.0);
    ov::<P>(&d2.1);
    ov::<P>(&d2.2);
    print_d3::<P>(d3);
}

fn sentinel<P: CurvePoint>(count: i32) -> Array1<'static, P> {
    Array1::with_bounds(1, count).unwrap()
}

fn filled(count: i32) -> Array1<'static, f64> {
    let mut array: Array1<'static, f64> = Array1::with_bounds(1, count).unwrap();
    array.init(91.0);
    array
}

fn span_cache<P: CurvePoint>(
    c: &Curve,
    poles: &Array1<'_, P>,
    u: f64,
    domain: f64,
    params: &[f64],
) -> Result<(), Failure> {
    let mut cp: Array1<'static, P> = sentinel(c.deg + 1);
    let mut cw: Array1<'static, f64> = filled(c.deg + 1);
    bsplclib::build_cache(
        u,
        domain,
        c.per,
        c.deg,
        &c.k,
        poles,
        c.w(),
        &mut cp,
        if c.rat { Some(&mut cw) } else { None },
    )?;
    opa(&cp);
    if c.rat {
        oa(&cw);
    }
    let cws = if c.rat { Some(&cw) } else { None };
    let span = CacheSpan {
        degree: c.deg,
        cache_parameter: u,
        span_length: domain,
    };
    for &t in params {
        let d0: P = bsplclib::cache_d0(t, span, &cp, cws)?;
        let d1 = bsplclib::cache_d1(t, span, &cp, cws)?;
        let d2 = bsplclib::cache_d2(t, span, &cp, cws)?;
        let d3 = bsplclib::cache_d3(t, span, &cp, cws)?;
        print_levels::<P>(d0, d1, d2, d3);
    }
    Ok(())
}

fn matrix_cache<P: CurvePoint>(
    c: &Curve,
    poles: &Array1<'_, P>,
    u: f64,
    domain: f64,
    span: i32,
) -> Result<(), Failure> {
    let width: i32 = P::DIMENSION as i32 + i32::from(c.rat);
    let mut a: Array2<'static, f64> = Array2::with_bounds(1, c.deg + 1, 1, width).unwrap();
    a.init(91.0);
    bsplclib::build_cache_matrix(u, domain, c.per, c.deg, span, &c.k, poles, c.w(), &mut a)?;
    a.as_slice().iter().for_each(|value| o(*value));
    Ok(())
}

fn class_cache<P: CurvePoint>(
    c: &Curve,
    poles: &Array1<'_, P>,
    u: f64,
    params: &[f64],
) -> Result<(), Failure> {
    let cache = Cache::new(c.deg, c.per, &c.k, poles, c.w())?;
    cache.build_cache(u, &c.k, poles, c.w())?;
    for &t in params {
        oi(i32::from(cache.is_cache_valid(t)));
        let d0: P = cache.d0(t)?;
        print_levels::<P>(d0, cache.d1(t)?, cache.d2(t)?, cache.d3(t)?);
    }
    Ok(())
}

fn cache_points<P: CurvePoint>(
    c: &Curve,
    poles: &Array1<'_, P>,
    u: f64,
    domain: f64,
    span: i32,
    params: &[f64],
) {
    report(span_cache(c, poles, u, domain, params), |()| {});
    report(matrix_cache(c, poles, u, domain, span), |()| {});
    report(class_cache(c, poles, u, params), |()| {});
}

// cache: curve(knots = flat) u domain span params
pub fn cache_case(v: &[f64]) {
    let mut f = In::new(v);
    let c = Curve::read(&mut f);
    let (u, domain, span, params) = (f.r(), f.r(), f.i(), f.buf());
    match c.kind {
        2 => cache_points(&c, &c.p2, u, domain, span, &params),
        _ => cache_points(&c, &c.p3, u, domain, span, &params),
    }
    println!();
}

fn bezier_evals<P: CurvePoint>(c: &Curve, poles: &Array1<'_, P>, u: f64) {
    report(bsplclib::bezier_d0(u, poles, c.w()), |p| op(&p));
    report(bsplclib::bezier_d1(u, poles, c.w()), |(p, v1)| {
        op(&p);
        ov::<P>(&v1);
    });
    report(bsplclib::bezier_d2(u, poles, c.w()), |(p, v1, v2)| {
        op(&p);
        ov::<P>(&v1);
        ov::<P>(&v2);
    });
    report(bsplclib::bezier_d3(u, poles, c.w()), print_d3::<P>);
}

fn bezier_points<P: CurvePoint>(c: &Curve, poles: &Array1<'_, P>, nd: i32, u: f64) {
    bezier_evals(c, poles, u);
    let mut np: Array1<'static, P> = sentinel(nd + 1);
    let mut nw: Array1<'static, f64> = filled(nd + 1);
    let result = bsplclib::increase_degree_bezier(
        nd,
        poles,
        c.w(),
        &mut np,
        if c.rat { Some(&mut nw) } else { None },
    );
    report(result, |()| {
        opa(&np);
        if c.rat {
            oa(&nw);
        }
    });
    report(coefficients(c, poles, u), |()| {});
}

fn coefficients<P: CurvePoint>(c: &Curve, poles: &Array1<'_, P>, u: f64) -> Result<(), Failure> {
    let count: i32 = poles.len() as i32;
    let mut cp: Array1<'static, P> = sentinel(count);
    let mut cw: Array1<'static, f64> = filled(count);
    bsplclib::poles_coefficients(
        poles,
        c.w(),
        &mut cp,
        if c.rat { Some(&mut cw) } else { None },
    )?;
    opa(&cp);
    if c.rat {
        oa(&cw);
    }
    let cws = if c.rat { Some(&cw) } else { None };
    let d0: P = bsplclib::coefs_d0(u, &cp, cws)?;
    let d1 = bsplclib::coefs_d1(u, &cp, cws)?;
    let d2 = bsplclib::coefs_d2(u, &cp, cws)?;
    let d3 = bsplclib::coefs_d3(u, &cp, cws)?;
    print_levels::<P>(d0, d1, d2, d3);
    Ok(())
}

// bezier: curve(poles only) newdeg u
pub fn bezier_case(v: &[f64]) {
    let mut f = In::new(v);
    let c = Curve::read(&mut f);
    let (nd, u) = (f.i(), f.r());
    let count: i32 = match c.kind {
        2 => {
            bezier_points(&c, &c.p2, nd, u);
            c.p2.len() as i32
        }
        _ => {
            bezier_points(&c, &c.p3, nd, u);
            c.p3.len() as i32
        }
    };
    let deg: i32 = count - 1;
    match bsplclib::flat_bezier_knots(deg) {
        Ok(knots) => knots[..(2 * (deg + 1)) as usize].iter().for_each(|k| o(*k)),
        Err(failure) => fail(&failure),
    }
    println!();
}
