//! Geom_BezierSurface edition, queries and evaluation representations; the
//! counterpart of tools/oracle/tkg3d_probe_bezier_surface.cxx.
use super::common::{
    Reader, attempt, built_surface, dump, end, flag, int, pnt, real, show, surface_summary, trsf,
};
use super::shapes;
use super::swept::surface_desc;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBezierSurface, GeomGeometry, GeomSurface};
use rustycad_tkg3d::geom_eval::{RepSurfaceDesc, RepSurfaceDescFull};
use rustycad_tkmath::gp::{Point3, Transform3};

fn arrays(b: &GeomBezierSurface) {
    let (nu, nv) = (b.nb_u_poles(), b.nb_v_poles());
    for lower in [1, 0] {
        let mut p =
            Array2::<Point3>::with_bounds(lower, lower + nu - 1, lower, lower + nv - 1).unwrap();
        show(b.poles_into(&mut p), |()| p.iter().copied().for_each(pnt));
        let mut w =
            Array2::<f64>::with_bounds(lower, lower + nu - 1, lower, lower + nv - 1).unwrap();
        show(b.weights_into(&mut w), |()| {
            w.iter().copied().for_each(real)
        });
    }
    let mut p = Array2::<Point3>::with_bounds(1, nu + 1, 1, nv).unwrap();
    attempt(|| b.poles_into(&mut p));
    let mut w = Array2::<f64>::with_bounds(1, nu, 1, nv + 1).unwrap();
    attempt(|| b.weights_into(&mut w));
}

fn state(b: &Handle<GeomBezierSurface>, uv: &[f64]) {
    [b.nb_u_poles(), b.nb_v_poles(), b.u_degree(), b.v_degree()]
        .into_iter()
        .for_each(int);
    [
        b.is_u_rational(),
        b.is_v_rational(),
        b.has_eval_representation(),
    ]
    .into_iter()
    .for_each(flag);
    for i in 0..=b.nb_u_poles() + 1 {
        for j in 0..=b.nb_v_poles() + 1 {
            show(b.pole(i, j), pnt);
            show(b.weight(i, j), real);
        }
    }
    b.weights_array(|w| w.iter().copied().for_each(real));
    flag(b.weights().is_some());
    b.poles().iter().copied().for_each(pnt);
    b.u_knots().iter().copied().for_each(real);
    b.v_knots().iter().copied().for_each(real);
    b.u_multiplicities().unwrap().iter().copied().for_each(int);
    b.v_multiplicities().unwrap().iter().copied().for_each(int);
    b.u_knot_sequence().unwrap().iter().copied().for_each(real);
    b.v_knot_sequence().unwrap().iter().copied().for_each(real);
    for tolerance in [1e-7, 2.5] {
        let (ut, vt) = b.resolution(tolerance).unwrap();
        real(ut);
        real(vt);
    }
    arrays(b);
    surface_summary(&**b, uv);
    dump(&**b, -1);
}

// A line of poles: lower, n, (x y z w)*n, weight lower shift.
struct Line {
    p: Array1<'static, Point3>,
    w: Array1<'static, f64>,
}

fn line(r: &mut Reader) -> Line {
    let (lower, n) = (r.integer(), r.integer());
    let mut p = Array1::<Point3>::with_bounds(lower, lower + n - 1).unwrap();
    let mut values = Vec::new();
    for k in 0..n {
        p[lower + k] = r.pnt();
        values.push(r.real());
    }
    let shift = r.integer();
    let mut w = Array1::<f64>::with_bounds(lower + shift, lower + shift + n - 1).unwrap();
    for (k, value) in values.into_iter().enumerate() {
        w[lower + shift + k as i32] = value;
    }
    Line { p, w }
}

// The arguments of one action: i j P w a b c d line trsf.
struct Args {
    i: i32,
    j: i32,
    p: Point3,
    w: f64,
    abcd: [f64; 4],
    line: Line,
    t: Transform3,
}

fn args(r: &mut Reader) -> Args {
    let (i, j, p, w) = (r.integer(), r.integer(), r.pnt(), r.real());
    let abcd = [r.real(), r.real(), r.real(), r.real()];
    let line = line(r);
    Args {
        i,
        j,
        p,
        w,
        abcd,
        line,
        t: trsf(r),
    }
}

fn insertion(b: &GeomBezierSurface, action: i32, a: &Args) -> Result<(), Failure> {
    let (i, p, w) = (a.i, &a.line.p, &a.line.w);
    match action {
        3 => b.insert_pole_col_after(i, p),
        4 => b.insert_pole_col_after_weighted(i, p, w),
        5 => b.insert_pole_col_before(i, p),
        6 => b.insert_pole_col_before_weighted(i, p, w),
        7 => b.insert_pole_row_after(i, p),
        8 => b.insert_pole_row_after_weighted(i, p, w),
        9 => b.insert_pole_row_before(i, p),
        _ => b.insert_pole_row_before_weighted(i, p, w),
    }
}

fn setter(b: &GeomBezierSurface, action: i32, a: &Args) -> Result<(), Failure> {
    let (i, p, w) = (a.i, &a.line.p, &a.line.w);
    match action {
        14 => b.set_pole(a.i, a.j, a.p),
        15 => b.set_pole_weighted(a.i, a.j, a.p, a.w),
        16 => b.set_pole_col(i, p),
        17 => b.set_pole_col_weighted(i, p, w),
        18 => b.set_pole_row(i, p),
        19 => b.set_pole_row_weighted(i, p, w),
        20 => b.set_weight(a.i, a.j, a.w),
        21 => b.set_weight_col(i, w),
        _ => b.set_weight_row(i, w),
    }
}

/// One action of g3_bzs_action.
fn action(b: &GeomBezierSurface, action: i32, r: &mut Reader) -> Result<(), Failure> {
    let a = args(r);
    match action {
        1 => b.exchange_uv(),
        2 => b.increase(a.i, a.j),
        3..=10 => insertion(b, action, &a),
        11 => b.remove_pole_col(a.i),
        12 => b.remove_pole_row(a.i),
        13 => b.segment(a.abcd[0], a.abcd[1], a.abcd[2], a.abcd[3]),
        14..=22 => setter(b, action, &a),
        23 => b.u_reverse(),
        24 => b.v_reverse(),
        _ => b.transform(a.t),
    }
}

pub fn bezier_surface_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(s) = built_surface(&mut r) else {
        return;
    };
    let uv = r.reals();
    let b: Handle<GeomBezierSurface> = s.cast();
    let count = r.integer();
    state(&b, &uv);
    for _ in 0..count {
        let code = r.integer();
        attempt(|| action(&b, code, &mut r));
        state(&b, &uv);
    }
    state(&Handle::new(GeomBezierSurface::copy_of(&b).unwrap()), &uv);
    int(GeomBezierSurface::max_degree());
    end();
}

fn same(a: &Handle<dyn RepSurfaceDesc>, b: &Handle<dyn RepSurfaceDesc>) -> bool {
    a.object_address() == b.object_address()
}

pub fn bezier_surface_evalrep_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(s) = built_surface(&mut r) else {
        return;
    };
    let uv = r.reals();
    let b: Handle<GeomBezierSurface> = s.cast();
    let rep = shapes::surface(&mut r).unwrap();
    let d = surface_desc(&mut r, rep);
    flag(b.has_eval_representation());
    attempt(|| b.set_eval_representation(d.clone()));
    flag(b.has_eval_representation());
    flag(same(&b.eval_representation(), &d));
    surface_summary(&*b, &uv);
    let c: Handle<GeomBezierSurface> = b.copy().unwrap().cast();
    flag(c.has_eval_representation());
    flag(same(&c.eval_representation(), &d));
    surface_summary(&*c, &uv);
    let own: Handle<dyn RepSurfaceDesc> =
        Handle::new(RepSurfaceDescFull::new(b.clone().cast())).cast();
    attempt(|| b.set_eval_representation(own));
    flag(same(&b.eval_representation(), &d));
    let none: Handle<dyn RepSurfaceDesc> =
        Handle::new(RepSurfaceDescFull::new(Handle::default())).cast();
    attempt(|| b.set_eval_representation(none));
    b.clear_eval_representation();
    flag(b.has_eval_representation());
    let count = r.integer();
    for _ in 0..count {
        let code = r.integer();
        attempt(|| b.set_eval_representation(d.clone()));
        attempt(|| action(&b, code, &mut r));
        flag(b.has_eval_representation());
    }
    attempt(|| b.set_eval_representation(d.clone()));
    surface_summary(&*b, &uv);
    end();
}
