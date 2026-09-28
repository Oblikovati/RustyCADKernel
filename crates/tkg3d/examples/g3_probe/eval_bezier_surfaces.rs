//! The GeomEval Bezier surfaces of the TKG3d probe; the counterpart of
//! tools/oracle/tkg3d_probe_eval_bezier_surfaces.cxx.
use super::common::{built_surface, dump, end, flag, int, name, pnt, real, show, surface_summary, trsf, vec};
use super::print::Reader;
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::geom_eval::{GeomEvalAHTBezierSurface, GeomEvalTBezierSurface};
use rustycad_tkmath::gp::{Point3, Transform3};

// The pole and weight grids of a spec, both starting at (lr, lc).
fn grids(r: &mut Reader, lr: i32, lc: i32) -> (Array2<'static, Point3>, Array2<'static, f64>) {
    let (nu, nv) = (r.integer(), r.integer());
    let mut poles = Array2::with_bounds(lr, lr + nu - 1, lc, lc + nv - 1).unwrap();
    for i in lr..lr + nu {
        for j in lc..lc + nv {
            poles.set_value(i, j, r.pnt()).unwrap();
        }
    }
    let (nwu, nwv) = (r.integer(), r.integer());
    let mut weights = Array2::with_bounds(lr, lr + nwu - 1, lc, lc + nwv - 1).unwrap();
    for i in lr..lr + nwu {
        for j in lc..lc + nwv {
            weights.set_value(i, j, r.real()).unwrap();
        }
    }
    (poles, weights)
}

/// One eval Bezier surface specification: 30 T-Bezier, 31 AHT-Bezier.
pub fn spec(kind: i32, r: &mut Reader) -> Result<Handle<dyn GeomSurface>, Failure> {
    let (rational, lr, lc) = (r.flag(), r.integer(), r.integer());
    if kind == 30 {
        let (p, w) = grids(r, lr, lc);
        let (au, av) = (r.real(), r.real());
        let surface = if rational {
            GeomEvalTBezierSurface::with_weights(&p, &w, au, av)?
        } else {
            GeomEvalTBezierSurface::new(&p, au, av)?
        };
        return Ok(Handle::new(surface).cast());
    }
    assert_eq!(kind, 31, "unknown eval Bezier surface kind {kind}");
    let degrees = [r.integer(), r.integer()];
    let alphas = [r.real(), r.real()];
    let betas = [r.real(), r.real()];
    let (p, w) = grids(r, lr, lc);
    let surface = if rational {
        GeomEvalAHTBezierSurface::with_weights(&p, &w, degrees, alphas, betas)?
    } else {
        GeomEvalAHTBezierSurface::new(&p, degrees, alphas, betas)?
    };
    Ok(Handle::new(surface).cast())
}

fn grid<T: Copy>(a: &Array2<'_, T>, values: bool, print: impl Fn(T)) {
    int(a.lower_row());
    int(a.lower_col());
    int(a.nb_rows() as i32);
    int(a.nb_columns() as i32);
    if values {
        a.as_slice().iter().for_each(|&x| print(x));
    }
}

fn state(s: &Handle<dyn GeomSurface>) {
    let t: Handle<GeomEvalTBezierSurface> = s.cast();
    if !t.is_null() {
        [t.nb_u_poles(), t.nb_v_poles(), t.order_u(), t.order_v()].into_iter().for_each(int);
        [t.alpha_u(), t.alpha_v()].into_iter().for_each(real);
        flag(t.is_rational());
        grid(&t.poles(), true, pnt);
        grid(&t.weights(), true, real);
    }
    let a: Handle<GeomEvalAHTBezierSurface> = s.cast();
    if !a.is_null() {
        [a.nb_poles_u(), a.nb_poles_v(), a.alg_degree_u(), a.alg_degree_v()].into_iter().for_each(int);
        [a.alpha_u(), a.alpha_v(), a.beta_u(), a.beta_v()].into_iter().for_each(real);
        flag(a.is_u_rational());
        flag(a.is_v_rational());
        grid(&a.poles(), true, pnt);
        grid(&a.weights(), a.is_u_rational(), real);
    }
}

// EvalDN of total orders 4 to 6, beyond the summary's 0..3 per direction.
fn high_orders(s: &Handle<dyn GeomSurface>, uv: &[f64]) {
    for pair in uv.as_chunks::<2>().0 {
        for nu in 0..=5 {
            for nv in (4 - nu).max(0)..=6 - nu {
                show(s.eval_dn(pair[0], pair[1], nu, nv), vec);
            }
        }
    }
}

fn refusals(s: &Handle<dyn GeomSurface>, t: Transform3) {
    show(s.u_reverse(), |()| {});
    show(s.v_reverse(), |()| {});
    show(s.u_iso(0.5), |c| name(&c));
    show(s.v_iso(0.5), |c| name(&c));
    show(s.transform(t), |()| {});
    show(s.transformed(t), |g| name(&g));
    show(s.u_reversed(), |c| name(&c));
}

/// g3_eval_bezier_surface: accessors of the surface and its copy, the copy's summary,
/// high EvalDN orders, the refused members and the dumps.
pub fn eval_bezier_surface_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(s) = built_surface(&mut r) else {
        return;
    };
    let uv = r.reals();
    let t = trsf(&mut r);
    name(&s);
    state(&s);
    let copy: Handle<dyn GeomSurface> = s.copy().unwrap().cast();
    name(&copy);
    state(&copy);
    surface_summary(&*copy, &uv);
    high_orders(&s, &uv);
    show(s.eval_dn(0.5, 0.5, 2, -1), vec);
    show(s.eval_dn(0.5, 0.5, 0, 0), vec);
    refusals(&s, t);
    for depth in [-1, 0, 1, 2] {
        dump(&*s, depth);
    }
    end();
}
