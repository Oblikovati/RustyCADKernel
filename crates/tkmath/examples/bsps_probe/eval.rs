//! Evaluation, isoparametric curves, flat pole arrays and knot operations.
use super::{Surf, grid1, grid3, od2, og1, og3, ov, pgrid, wgrid};
use crate::bspl_probe::{In, fail, o, oa, ob, oi, oia, op, opa, report};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::bsplslib::{self, SurfaceD3};
use rustycad_tkmath::gp::Point3;

// eval: surface uindex vindex nu nv count (u v)...
pub fn eval_case(v: &[f64]) {
    let mut f = In::new(v);
    let s = Surf::read(&mut f);
    let (ui, vi, nu, nv, n) = (f.i(), f.i(), f.i(), f.i(), f.i());
    for _ in 0..n {
        let (u, v) = (f.r(), f.r());
        let span = s.span(u, v, ui, vi);
        report(bsplslib::d0(&span, &s.p, s.w()), |p| op(&p));
        report(bsplslib::d1(&span, &s.p, s.w()), |(p, vu, vv)| {
            op(&p);
            ov(&vu);
            ov(&vv);
        });
        report(bsplslib::d2(&span, &s.p, s.w()), |d| od2(&d));
        report(bsplslib::d3(&span, &s.p, s.w()), |d| od3(&d));
        report(bsplslib::dn(&span, nu, nv, &s.p, s.w()), |vn| ov(&vn));
        report(bsplslib::homogeneous_d0(&span, &s.p, s.w()), |(w, p)| {
            o(w);
            op(&p);
        });
        report(
            bsplslib::homogeneous_d1(&span, &s.p, s.w()),
            |(p, vu, vv, d, du, dv)| {
                op(&p);
                ov(&vu);
                ov(&vv);
                [d, du, dv].into_iter().for_each(o);
            },
        );
    }
    println!();
}

fn od3(d: &SurfaceD3) {
    op(&d.p);
    [
        d.vu, d.vv, d.vuu, d.vvv, d.vuv, d.vuuu, d.vvvv, d.vuuv, d.vuvv,
    ]
    .iter()
    .for_each(ov);
}

// rational: udeg vdeg n m all ders
pub fn rational_case(v: &[f64]) {
    let mut f = In::new(v);
    let (ud, vd, n, m, all) = (f.i(), f.i(), f.i(), f.i(), f.b());
    let h = f.buf();
    let mut r: Vec<f64> = vec![91.0; (3 * (n + 1) * (m + 1)) as usize];
    report(
        bsplslib::rational_derivative(ud, vd, n, m, &h, &mut r, all),
        |()| ob(&r),
    );
    println!();
}

// iso: surface param isu cweights clower count | pairs (i1 i2 j1 j2 eps)... | last udir
pub fn iso_case(v: &[f64]) {
    let mut f = In::new(v);
    let s = Surf::read(&mut f);
    let (param, isu, cw, cl, cn) = (f.r(), f.b(), f.b(), f.i(), f.i());
    let mut cp: Array1<'static, Point3> = Array1::with_bounds(cl, cl + cn - 1).unwrap();
    cp.init(Point3::new(91.0, 91.0, 91.0));
    let mut cwts: Array1<'static, f64> = Array1::with_bounds(cl, cl + cn - 1).unwrap();
    cwts.init(91.0);
    let (knots, mults) = if isu {
        (&s.uk, s.umults())
    } else {
        (&s.vk, s.vmults())
    };
    let (degree, periodic) = if isu { (s.ud, s.up) } else { (s.vd, s.vp) };
    let weights = if cw { Some(&mut cwts) } else { None };
    let result = bsplslib::iso(
        param,
        isu,
        &s.p,
        s.w(),
        knots,
        mults,
        degree,
        periodic,
        &mut cp,
        weights,
    );
    report(result, |()| {
        opa(&cp);
        if cw {
            oa(&cwts);
        }
    });
    for _ in 0..f.i() {
        let (i1, i2, j1, j2, eps) = (f.i(), f.i(), f.i(), f.i(), f.r());
        report(bsplslib::is_rational(&s.w, i1, i2, j1, j2, eps), |r| {
            oi(i32::from(r))
        });
    }
    let (last, udir) = (f.i(), f.b());
    let mut rp = s.p.clone();
    report(bsplslib::reverse(&mut rp, last, udir), |()| og3(&rp));
    let mut rw = s.w.clone();
    report(bsplslib::reverse(&mut rw, last, udir), |()| og1(&rw));
    println!();
}

// poles: poles weights udir fplower fp
pub fn poles_case(v: &[f64]) {
    let mut f = In::new(v);
    let (p, w) = (grid3(&mut f), grid1(&mut f));
    let (udir, fl) = (f.b(), f.i());
    let n: i32 = p.len() as i32;
    let mut f3: Array1<'static, f64> = Array1::with_bounds(fl, fl + 3 * n - 1).unwrap();
    f3.init(91.0);
    let mut f4: Array1<'static, f64> = Array1::with_bounds(fl, fl + 4 * n - 1).unwrap();
    f4.init(91.0);
    report(bsplslib::set_poles(&p, &mut f3, udir), |()| oa(&f3));
    report(bsplslib::set_rational_poles(&p, &w, &mut f4, udir), |()| {
        oa(&f4)
    });
    let g = f.reals();
    let (rl, ru, cl, cu) = (p.lower_row(), p.upper_row(), p.lower_col(), p.upper_col());
    let mut q: Array2<'static, Point3> = Array2::with_bounds(rl, ru, cl, cu).unwrap();
    q.init(Point3::new(91.0, 91.0, 91.0));
    let mut vw: Array2<'static, f64> = Array2::with_bounds(rl, ru, cl, cu).unwrap();
    vw.init(91.0);
    report(bsplslib::get_poles(&g, &mut q, udir), |()| og3(&q));
    report(
        bsplslib::get_rational_poles(&g, &mut q, &mut vw, udir),
        |()| {
            og3(&q);
            og1(&vw);
        },
    );
    println!();
}

/// Output grids of a surface knot operation along one direction.
struct Grids {
    p: Array2<'static, Point3>,
    w: Array2<'static, f64>,
    k: Array1<'static, f64>,
    m: Array1<'static, i32>,
}

impl Grids {
    fn new(s: &Surf, udir: bool, np: i32, nk: i32) -> Self {
        let (rows, cols) = (s.p.nb_rows() as i32, s.p.nb_columns() as i32);
        let (r, c) = if udir { (np, cols) } else { (rows, np) };
        let mut k: Array1<'static, f64> = Array1::with_bounds(1, nk).unwrap();
        k.init(91.0);
        let mut m: Array1<'static, i32> = Array1::with_bounds(1, nk).unwrap();
        m.init(-7);
        Self {
            p: pgrid(r, c),
            w: wgrid(r, c),
            k,
            m,
        }
    }
    fn print(&self, s: &Surf) {
        og3(&self.p);
        if s.hw {
            og1(&self.w);
        }
        oa(&self.k);
        oia(&self.m);
    }
}

/// The mutable output arguments of a knot operation: poles, weights, knots, mults.
type Outputs<'g> = (
    &'g mut Array2<'static, Point3>,
    Option<&'g mut Array2<'static, f64>>,
    &'g mut Array1<'static, f64>,
    &'g mut Array1<'static, i32>,
);

// Splits the output grids into the argument references of a knot operation.
fn outputs<'g>(r: &'g mut Grids, s: &Surf) -> Outputs<'g> {
    let Grids { p, w, k, m } = r;
    (p, if s.hw { Some(w) } else { None }, k, m)
}

/// The direction data of a knot operation.
struct Dir<'s> {
    udir: bool,
    deg: i32,
    per: bool,
    k: &'s Array1<'static, f64>,
    m: &'s Array1<'static, i32>,
}

fn direction(s: &Surf, udir: bool) -> Dir<'_> {
    if udir {
        return Dir {
            udir,
            deg: s.ud,
            per: s.up,
            k: &s.uk,
            m: &s.um_,
        };
    }
    Dir {
        udir,
        deg: s.vd,
        per: s.vp,
        k: &s.vk,
        m: &s.vm,
    }
}

// ops: surface udir | insert | remove | degree | (unperiodize)
pub fn ops_case(v: &[f64]) {
    let mut f = In::new(v);
    let s = Surf::read(&mut f);
    let d = direction(&s, f.b());
    insert(&mut f, &s, &d);
    remove(&mut f, &s, &d);
    degree(&mut f, &s, &d);
    if d.per {
        unperiodize(&s, &d);
    }
    println!();
}

fn insert(f: &mut In<'_>, s: &Surf, d: &Dir<'_>) {
    let (ak, useam, am, eps, add) = (f.reals(), f.b(), f.ints(), f.r(), f.b());
    let am = if useam { Some(&am) } else { None };
    let (mut np, mut nk): (i32, i32) = (-7, -7);
    let ok: bool =
        bsplclib::prepare_insert_knots(d.deg, d.per, d.k, d.m, &ak, am, &mut np, &mut nk, eps, add)
            .unwrap();
    oi(i32::from(ok));
    oi(np);
    oi(nk);
    if !ok {
        return;
    }
    let mut r = Grids::new(s, d.udir, np, nk);
    let (rp, rw, rk, rm) = outputs(&mut r, s);
    let result = bsplslib::insert_knots(
        d.udir,
        d.deg,
        d.per,
        &s.p,
        s.w(),
        d.k,
        d.m,
        &ak,
        am,
        rp,
        rw,
        rk,
        rm,
        eps,
        add,
    );
    report(result, |()| r.print(s));
}

fn remove(f: &mut In<'_>, s: &Surf, d: &Dir<'_>) {
    let (index, mult, tol, np, nk) = (f.i(), f.i(), f.r(), f.i(), f.i());
    let mut r = Grids::new(s, d.udir, np, nk);
    let (rp, rw, rk, rm) = outputs(&mut r, s);
    let result = bsplslib::remove_knot(
        d.udir,
        index,
        mult,
        d.deg,
        d.per,
        &s.p,
        s.w(),
        d.k,
        d.m,
        rp,
        rw,
        rk,
        rm,
        tol,
    );
    report(result, |ok| {
        oi(i32::from(ok));
        r.print(s);
    });
}

fn degree(f: &mut In<'_>, s: &Surf, d: &Dir<'_>) {
    let (nd, np) = (f.i(), f.i());
    let nk: i32 = match bsplclib::increase_degree_count_knots(d.deg, nd, d.per, d.m) {
        Ok(nk) => nk,
        Err(failure) => return fail(&failure),
    };
    oi(nk);
    let mut r = Grids::new(s, d.udir, np, nk);
    let (rp, rw, rk, rm) = outputs(&mut r, s);
    let result = bsplslib::increase_degree(
        d.udir,
        d.deg,
        nd,
        d.per,
        &s.p,
        s.w(),
        d.k,
        d.m,
        rp,
        rw,
        rk,
        rm,
    );
    report(result, |()| r.print(s));
}

fn unperiodize(s: &Surf, d: &Dir<'_>) {
    let (nk, np) = match bsplclib::prepare_unperiodize(d.deg, d.m) {
        Ok(counts) => counts,
        Err(failure) => return fail(&failure),
    };
    oi(nk);
    oi(np);
    let mut r = Grids::new(s, d.udir, np, nk);
    let (rp, rw, rk, rm) = outputs(&mut r, s);
    let result = bsplslib::unperiodize(d.udir, d.deg, d.m, d.k, &s.p, s.w(), rm, rk, rp, rw);
    report(result, |()| r.print(s));
}
