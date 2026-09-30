// Rust twin of tools/oracle/tkgeomalgo_probe_gfill.cxx: the skinning classes of the
// GeomFill filling part, with the curve reader and dumps shared by gfill_*.rs.
use super::common::{Reader, attempt, end, flag, int, pnt, pnt2d, real, show};
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomCircle, GeomCurve, GeomEllipse, GeomLine,
    GeomTrimmedCurve,
};
use rustycad_tkgeomalgo::app_blend::{AppBlendApprox, AppBlendSection};
use rustycad_tkgeomalgo::geom_fill::{
    GeomFillAppSurf, GeomFillLine, GeomFillProfiler, GeomFillSectionGenerator,
};
use rustycad_tkmath::gp::{Axis1, Axis2, Point2, Point3, Vector2, Vector3};

fn array<T: Clone + Default>(lower: i32, upper: i32) -> Array1<'static, T> {
    Array1::with_bounds(lower, upper).expect("probe array")
}

/// Kind 0 of the curve reader.
pub fn read_bspline(r: &mut Reader) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let (deg, periodic, rational, nk) = (r.integer(), r.flag(), r.flag(), r.integer());
    let mut k: Array1<'static, f64> = array(1, nk);
    let mut m: Array1<'static, i32> = array(1, nk);
    (1..=nk).for_each(|i| k[i] = r.real());
    (1..=nk).for_each(|i| m[i] = r.integer());
    let np = r.integer();
    let mut p: Array1<'static, Point3> = array(1, np);
    (1..=np).for_each(|i| p[i] = r.pnt());
    if !rational {
        return Ok(Handle::new(GeomBSplineCurve::new(
            &p, &k, &m, deg, periodic,
        )?));
    }
    let mut w: Array1<'static, f64> = array(1, np);
    (1..=np).for_each(|i| w[i] = r.real());
    Ok(Handle::new(GeomBSplineCurve::with_weights(
        &p, &w, &k, &m, deg, periodic,
    )?))
}

/// Kind 1 of the curve reader.
pub fn read_bezier(r: &mut Reader) -> Result<Handle<GeomBezierCurve>, Failure> {
    let np = r.integer();
    let mut p: Array1<'static, Point3> = array(1, np);
    (1..=np).for_each(|i| p[i] = r.pnt());
    if !r.flag() {
        return Ok(Handle::new(GeomBezierCurve::new(&p)?));
    }
    let mut w: Array1<'static, f64> = array(1, np);
    (1..=np).for_each(|i| w[i] = r.real());
    Ok(Handle::new(GeomBezierCurve::with_weights(&p, &w)?))
}

/// The curve reader of `ga_gf_curve`.
pub fn read_curve(r: &mut Reader) -> Result<Handle<dyn GeomCurve>, Failure> {
    match r.integer() {
        0 => Ok(read_bspline(r)?.cast()),
        1 => Ok(read_bezier(r)?.cast()),
        2 => {
            let (c, n) = (r.pnt(), r.dir());
            let radius = r.real();
            Ok(Handle::new(GeomCircle::new(Axis2::new(c, n)?, radius)?).cast())
        }
        3 => {
            let (c, n) = (r.pnt(), r.dir());
            let (a, b) = (r.real(), r.real());
            Ok(Handle::new(GeomEllipse::new(Axis2::new(c, n)?, a, b)?).cast())
        }
        4 => {
            let (u1, u2) = (r.real(), r.real());
            let basis = read_curve(r)?;
            Ok(Handle::new(GeomTrimmedCurve::new(&basis, u1, u2, true, true)?).cast())
        }
        _ => {
            let p = r.pnt();
            Ok(Handle::new(GeomLine::new(Axis1::new(p, r.dir()))).cast())
        }
    }
}

/// `ga_gf_dump_bspline`.
pub fn dump_bspline(c: &Handle<GeomBSplineCurve>) {
    let Some(c) = c.get() else {
        print!("null ");
        return;
    };
    int(c.degree());
    flag(c.is_periodic());
    flag(c.is_rational());
    int(c.nb_knots());
    c.knots().iter().for_each(|&k| real(k));
    c.multiplicities().iter().for_each(|&m| int(m));
    int(c.nb_poles());
    c.poles().iter().for_each(|&p| pnt(p));
    c.weights_array(|w| w.iter().for_each(|&x| real(x)));
}

/// `ga_gf_add_curves`: the reader errors never happen on the corpus.
fn add_curves(r: &mut Reader, p: &mut GeomFillProfiler) {
    let n = r.integer();
    for _ in 0..n {
        let curve = read_curve(r).expect("probe curve");
        attempt(|| p.add_curve(&curve));
    }
}

fn count_curves(p: &GeomFillProfiler) -> i32 {
    let mut n = 0;
    while p.curve(n + 1).is_ok() {
        n += 1;
    }
    n
}

fn profiler_report(p: &GeomFillProfiler, n: i32) {
    flag(p.is_periodic());
    show(p.degree(), int);
    show(p.nb_poles(), int);
    show(p.nb_knots(), int);
    let (np, nk) = match (p.nb_poles(), p.nb_knots()) {
        (Ok(np), Ok(nk)) => (np, nk),
        _ => (0, 0),
    };
    for extra in 0..2 {
        let mut k: Array1<'static, f64> = array(1, nk + extra);
        let mut m: Array1<'static, i32> = array(1, nk);
        show(p.knots_and_mults(&mut k, &mut m), |()| {
            k.iter().for_each(|&x| real(x));
            m.iter().for_each(|&x| int(x));
        });
    }
    for i in 0..=n + 1 {
        let mut q: Array1<'static, Point3> = array(1, np);
        show(p.poles(i, &mut q), |()| {
            int(q.lower());
            q.iter().for_each(|&x| pnt(x));
        });
        let mut w: Array1<'static, f64> = array(0, np - 1);
        show(p.weights(i, &mut w), |()| {
            int(w.lower());
            w.iter().for_each(|&x| real(x));
        });
        show(p.curve(i), |c| dump_bspline(&c.cast()));
    }
    let mut q: Array1<'static, Point3> = array(1, np + 1);
    attempt(|| p.poles(1, &mut q));
}

pub fn profiler_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (ptol, perform) = (r.real(), r.flag());
    let mut p = GeomFillProfiler::new();
    add_curves(&mut r, &mut p);
    let n = count_curves(&p);
    profiler_report(&p, n);
    if perform {
        attempt(|| p.perform(ptol));
        profiler_report(&p, n);
    }
    end();
}

fn section_arrays(np: i32) -> AppBlendSection {
    let mut poles2d: Array1<'static, Point2> = array(1, 1);
    poles2d[1] = Point2::new(7.0, 8.0);
    AppBlendSection {
        poles: array(1, np),
        d_poles: array::<Vector3>(1, np),
        poles2d,
        d_poles2d: array::<Vector2>(1, 1),
        weights: array(1, np),
        d_weights: array(1, np),
    }
}

fn secgen_report(g: &GeomFillSectionGenerator, n: i32, params: bool) {
    let mut shape = [1, 1, 0, 0];
    show(g.get_shape(), |s| {
        shape = s;
        s.into_iter().for_each(int);
    });
    let (np, nk) = (shape[0], shape[1]);
    let mut k: Array1<'static, f64> = array(1, nk);
    show(g.knots(&mut k), |()| {
        int(k.lower());
        k.iter().for_each(|&x| real(x));
    });
    let mut m: Array1<'static, i32> = array(1, nk);
    show(g.mults(&mut m), |()| m.iter().for_each(|&x| int(x)));
    for i in 0..=n + 1 {
        let mut s = section_arrays(np);
        show(g.section_d1(i, &mut s), |d| {
            flag(d);
            int(s.poles.length());
            s.poles.iter().for_each(|&x| pnt(x));
            s.weights.iter().for_each(|&x| real(x));
            pnt2d(s.poles2d[1]);
        });
        if params {
            show(g.parameter(i), real);
        }
    }
}

pub fn secgen_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (ptol, lower) = (r.real(), r.integer());
    let prm = r.reals();
    let mut g = GeomFillSectionGenerator::new();
    add_curves(&mut r, &mut g);
    let n = count_curves(&g);
    attempt(|| g.perform(ptol));
    if !prm.is_empty() {
        let upper = lower + prm.len() as i32 - 1;
        let h = Handle::new(HArray1::with_bounds(lower, upper).expect("probe array"));
        for (i, &x) in prm.iter().enumerate() {
            h.get().unwrap().change_array1()[lower + i as i32] = x;
        }
        attempt(|| g.set_param(&h));
        h.get().unwrap().array1().iter().for_each(|&x| real(x));
    }
    secgen_report(&g, n, !prm.is_empty());
    end();
}

fn appsurf_report(s: &GeomFillAppSurf) {
    flag(s.is_done());
    show(s.surf_shape(), |shape| shape.into_iter().for_each(int));
    show(
        s.surf_poles().and_then(|p| Ok((p, s.surf_weights()?))),
        |(p, w)| {
            for i in p.lower_row()..=p.upper_row() {
                for j in p.lower_col()..=p.upper_col() {
                    pnt(p[(i, j)]);
                    real(w[(i, j)]);
                }
            }
        },
    );
    show(
        s.surf_u_knots().and_then(|k| Ok((k, s.surf_u_mults()?))),
        |(k, m)| {
            k.iter().for_each(|&x| real(x));
            m.iter().for_each(|&x| int(x));
        },
    );
    show(
        s.surf_v_knots().and_then(|k| Ok((k, s.surf_v_mults()?))),
        |(k, m)| {
            k.iter().for_each(|&x| real(x));
            m.iter().for_each(|&x| int(x));
        },
    );
    show(
        s.u_degree().and_then(|u| Ok((u, s.v_degree()?))),
        |(u, v)| {
            int(u);
            int(v);
        },
    );
    show(s.tol_reached(), |(t3, t2)| {
        real(t3);
        real(t2);
    });
    show(s.nb_curves2d(), int);
    show(s.curves2d_degree(), int);
}

pub fn appsurf_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let ptol = r.real();
    let prm = r.reals();
    let (mode, dmin, dmax) = (r.integer(), r.integer(), r.integer());
    let (t3, t2, nbit, knownp) = (r.real(), r.real(), r.integer(), r.flag());
    let (nbmaxp, nlin) = (r.integer(), r.integer());
    let mut g = GeomFillSectionGenerator::new();
    add_curves(&mut r, &mut g);
    attempt(|| g.perform(ptol));
    if !prm.is_empty() {
        let mut values: Array1<'static, f64> = array(1, prm.len() as i32);
        values.as_mut_slice().copy_from_slice(&prm);
        let h = Handle::new(HArray1::from_owned(values));
        attempt(|| g.set_param(&h));
    }
    let line = Handle::new(GeomFillLine::with_nb_points(nlin));
    int(line.nb_points());
    let mut s = GeomFillAppSurf::new(dmin, dmax, t3, t2, nbit, knownp);
    for _ in 0..2 {
        attempt(|| match mode {
            0 => s.perform(line.get(), &mut g, false),
            1 => s.perform(line.get(), &mut g, true),
            2 => s.perform_smoothing(line.get(), &mut g),
            _ => s.perform_by_pieces(line.get(), &mut g, nbmaxp),
        });
        appsurf_report(&s);
    }
    end();
}

pub fn line_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let n = r.integer();
    let idx = r.reals();
    let line = GeomFillLine::with_nb_points(n);
    int(line.nb_points());
    idx.iter().for_each(|&i| int(line.point(i as i32)));
    int(GeomFillLine::new().nb_points());
    end();
}
