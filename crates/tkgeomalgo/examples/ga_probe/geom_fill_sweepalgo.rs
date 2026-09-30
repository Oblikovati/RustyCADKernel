//! GeomFill_SweepFunction and GeomFill_Sweep probe operations, the Rust twin of
//! tools/oracle/tkgeomalgo_probe_geomfill_sweepalgo.cxx.
use super::common::{Reader, end, flag, int, pnt, pnt2d, real, show, xy};
use super::geom_fill_laws::{gfl_location, gfl_section};
use super::geom_fill_sweep::{filled, pnts, reals, vecs};
use super::geom_fill_trihedron::gft_curve;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomRectangularTrimmedSurface, GeomSurface};
use rustycad_tkgeomalgo::geom_fill::*;
use rustycad_tkgeombase::approx::{SweepD1, SweepD2, SweepFunction};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};

/// `ga_gfw_location`.
fn gfw_location(r: &mut Reader) -> Result<Handle<dyn GeomFillLocationLaw>, Failure> {
    let spec = gfl_location(r)?;
    let l = spec.law.get().expect("probe law");
    let c = gft_curve(r);
    show(l.set_curve(&c), flag);
    let draft: Handle<GeomFillLocationDraft> = spec.law.cast();
    if let (Some(d), Some(stop)) = (draft.get(), &spec.stop) {
        show(d.set_stop_surf(stop), |()| ());
    }
    let guide: Handle<GeomFillLocationGuide> = spec.law.cast();
    if let (Some(g), Some((sec, set))) = (guide.get(), &spec.section) {
        show(g.set(sec, set[0] != 0.0, set[1], set[2], set[3]), real);
    }
    Ok(spec.law)
}

/// The output arrays of the evaluators.
struct Evals {
    p: Array1<'static, Point3>,
    dp: Array1<'static, Vector3>,
    d2p: Array1<'static, Vector3>,
    w: Array1<'static, f64>,
    dw: Array1<'static, f64>,
    d2w: Array1<'static, f64>,
    p2: Array1<'static, Point2>,
    dp2: Array1<'static, Vector2>,
    d2p2: Array1<'static, Vector2>,
}

impl Evals {
    fn new(np: i32, n2: i32) -> Self {
        let (neg, neg2) = (Vector3::new(-1.0, -1.0, -1.0), Vector2::new(-1.0, -1.0));
        Self {
            p: filled(1, np, Point3::new(-1.0, -1.0, -1.0)),
            dp: filled(1, np, neg),
            d2p: filled(1, np, neg),
            w: filled(1, np, -1.0),
            dw: filled(1, np, -1.0),
            d2w: filled(1, np, -1.0),
            p2: filled(1, n2, Point2::new(-1.0, -1.0)),
            dp2: filled(1, n2, neg2),
            d2p2: filled(1, n2, neg2),
        }
    }
}

fn show2d(a: &Array1<'_, Point2>) {
    a.as_slice().iter().for_each(|p| pnt2d(*p));
}

fn showv2d(a: &Array1<'_, Vector2>) {
    a.as_slice().iter().for_each(|v| xy(v.xy()));
}

/// `ga_gfw_eval`.
fn gfw_eval(f: &dyn SweepFunction, u: f64, np: i32, n2: i32) {
    let mut e = Evals::new(np, n2);
    show(f.d0(u, 0.0, 1.0, &mut e.p, &mut e.p2, &mut e.w), flag);
    pnts(&e.p);
    show2d(&e.p2);
    reals(&e.w);
    let mut e = Evals::new(np, n2);
    let values = SweepD1 {
        poles: &mut e.p,
        d_poles: &mut e.dp,
        poles2d: &mut e.p2,
        d_poles2d: &mut e.dp2,
        weights: &mut e.w,
        d_weights: &mut e.dw,
    };
    show(f.d1(u, 0.0, 1.0, values), flag);
    pnts(&e.p);
    vecs(&e.dp);
    show2d(&e.p2);
    showv2d(&e.dp2);
    reals(&e.w);
    reals(&e.dw);
    gfw_eval_d2(f, u, np, n2);
}

fn gfw_eval_d2(f: &dyn SweepFunction, u: f64, np: i32, n2: i32) {
    let mut e = Evals::new(np, n2);
    let values = SweepD2 {
        d1: SweepD1 {
            poles: &mut e.p,
            d_poles: &mut e.dp,
            poles2d: &mut e.p2,
            d_poles2d: &mut e.dp2,
            weights: &mut e.w,
            d_weights: &mut e.dw,
        },
        d2_poles: &mut e.d2p,
        d2_poles2d: &mut e.d2p2,
        d2_weights: &mut e.d2w,
    };
    show(f.d2(u, 0.0, 1.0, values), flag);
    pnts(&e.p);
    vecs(&e.dp);
    vecs(&e.d2p);
    show2d(&e.p2);
    showv2d(&e.dp2);
    showv2d(&e.d2p2);
    reals(&e.w);
    reals(&e.dw);
    reals(&e.d2w);
}

const SHAPES: [Shape; 7] = [
    Shape::C0,
    Shape::G1,
    Shape::C1,
    Shape::G2,
    Shape::C2,
    Shape::C3,
    Shape::CN,
];

/// `ga_gfw_queries`: returns NbPoles.
fn gfw_queries(f: &dyn SweepFunction) -> i32 {
    show(f.nb_2d_curves(), int);
    let (mut np, mut nk) = (0, 0);
    show(f.section_shape(), |(p, k, d)| {
        (np, nk) = (p, k);
        [p, k, d].into_iter().for_each(int);
    });
    let n = if nk > 0 { nk } else { 1 };
    let mut k = filled(1, n, 0.0);
    show(f.knots(&mut k), |()| {
        int(k.lower());
        reals(&k);
    });
    let mut m = filled(1, n, 0);
    show(f.mults(&mut m), |()| {
        m.as_slice().iter().for_each(|v| int(*v))
    });
    show(f.is_rational(), flag);
    for sh in SHAPES {
        show(f.nb_intervals(sh), |n| {
            int(n);
            let mut t = filled(1, n + 1, -1.0);
            show(f.intervals(&mut t, sh), |()| reals(&t));
        });
    }
    let np1 = if np > 0 { np } else { 1 };
    let mut t = filled(1, np1, 0.0);
    show(f.get_tolerance(1e-3, 1e-2, 0.1, &mut t), |()| reals(&t));
    show(f.resolution(1, 1e-3), |(tu, tv)| {
        [tu, tv].into_iter().for_each(real)
    });
    show(f.barycentre_of_surf(), pnt);
    show(f.maximal_section(), real);
    let mut w = filled(1, np1, 0.0);
    show(f.get_minimal_weight(&mut w), |()| reals(&w));
    np
}

/// Prints the exception of a failed setup and ends the line.
fn fail(error: Failure) {
    print!("{} ", error.exception_type());
    end();
}

/// `location section f fOnS ratio nparams params... a b`.
pub fn function_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let l = match gfw_location(&mut r) {
        Ok(l) => l,
        Err(error) => return fail(error),
    };
    let s = match gfl_section(&mut r) {
        Ok(s) => s,
        Err(error) => return fail(error),
    };
    let (f, fos, ratio) = (r.real(), r.real(), r.real());
    let params = r.reals();
    let (a, b) = (r.real(), r.real());
    let func = GeomFillSweepFunction::new(&s, &l, f, fos, ratio);
    let np = gfw_queries(&func);
    let n2 = l.get().expect("probe law").nb_2d_curves();
    if np <= 0 {
        return end();
    }
    params.iter().for_each(|u| gfw_eval(&func, *u, np, n2));
    show(func.set_interval(a, b), |()| ());
    show(func.set_tolerance(1e-5, 1e-6), |()| ());
    params.iter().for_each(|u| gfw_eval(&func, *u, np, n2));
    end();
}

fn curve2d(c: Handle<dyn Geom2dCurve>) {
    let Some(curve) = c.get() else {
        return int(-1);
    };
    let (f, l) = (curve.first_parameter(), curve.last_parameter());
    real(f);
    real(l);
    show(curve.value(0.5 * (f + l)), pnt2d);
}

/// `ga_gfw_surface`.
pub fn surface(s: &Handle<dyn GeomSurface>) {
    let Some(surf) = s.get() else {
        return int(-1);
    };
    print!("{} ", s.dynamic_type().expect("surface").name());
    let rt: Handle<GeomRectangularTrimmedSurface> = s.cast();
    if let Some(rt) = rt.get() {
        let basis = rt.basis_surface();
        print!("{} ", basis.dynamic_type().expect("basis").name());
    }
    let (u1, u2, v1, v2) = surf.bounds();
    [u1, u2, v1, v2].into_iter().for_each(real);
    flag(surf.is_u_periodic());
    flag(surf.is_v_periodic());
    for tu in [0.1, 0.5, 0.9] {
        for tv in [0.1, 0.5, 0.9] {
            show(surf.value(u1 + (u2 - u1) * tu, v1 + (v2 - v1) * tv), pnt);
        }
    }
    let b: Handle<GeomBSplineSurface> = s.cast();
    if let Some(b) = b.get() {
        [b.nb_u_poles(), b.nb_v_poles(), b.u_degree(), b.v_degree()]
            .into_iter()
            .for_each(int);
        flag(b.is_u_rational());
        flag(b.is_v_rational());
    }
}

/// `ga_gfw_result`.
fn result(sw: &GeomFillSweep) {
    flag(sw.is_done());
    flag(sw.exchange_uv());
    flag(sw.u_reversed());
    flag(sw.v_reversed());
    real(sw.error_on_surface());
    let s = sw.surface();
    surface(&s);
    show(sw.number_of_trace(), int);
    if s.cast::<GeomBSplineSurface>().is_null() {
        return;
    }
    for first in [true, false] {
        show(sw.error_on_restriction(first), |(eu, ev)| {
            [eu, ev].into_iter().for_each(real)
        });
        show(sw.restriction(first), curve2d);
    }
    let nt = sw.number_of_trace().expect("probe law");
    for i in -1..=nt + 1 {
        show(sw.error_on_trace(i), |(eu, ev)| {
            [eu, ev].into_iter().for_each(real)
        });
        show(sw.trace(i), curve2d);
    }
}

/// `location section kpart domain [4] tol[4] forcec1 methode cont degmax segmax cont2`.
pub fn sweep_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let l = match gfw_location(&mut r) {
        Ok(l) => l,
        Err(error) => return fail(error),
    };
    let s = match gfl_section(&mut r) {
        Ok(s) => s,
        Err(error) => return fail(error),
    };
    let (kpart, dom) = (r.flag(), r.flag());
    let d = if dom {
        [r.real(), r.real(), r.real(), r.real()]
    } else {
        [0.0; 4]
    };
    let t = [r.real(), r.real(), r.real(), r.real()];
    let fc = r.flag();
    let methode = if r.integer() == 0 {
        GeomFillApproxStyle::Section
    } else {
        GeomFillApproxStyle::Location
    };
    let (cont, degmax, segmax, cont2) = (r.integer(), r.integer(), r.integer(), r.integer());
    let mut sw = match GeomFillSweep::new(&l, kpart) {
        Ok(sw) => sw,
        Err(error) => return fail(error),
    };
    if dom {
        sw.set_domain(d[0], d[1], d[2], d[3]);
    }
    sw.set_tolerance(t[0], t[1], t[2], t[3]);
    sw.set_force_approx_c1(fc);
    for c in [cont, cont2] {
        show(
            sw.build(&s, methode, SHAPES[c as usize], degmax, segmax),
            |()| (),
        );
        result(&sw);
    }
    end();
}

/// `path curve1 curve2 radius polynomial nparams params... a b`.
pub fn blend_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (p, c1, c2) = (gft_curve(&mut r), gft_curve(&mut r), gft_curve(&mut r));
    let (rad, poly) = (r.real(), r.flag());
    let params = r.reals();
    let (a, b) = (r.real(), r.real());
    let func = match GeomFillCircularBlendFunc::new(&p, &c1, &c2, rad, poly) {
        Ok(func) => func,
        Err(error) => return fail(error),
    };
    let np = gfw_queries(&func);
    params.iter().for_each(|u| gfw_eval(&func, *u, np, 0));
    show(func.set_interval(a, b), |()| ());
    show(func.set_tolerance(1e-5, 1e-6), |()| ());
    params.iter().for_each(|u| gfw_eval(&func, *u, np, 0));
    end();
}
